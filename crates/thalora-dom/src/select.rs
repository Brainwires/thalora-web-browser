//! CSS selector matching (`querySelector`, `matches`, `closest`) via the
//! `selectors` crate.

use std::fmt;

use cssparser::ToCss;
use html5ever::{LocalName, Namespace};
use precomputed_hash::PrecomputedHash;
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::bloom::BloomFilter;
use selectors::context::{
    MatchingContext, MatchingForInvalidation, MatchingMode, NeedsSelectorFlags, QuirksMode,
    SelectorCaches,
};
use selectors::matching::{ElementSelectorFlags, matches_selector_list};
use selectors::parser::{ParseRelative, SelectorList, SelectorParseErrorKind};
use selectors::{OpaqueElement, parser};

use crate::node::{ElementData, NodeData};
use crate::{Document, DomError, DomResult, NodeId, QuirksMode as DocQuirks};

/// The `selectors` implementation types for this DOM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomSelectorImpl;

/// A CSS string (attribute selector value) usable by `selectors`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CssString(pub String);

impl<'a> From<&'a str> for CssString {
    fn from(s: &'a str) -> Self {
        CssString(s.to_string())
    }
}

impl AsRef<str> for CssString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToCss for CssString {
    fn to_css<W: fmt::Write>(&self, dest: &mut W) -> fmt::Result {
        cssparser::serialize_string(&self.0, dest)
    }
}

/// An interned name (tag, id, class, attribute) usable by `selectors`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CssLocalName(pub LocalName);

impl<'a> From<&'a str> for CssLocalName {
    fn from(s: &'a str) -> Self {
        CssLocalName(LocalName::from(s))
    }
}

impl ToCss for CssLocalName {
    fn to_css<W: fmt::Write>(&self, dest: &mut W) -> fmt::Result {
        cssparser::serialize_identifier(&self.0, dest)
    }
}

impl PrecomputedHash for CssLocalName {
    fn precomputed_hash(&self) -> u32 {
        self.0.precomputed_hash()
    }
}

/// Supported non-tree-structural pseudo-classes.
///
/// Interaction state (hover, focus, ...) does not exist yet, so those parse
/// but never match. Form-state pseudo-classes use content attributes
/// (`checked`, `disabled`, `required`) as an approximation of the live
/// state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonTSPseudoClass {
    /// `:any-link`
    AnyLink,
    /// `:link`
    Link,
    /// `:visited` (never matches)
    Visited,
    /// `:hover` (never matches yet)
    Hover,
    /// `:active` (never matches yet)
    Active,
    /// `:focus` (never matches yet)
    Focus,
    /// `:focus-visible` (never matches yet)
    FocusVisible,
    /// `:focus-within` (never matches yet)
    FocusWithin,
    /// `:target` (never matches yet)
    Target,
    /// `:checked`
    Checked,
    /// `:disabled`
    Disabled,
    /// `:enabled`
    Enabled,
    /// `:required`
    Required,
    /// `:optional`
    Optional,
}

impl NonTSPseudoClass {
    fn css_name(self) -> &'static str {
        match self {
            NonTSPseudoClass::AnyLink => "any-link",
            NonTSPseudoClass::Link => "link",
            NonTSPseudoClass::Visited => "visited",
            NonTSPseudoClass::Hover => "hover",
            NonTSPseudoClass::Active => "active",
            NonTSPseudoClass::Focus => "focus",
            NonTSPseudoClass::FocusVisible => "focus-visible",
            NonTSPseudoClass::FocusWithin => "focus-within",
            NonTSPseudoClass::Target => "target",
            NonTSPseudoClass::Checked => "checked",
            NonTSPseudoClass::Disabled => "disabled",
            NonTSPseudoClass::Enabled => "enabled",
            NonTSPseudoClass::Required => "required",
            NonTSPseudoClass::Optional => "optional",
        }
    }

    const ALL: [NonTSPseudoClass; 14] = [
        NonTSPseudoClass::AnyLink,
        NonTSPseudoClass::Link,
        NonTSPseudoClass::Visited,
        NonTSPseudoClass::Hover,
        NonTSPseudoClass::Active,
        NonTSPseudoClass::Focus,
        NonTSPseudoClass::FocusVisible,
        NonTSPseudoClass::FocusWithin,
        NonTSPseudoClass::Target,
        NonTSPseudoClass::Checked,
        NonTSPseudoClass::Disabled,
        NonTSPseudoClass::Enabled,
        NonTSPseudoClass::Required,
        NonTSPseudoClass::Optional,
    ];
}

impl ToCss for NonTSPseudoClass {
    fn to_css<W: fmt::Write>(&self, dest: &mut W) -> fmt::Result {
        dest.write_char(':')?;
        dest.write_str(self.css_name())
    }
}

impl parser::NonTSPseudoClass for NonTSPseudoClass {
    fn is_active_or_hover(&self) -> bool {
        matches!(self, NonTSPseudoClass::Active | NonTSPseudoClass::Hover)
    }

    fn is_user_action_state(&self) -> bool {
        matches!(
            self,
            NonTSPseudoClass::Active
                | NonTSPseudoClass::Hover
                | NonTSPseudoClass::Focus
                | NonTSPseudoClass::FocusVisible
                | NonTSPseudoClass::FocusWithin
        )
    }
}

/// Supported pseudo-elements. They parse (so `querySelector("p::before")`
/// is not a `SyntaxError`) but never match an element, as the DOM requires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PseudoElement {
    /// `::before`
    Before,
    /// `::after`
    After,
    /// `::first-line`
    FirstLine,
    /// `::first-letter`
    FirstLetter,
    /// `::marker`
    Marker,
    /// `::placeholder`
    Placeholder,
    /// `::selection`
    Selection,
}

impl PseudoElement {
    fn css_name(self) -> &'static str {
        match self {
            PseudoElement::Before => "before",
            PseudoElement::After => "after",
            PseudoElement::FirstLine => "first-line",
            PseudoElement::FirstLetter => "first-letter",
            PseudoElement::Marker => "marker",
            PseudoElement::Placeholder => "placeholder",
            PseudoElement::Selection => "selection",
        }
    }

    const ALL: [PseudoElement; 7] = [
        PseudoElement::Before,
        PseudoElement::After,
        PseudoElement::FirstLine,
        PseudoElement::FirstLetter,
        PseudoElement::Marker,
        PseudoElement::Placeholder,
        PseudoElement::Selection,
    ];
}

impl ToCss for PseudoElement {
    fn to_css<W: fmt::Write>(&self, dest: &mut W) -> fmt::Result {
        dest.write_str("::")?;
        dest.write_str(self.css_name())
    }
}

impl parser::PseudoElement for PseudoElement {}

impl parser::SelectorImpl for DomSelectorImpl {
    type ExtraMatchingData<'a> = ();
    type AttrValue = CssString;
    type Identifier = CssLocalName;
    type LocalName = CssLocalName;
    type NamespaceUrl = Namespace;
    type NamespacePrefix = CssLocalName;
    type BorrowedNamespaceUrl = Namespace;
    type BorrowedLocalName = CssLocalName;
    type NonTSPseudoClass = NonTSPseudoClass;
    type PseudoElement = PseudoElement;
}

struct SelectorParser;

impl<'i> parser::Parser<'i> for SelectorParser {
    type Impl = DomSelectorImpl;
    type Error = SelectorParseErrorKind;

    fn parse_is_and_where(&self) -> bool {
        true
    }

    fn parse_has(&self) -> bool {
        true
    }

    fn parse_nth_child_of(&self) -> bool {
        true
    }

    fn parse_non_ts_pseudo_class(
        &self,
        name: &str,
    ) -> Result<NonTSPseudoClass, cssparser::ParseError<SelectorParseErrorKind>> {
        NonTSPseudoClass::ALL
            .into_iter()
            .find(|pc| pc.css_name().eq_ignore_ascii_case(name))
            .ok_or_else(|| {
                cssparser::ParseError::custom(
                    SelectorParseErrorKind::UnsupportedPseudoClassOrElement,
                )
            })
    }

    fn parse_pseudo_element(
        &self,
        name: &str,
    ) -> Result<PseudoElement, cssparser::ParseError<SelectorParseErrorKind>> {
        PseudoElement::ALL
            .into_iter()
            .find(|pe| pe.css_name().eq_ignore_ascii_case(name))
            .ok_or_else(|| {
                cssparser::ParseError::custom(
                    SelectorParseErrorKind::UnsupportedPseudoClassOrElement,
                )
            })
    }
}

/// A parsed selector list, reusable across matches (the binding can cache
/// these per selector string).
#[derive(Debug, Clone)]
pub struct Selector {
    list: SelectorList<DomSelectorImpl>,
}

impl Selector {
    /// Parses a selector list. Invalid selectors yield [`DomError::Syntax`].
    pub fn parse(selectors: &str) -> DomResult<Selector> {
        let mut input = cssparser::Parser::new(selectors);
        input
            .parse_entirely(|p| SelectorList::parse(&SelectorParser, p, ParseRelative::No))
            .map(|list| Selector { list })
            .map_err(|e| DomError::Syntax(format!("invalid selector {selectors:?}: {:?}", e.kind)))
    }
}

impl ToCss for Selector {
    fn to_css<W: fmt::Write>(&self, dest: &mut W) -> fmt::Result {
        self.list.to_css(dest)
    }
}

/// An element of a [`Document`] as seen by the `selectors` matcher.
#[derive(Clone, Copy)]
pub struct ElementRef<'a> {
    doc: &'a Document,
    index: u32,
}

impl fmt::Debug for ElementRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ElementRef({:?})", self.id())
    }
}

impl<'a> ElementRef<'a> {
    /// Wraps `id` if it names an element of `doc`.
    pub fn new(doc: &'a Document, id: NodeId) -> DomResult<ElementRef<'a>> {
        let index = doc.check(id)?;
        Self::from_index(doc, index).ok_or(DomError::InvalidNodeType)
    }

    fn from_index(doc: &'a Document, index: u32) -> Option<ElementRef<'a>> {
        doc.node(index)
            .data
            .is_element()
            .then_some(ElementRef { doc, index })
    }

    /// The element's node id.
    pub fn id(&self) -> NodeId {
        self.doc.id_of(self.index)
    }

    fn data(&self) -> &'a ElementData {
        self.doc
            .element(self.index)
            .expect("ElementRef always wraps an element")
    }

    fn sibling_element(&self, forward: bool) -> Option<Self> {
        let mut s = self.step(self.index, forward);
        while let Some(i) = s {
            if let Some(e) = Self::from_index(self.doc, i) {
                return Some(e);
            }
            s = self.step(i, forward);
        }
        None
    }

    fn step(&self, i: u32, forward: bool) -> Option<u32> {
        let n = self.doc.node(i);
        if forward {
            n.next_sibling
        } else {
            n.prev_sibling
        }
    }

    fn is_form_control(&self) -> bool {
        let e = self.data();
        e.is_html()
            && matches!(
                &*e.name.local,
                "button" | "input" | "select" | "textarea" | "optgroup" | "option" | "fieldset"
            )
    }
}

impl selectors::Element for ElementRef<'_> {
    type Impl = DomSelectorImpl;

    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(self.doc.node(self.index))
    }

    fn parent_element(&self) -> Option<Self> {
        let p = self.doc.node(self.index).parent?;
        Self::from_index(self.doc, p)
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }

    fn is_pseudo_element(&self) -> bool {
        false
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        self.sibling_element(false)
    }

    fn next_sibling_element(&self) -> Option<Self> {
        self.sibling_element(true)
    }

    fn first_element_child(&self) -> Option<Self> {
        let mut c = self.doc.node(self.index).first_child;
        while let Some(i) = c {
            if let Some(e) = Self::from_index(self.doc, i) {
                return Some(e);
            }
            c = self.doc.node(i).next_sibling;
        }
        None
    }

    fn is_html_element_in_html_document(&self) -> bool {
        self.data().is_html()
    }

    fn has_local_name(&self, local_name: &CssLocalName) -> bool {
        self.data().name.local == local_name.0
    }

    fn has_namespace(&self, ns: &Namespace) -> bool {
        self.data().name.ns == *ns
    }

    fn is_same_type(&self, other: &Self) -> bool {
        let (a, b) = (self.data(), other.data());
        a.name.local == b.name.local && a.name.ns == b.name.ns
    }

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&Namespace>,
        local_name: &CssLocalName,
        operation: &AttrSelectorOperation<&CssString>,
    ) -> bool {
        self.data().attrs.iter().any(|a| {
            let ns_ok = match ns {
                NamespaceConstraint::Any => true,
                NamespaceConstraint::Specific(url) => a.name.ns == **url,
            };
            ns_ok && a.name.local == local_name.0 && operation.eval_str(&a.value)
        })
    }

    fn match_non_ts_pseudo_class(
        &self,
        pc: &NonTSPseudoClass,
        _context: &mut MatchingContext<'_, DomSelectorImpl>,
    ) -> bool {
        let e = self.data();
        match pc {
            NonTSPseudoClass::AnyLink | NonTSPseudoClass::Link => self.is_link(),
            NonTSPseudoClass::Visited
            | NonTSPseudoClass::Hover
            | NonTSPseudoClass::Active
            | NonTSPseudoClass::Focus
            | NonTSPseudoClass::FocusVisible
            | NonTSPseudoClass::FocusWithin
            | NonTSPseudoClass::Target => false,
            NonTSPseudoClass::Checked => {
                (e.is_html_named("input")
                    && e.attr("type").is_some_and(|t| {
                        t.eq_ignore_ascii_case("checkbox") || t.eq_ignore_ascii_case("radio")
                    })
                    && e.attr("checked").is_some())
                    || (e.is_html_named("option") && e.attr("selected").is_some())
            }
            NonTSPseudoClass::Disabled => self.is_form_control() && e.attr("disabled").is_some(),
            NonTSPseudoClass::Enabled => self.is_form_control() && e.attr("disabled").is_none(),
            NonTSPseudoClass::Required | NonTSPseudoClass::Optional => {
                let input_like = e.is_html_named("input")
                    || e.is_html_named("select")
                    || e.is_html_named("textarea");
                let required = e.attr("required").is_some();
                input_like && (required == matches!(pc, NonTSPseudoClass::Required))
            }
        }
    }

    fn match_pseudo_element(
        &self,
        _pe: &PseudoElement,
        _context: &mut MatchingContext<'_, DomSelectorImpl>,
    ) -> bool {
        false
    }

    fn apply_selector_flags(&self, _flags: ElementSelectorFlags) {}

    fn is_link(&self) -> bool {
        let e = self.data();
        e.is_html() && matches!(&*e.name.local, "a" | "area" | "link") && e.attr("href").is_some()
    }

    fn is_html_slot_element(&self) -> bool {
        self.data().is_html_named("slot")
    }

    fn has_id(&self, id: &CssLocalName, case_sensitivity: CaseSensitivity) -> bool {
        self.data()
            .id()
            .is_some_and(|v| case_sensitivity.eq(id.0.as_bytes(), v.as_bytes()))
    }

    fn has_class(&self, name: &CssLocalName, case_sensitivity: CaseSensitivity) -> bool {
        self.data().attr("class").is_some_and(|classes| {
            classes
                .split_ascii_whitespace()
                .any(|c| case_sensitivity.eq(name.0.as_bytes(), c.as_bytes()))
        })
    }

    fn has_custom_state(&self, _name: &CssLocalName) -> bool {
        false
    }

    fn imported_part(&self, _name: &CssLocalName) -> Option<CssLocalName> {
        None
    }

    fn is_part(&self, _name: &CssLocalName) -> bool {
        false
    }

    fn is_empty(&self) -> bool {
        let mut c = self.doc.node(self.index).first_child;
        while let Some(i) = c {
            match &self.doc.node(i).data {
                NodeData::Element(_) => return false,
                NodeData::Text(t) if !t.is_empty() => return false,
                _ => {}
            }
            c = self.doc.node(i).next_sibling;
        }
        true
    }

    fn is_root(&self) -> bool {
        self.doc
            .node(self.index)
            .parent
            .is_some_and(|p| matches!(self.doc.node(p).data, NodeData::Document))
    }

    fn add_element_unique_hashes(&self, _filter: &mut BloomFilter) -> bool {
        false
    }
}

impl Document {
    fn selector_quirks(&self) -> QuirksMode {
        match self.quirks_mode() {
            DocQuirks::NoQuirks => QuirksMode::NoQuirks,
            DocQuirks::LimitedQuirks => QuirksMode::LimitedQuirks,
            DocQuirks::Quirks => QuirksMode::Quirks,
        }
    }

    fn matches_index(
        &self,
        selector: &Selector,
        index: u32,
        scope: Option<u32>,
        caches: &mut SelectorCaches,
    ) -> bool {
        let Some(element) = ElementRef::from_index(self, index) else {
            return false;
        };
        let mut context = MatchingContext::new(
            MatchingMode::Normal,
            None,
            caches,
            self.selector_quirks(),
            NeedsSelectorFlags::No,
            MatchingForInvalidation::No,
        );
        context.scope_element = scope
            .and_then(|s| ElementRef::from_index(self, s))
            .map(|s| selectors::Element::opaque(&s));
        matches_selector_list(&selector.list, element, &mut context)
    }

    /// Validates a `querySelector*` scope (document, element or fragment).
    fn query_scope(&self, scope: NodeId) -> DomResult<u32> {
        let i = self.check(scope)?;
        match self.node(i).data {
            NodeData::Document | NodeData::Element(_) | NodeData::DocumentFragment => Ok(i),
            _ => Err(DomError::InvalidNodeType),
        }
    }

    /// `querySelector` with a pre-parsed selector.
    pub fn select_first(&self, scope: NodeId, selector: &Selector) -> DomResult<Option<NodeId>> {
        let root = self.query_scope(scope)?;
        let mut caches = SelectorCaches::default();
        let mut cur = root;
        while let Some(n) = self.next_in_subtree(cur, root) {
            if self.matches_index(selector, n, Some(root), &mut caches) {
                return Ok(Some(self.id_of(n)));
            }
            cur = n;
        }
        Ok(None)
    }

    /// `querySelectorAll` with a pre-parsed selector (a static list, in tree
    /// order).
    pub fn select_all(&self, scope: NodeId, selector: &Selector) -> DomResult<Vec<NodeId>> {
        let root = self.query_scope(scope)?;
        let mut caches = SelectorCaches::default();
        let mut out = Vec::new();
        let mut cur = root;
        while let Some(n) = self.next_in_subtree(cur, root) {
            if self.matches_index(selector, n, Some(root), &mut caches) {
                out.push(self.id_of(n));
            }
            cur = n;
        }
        Ok(out)
    }

    /// `Element.matches` with a pre-parsed selector.
    pub fn element_matches(&self, element: NodeId, selector: &Selector) -> DomResult<bool> {
        let e = ElementRef::new(self, element)?;
        Ok(self.matches_index(
            selector,
            e.index,
            Some(e.index),
            &mut SelectorCaches::default(),
        ))
    }

    /// `Element.closest` with a pre-parsed selector.
    pub fn element_closest(
        &self,
        element: NodeId,
        selector: &Selector,
    ) -> DomResult<Option<NodeId>> {
        let start = ElementRef::new(self, element)?.index;
        let mut caches = SelectorCaches::default();
        let mut n = Some(start);
        while let Some(i) = n {
            if self.node(i).data.is_element()
                && self.matches_index(selector, i, Some(start), &mut caches)
            {
                return Ok(Some(self.id_of(i)));
            }
            n = self.node(i).parent;
        }
        Ok(None)
    }

    /// `ParentNode.querySelector`: the first descendant element of `scope`
    /// (document, element or fragment) matching `selectors`, in tree order.
    pub fn query_selector(&self, scope: NodeId, selectors: &str) -> DomResult<Option<NodeId>> {
        self.select_first(scope, &Selector::parse(selectors)?)
    }

    /// `ParentNode.querySelectorAll`: every matching descendant element, in
    /// tree order.
    pub fn query_selector_all(&self, scope: NodeId, selectors: &str) -> DomResult<Vec<NodeId>> {
        self.select_all(scope, &Selector::parse(selectors)?)
    }

    /// `Element.matches`.
    pub fn matches(&self, element: NodeId, selectors: &str) -> DomResult<bool> {
        self.element_matches(element, &Selector::parse(selectors)?)
    }

    /// `Element.closest`: the nearest inclusive ancestor element matching
    /// `selectors`.
    pub fn closest(&self, element: NodeId, selectors: &str) -> DomResult<Option<NodeId>> {
        self.element_closest(element, &Selector::parse(selectors)?)
    }
}
