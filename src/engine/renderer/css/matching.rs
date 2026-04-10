use super::types::{CompiledRule, ComputedStyles, CssProcessor, ParsedRule, RuleIndex};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

/// Compare two ParsedRules for cascade ordering.
/// CSS Cascade 5 order: layered styles < unlayered styles.
/// Within layered: earlier-declared layers < later-declared layers.
/// Within same layer (or both unlayered): specificity, then source order.
fn cascade_cmp(a: &ParsedRule, b: &ParsedRule) -> Ordering {
    // Layer comparison: None (unlayered) > Some (layered)
    match (&a.layer, &b.layer) {
        (None, Some(_)) => return Ordering::Greater, // unlayered wins
        (Some(_), None) => return Ordering::Less,    // layered loses
        (Some(a_layer), Some(b_layer)) => {
            // Within layers: higher index = later-declared = wins
            let layer_cmp = a_layer.cmp(b_layer);
            if layer_cmp != Ordering::Equal {
                return layer_cmp;
            }
        }
        (None, None) => {} // both unlayered, fall through
    }
    // Same layer or both unlayered: specificity then source order
    a.specificity
        .cmp(&b.specificity)
        .then(a.source_order.cmp(&b.source_order))
}

impl CssProcessor {
    // ── Section 1: Selector compilation & indexing ───────────────────────

    /// Pre-compile all CSS selectors for fast matching.
    /// Call this after all stylesheets have been parsed, before computing styles.
    /// This avoids re-parsing selector strings on every element match.
    pub fn compile_selectors(&mut self) {
        let start = std::time::Instant::now();
        self.compiled_rules.clear();
        self.compiled_rules.reserve(self.rules.len());

        for (idx, rule) in self.rules.iter().enumerate() {
            let selector_alternatives: Vec<&str> =
                rule.selector.split(',').map(|s| s.trim()).collect();
            let mut compiled_selectors = Vec::with_capacity(selector_alternatives.len());
            let mut has_hover = false;
            let mut hover_base_selectors = Vec::new();
            let mut key_tags = Vec::with_capacity(selector_alternatives.len());
            let mut key_classes = Vec::with_capacity(selector_alternatives.len());
            let mut key_ids = Vec::with_capacity(selector_alternatives.len());

            for raw_selector in &selector_alternatives {
                if raw_selector.is_empty() {
                    compiled_selectors.push(None);
                    hover_base_selectors.push(None);
                    key_tags.push(None);
                    key_classes.push(None);
                    key_ids.push(None);
                    continue;
                }

                // Extract key tag, class, and ID from rightmost simple selector for indexed lookup
                key_tags.push(Self::extract_key_tag(raw_selector));
                key_classes.push(Self::extract_key_class(raw_selector));
                key_ids.push(Self::extract_key_id(raw_selector));

                // Check for :hover
                let is_hover = Self::contains_hover_pseudo(raw_selector);
                if is_hover {
                    has_hover = true;
                }

                // Strip `:not(:focus)`, `:not(:hover)`, `:not(:active)`, `:not(:focus-visible)`
                // from selectors before compilation. In static rendering, elements are never
                // focused/hovered/active, so these negations are always true.
                let preprocessed = raw_selector
                    .replace(":not(:focus-visible)", "")
                    .replace(":not(:focus-within)", "")
                    .replace(":not(:focus)", "")
                    .replace(":not(:hover)", "")
                    .replace(":not(:active)", "");
                let sel_to_compile = if preprocessed.trim().is_empty() {
                    raw_selector.to_string()
                } else {
                    preprocessed
                };

                // Preprocess :is(), :where(), :has() before compilation.
                // scraper doesn't support these natively.
                let sel_to_compile = Self::preprocess_modern_pseudos(&sel_to_compile);

                // Compile the full selector
                let compiled = scraper::Selector::parse(&sel_to_compile).ok();
                compiled_selectors.push(compiled);

                // For hover selectors, also compile the base (with :hover stripped)
                if is_hover {
                    let base = Self::strip_hover_pseudo(raw_selector);
                    if base.is_empty() {
                        hover_base_selectors.push(None); // bare :hover matches anything
                    } else {
                        hover_base_selectors.push(scraper::Selector::parse(&base).ok());
                    }
                } else {
                    hover_base_selectors.push(None);
                }
            }

            self.compiled_rules.push(CompiledRule {
                rule_index: idx,
                compiled_selectors,
                has_hover,
                hover_base_selectors,
                key_tags,
                key_classes,
                key_ids,
            });
        }

        // Build the rule index: group compiled rules by key tag, key class, key id, or universal
        let mut by_tag: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_class: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_id: HashMap<String, Vec<usize>> = HashMap::new();
        let mut universal: Vec<usize> = Vec::new();

        for (ci, compiled) in self.compiled_rules.iter().enumerate() {
            // A rule is indexed by the FIRST selector alternative that has a key tag, class, or id.
            // If any alternative has no key tag/class/id, the rule must go in universal.
            let mut has_specific = false;
            let mut has_universal_alt = false;

            for i in 0..compiled.key_tags.len() {
                let kt = compiled.key_tags.get(i).and_then(|x| x.as_ref());
                let kc = compiled.key_classes.get(i).and_then(|x| x.as_ref());
                let ki = compiled.key_ids.get(i).and_then(|x| x.as_ref());

                if kt.is_some() || kc.is_some() || ki.is_some() {
                    has_specific = true;
                    if let Some(tag) = kt {
                        by_tag.entry(tag.clone()).or_default().push(ci);
                    }
                    if let Some(cls) = kc {
                        by_class.entry(cls.clone()).or_default().push(ci);
                    }
                    if let Some(id) = ki {
                        by_id.entry(id.clone()).or_default().push(ci);
                    }
                } else {
                    has_universal_alt = true;
                }
            }

            // If any selector alternative is universal (no tag/class/id), the rule must also
            // be in the universal bucket so it gets checked for all elements.
            if has_universal_alt || !has_specific {
                universal.push(ci);
            }
        }

        eprintln!(
            "[TIMING] compile_selectors: {}ms ({} rules, {} universal, {} tag-indexed, {} class-indexed, {} id-indexed)",
            start.elapsed().as_millis(),
            self.rules.len(),
            universal.len(),
            by_tag.len(),
            by_class.len(),
            by_id.len()
        );

        self.rule_index = Some(RuleIndex {
            by_tag,
            by_class,
            by_id,
            universal,
        });
    }

    /// Extract the tag name from the rightmost simple selector of a CSS selector string.
    /// Returns None for class-only, ID-only, or universal selectors.
    /// Examples:
    ///   "nav > ul > li > a.link" → Some("a")
    ///   ".container"             → None
    ///   "div.flex"               → Some("div")
    ///   "#main"                  → None
    ///   "h1"                     → Some("h1")
    ///   "*"                      → None
    fn extract_key_tag(selector: &str) -> Option<String> {
        // Get the last simple selector (after the last combinator: space, >, +, ~)
        let last = selector
            .rsplit([' ', '>', '+', '~'])
            .next()
            .unwrap_or(selector)
            .trim();
        if last.is_empty() {
            return None;
        }
        // Extract tag name: everything before first '.', '#', ':', '['
        let tag_end = last.find(['.', '#', ':', '[']).unwrap_or(last.len());
        let tag = &last[..tag_end];
        if tag.is_empty() || tag == "*" {
            None
        } else {
            Some(tag.to_lowercase())
        }
    }

    /// Extract the ID from the rightmost simple selector of a CSS selector string.
    /// Returns None for tag-only, class-only, or universal selectors.
    /// Examples:
    ///   "#main"        → Some("main")
    ///   "div#content"  → Some("content")
    ///   ".container"   → None
    ///   "h1"           → None
    fn extract_key_id(selector: &str) -> Option<String> {
        let last = selector
            .rsplit([' ', '>', '+', '~'])
            .next()
            .unwrap_or(selector)
            .trim();
        if last.is_empty() {
            return None;
        }
        // Find first '#' that starts an ID
        if let Some(hash_pos) = last.find('#') {
            let after_hash = &last[hash_pos + 1..];
            // ID ends at next '.', '#', ':', '['
            let id_end = after_hash
                .find(['.', '#', ':', '['])
                .unwrap_or(after_hash.len());
            let id = &after_hash[..id_end];
            if !id.is_empty() {
                return Some(id.to_lowercase());
            }
        }
        None
    }

    /// Extract the first class name from the rightmost simple selector of a CSS selector string.
    /// Returns None for tag-only, ID-only, or universal selectors.
    /// Examples:
    ///   ".container"        → Some("container")
    ///   "div.flex"          → Some("flex")
    ///   "a.nav-link.active" → Some("nav-link")
    ///   "#main"             → None
    ///   "h1"                → None
    fn extract_key_class(selector: &str) -> Option<String> {
        let last = selector
            .rsplit([' ', '>', '+', '~'])
            .next()
            .unwrap_or(selector)
            .trim();
        if last.is_empty() {
            return None;
        }
        // Find first '.' that starts a class name
        if let Some(dot_pos) = last.find('.') {
            let after_dot = &last[dot_pos + 1..];
            // Class name ends at next '.', '#', ':', '['
            let class_end = after_dot
                .find(['.', '#', ':', '['])
                .unwrap_or(after_dot.len());
            let class = &after_dot[..class_end];
            if !class.is_empty() {
                return Some(class.to_lowercase());
            }
        }
        None
    }

    // ── Section 2: Rule matching & style computation ────────────────────

    /// Get all rules that match a given selector
    pub fn get_matching_rules(&self, selector: &str) -> Vec<&ParsedRule> {
        self.rules
            .iter()
            .filter(|rule| self.selectors_match(&rule.selector, selector))
            .collect()
    }

    /// Compute styles for an element given its selector chain (legacy string-based matching)
    /// selector_chain is a list of selectors from root to the element (e.g., ["html", "body", "div.container", "p"])
    pub fn compute_style(&self, selector: &str) -> ComputedStyles {
        let mut styles = ComputedStyles::default();

        // Collect all matching rules sorted by specificity
        let mut matching_rules: Vec<&ParsedRule> = self
            .rules
            .iter()
            .filter(|rule| self.selector_applies(&rule.selector, selector))
            .collect();

        // Sort by cascade layer, then specificity, then source order
        matching_rules.sort_by(|a, b| cascade_cmp(a, b));

        // Apply declarations in order
        for rule in matching_rules {
            self.apply_declarations(&mut styles, &rule.declarations);
        }

        styles
    }

    /// Compute styles for an element using indexed rule lookup.
    /// Instead of checking all rules, only checks rules that could match this element
    /// based on its tag name and class names. This reduces matching from O(all_rules) to
    /// O(relevant_rules) per element, which is typically 5-20x fewer rules.
    pub fn compute_style_for_element(&self, element: &scraper::ElementRef) -> ComputedStyles {
        let mut styles = ComputedStyles::default();

        // Fast path: use indexed rule lookup
        if let Some(ref index) = self.rule_index {
            let mut matching_rules: Vec<(&ParsedRule, bool)> = Vec::new();
            let el = element.value();
            let tag_name = el.name().to_lowercase();

            // Collect candidate rule indices from the index
            let mut candidates = Vec::new();

            // Add rules indexed by this element's tag
            if let Some(tag_rules) = index.by_tag.get(&tag_name) {
                candidates.extend_from_slice(tag_rules);
            }

            // Add rules indexed by this element's ID
            if let Some(id_attr) = el.attr("id") {
                let id_lower = id_attr.to_lowercase();
                if let Some(id_rules) = index.by_id.get(&id_lower) {
                    candidates.extend_from_slice(id_rules);
                }
            }

            // Add rules indexed by this element's classes
            if let Some(class_attr) = el.attr("class") {
                for cls in class_attr.split_whitespace() {
                    let cls_lower = cls.to_lowercase();
                    if let Some(class_rules) = index.by_class.get(&cls_lower) {
                        candidates.extend_from_slice(class_rules);
                    }
                }
            }

            // Add universal rules (always checked)
            candidates.extend_from_slice(&index.universal);

            // Deduplicate candidate indices
            candidates.sort_unstable();
            candidates.dedup();

            // Only check candidate rules
            for &ci in &candidates {
                let compiled = &self.compiled_rules[ci];
                let rule = &self.rules[compiled.rule_index];

                let mut matched = false;
                for (i, compiled_sel) in compiled.compiled_selectors.iter().enumerate() {
                    if let Some(sel) = compiled_sel
                        && sel.matches(element)
                    {
                        matched = true;
                        break;
                    } else {
                        // Fallback: selector failed to compile, try pseudo-class fallback
                        let raw_selectors: Vec<&str> =
                            rule.selector.split(',').map(|s| s.trim()).collect();
                        if let Some(raw_sel) = raw_selectors.get(i)
                            && Self::matches_pseudo_class_fallback(raw_sel, &tag_name, el)
                        {
                            matched = true;
                            break;
                        }
                    }
                }

                if matched {
                    matching_rules.push((rule, false));
                }
            }

            // Sort by specificity then source order
            matching_rules.sort_by(|a, b| cascade_cmp(a.0, b.0));

            for (rule, _) in &matching_rules {
                self.apply_declarations(&mut styles, &rule.declarations);
            }

            return styles;
        }

        // Slow path: no pre-compiled selectors, parse at runtime (legacy)
        let mut matching_rules: Vec<(&ParsedRule, bool)> = Vec::new();
        let el = element.value();
        let tag_name = el.name().to_lowercase();

        for rule in &self.rules {
            for raw_selector in rule.selector.split(',').map(|s| s.trim()) {
                if raw_selector.is_empty() {
                    continue;
                }
                // Strip always-true negations for static rendering
                let preprocessed_slow = raw_selector
                    .replace(":not(:focus-visible)", "")
                    .replace(":not(:focus-within)", "")
                    .replace(":not(:focus)", "")
                    .replace(":not(:hover)", "")
                    .replace(":not(:active)", "");
                let sel_slow = if preprocessed_slow.trim().is_empty() {
                    raw_selector.to_string()
                } else {
                    preprocessed_slow
                };
                let sel_slow = Self::preprocess_modern_pseudos(&sel_slow);
                let sel_slow = sel_slow.as_str();
                if let Ok(parsed_selector) = scraper::Selector::parse(sel_slow)
                    && parsed_selector.matches(element)
                {
                    matching_rules.push((rule, false));
                    break;
                } else if Self::matches_pseudo_class_fallback(raw_selector, &tag_name, el) {
                    matching_rules.push((rule, false));
                    break;
                }
            }
        }

        matching_rules.sort_by(|a, b| cascade_cmp(a.0, b.0));

        for (rule, _) in &matching_rules {
            self.apply_declarations(&mut styles, &rule.declarations);
        }

        styles
    }

    /// Collect CSS custom property definitions (--name: value) from rules matching this element.
    /// Used for element-scoped variable resolution: CSS custom properties are inherited and
    /// can be scoped to specific elements via attribute selectors like [data-color-mode="dark"].
    /// Returns only rules that define `--` custom properties, sorted by cascade order.
    pub fn collect_custom_properties_for_element(
        &self,
        element: &scraper::ElementRef,
    ) -> HashMap<String, String> {
        let mut vars: HashMap<String, String> = HashMap::new();

        if let Some(ref index) = self.rule_index {
            let el = element.value();
            let tag_name = el.name().to_lowercase();

            let mut candidates = Vec::new();
            if let Some(tag_rules) = index.by_tag.get(&tag_name) {
                candidates.extend_from_slice(tag_rules);
            }
            if let Some(id_attr) = el.attr("id") {
                let id_lower = id_attr.to_lowercase();
                if let Some(id_rules) = index.by_id.get(&id_lower) {
                    candidates.extend_from_slice(id_rules);
                }
            }
            if let Some(class_attr) = el.attr("class") {
                for cls in class_attr.split_whitespace() {
                    let cls_lower = cls.to_lowercase();
                    if let Some(class_rules) = index.by_class.get(&cls_lower) {
                        candidates.extend_from_slice(class_rules);
                    }
                }
            }
            candidates.extend_from_slice(&index.universal);
            candidates.sort_unstable();
            candidates.dedup();

            let mut matching_rules: Vec<&ParsedRule> = Vec::new();
            for &ci in &candidates {
                let compiled = &self.compiled_rules[ci];
                let rule = &self.rules[compiled.rule_index];

                // Skip rules with no custom property declarations — fast path
                if !rule.declarations.keys().any(|k| k.starts_with("--")) {
                    continue;
                }

                let mut matched = false;
                for (i, compiled_sel) in compiled.compiled_selectors.iter().enumerate() {
                    if let Some(sel) = compiled_sel
                        && sel.matches(element)
                    {
                        matched = true;
                        break;
                    } else {
                        let raw_selectors: Vec<&str> =
                            rule.selector.split(',').map(|s| s.trim()).collect();
                        if let Some(raw_sel) = raw_selectors.get(i)
                            && Self::matches_pseudo_class_fallback(raw_sel, &tag_name, el)
                        {
                            matched = true;
                            break;
                        }
                    }
                }

                if matched {
                    matching_rules.push(rule);
                }
            }

            // Apply in cascade order so higher-specificity rules win
            matching_rules.sort_by(|a, b| cascade_cmp(a, b));
            for rule in matching_rules {
                for (prop, val) in &rule.declarations {
                    if prop.starts_with("--") {
                        vars.insert(
                            prop.clone(),
                            val.trim_end_matches(" !important").to_string(),
                        );
                    }
                }
            }
        }

        vars
    }

    /// Compute hover-specific styles for an element using scraper's selector matching.
    ///
    /// This iterates all parsed rules looking for selectors that contain `:hover`.
    /// For each matching rule, we strip `:hover` from the selector and check if the
    /// base selector matches the element. Returns the accumulated hover-only declarations.
    pub fn compute_hover_style_for_element(&self, element: &scraper::ElementRef) -> ComputedStyles {
        let mut styles = ComputedStyles::default();

        // Fast path: use pre-compiled selectors
        if !self.compiled_rules.is_empty() {
            let mut matching_rules: Vec<&ParsedRule> = Vec::new();
            let el = element.value();
            let tag_name = el.name().to_lowercase();

            for compiled in &self.compiled_rules {
                // Skip rules that don't have :hover at all
                if !compiled.has_hover {
                    continue;
                }

                let rule = &self.rules[compiled.rule_index];
                let raw_selectors: Vec<&str> = rule.selector.split(',').map(|s| s.trim()).collect();
                let mut matched = false;

                for (i, raw_sel) in raw_selectors.iter().enumerate() {
                    if !Self::contains_hover_pseudo(raw_sel) {
                        continue;
                    }

                    // Use pre-compiled hover base selector
                    if let Some(Some(base_sel)) = compiled.hover_base_selectors.get(i)
                        && base_sel.matches(element)
                    {
                        matched = true;
                        break;
                    } else if let Some(None) = compiled.hover_base_selectors.get(i) {
                        // Base selector was empty (bare ":hover") or failed to compile
                        let base = Self::strip_hover_pseudo(raw_sel);
                        if base.is_empty() {
                            matched = true;
                            break;
                        }
                        // Fallback for failed compile
                        if Self::simple_selector_matches(&base, &tag_name, el) {
                            matched = true;
                            break;
                        }
                    }
                }

                if matched {
                    matching_rules.push(rule);
                }
            }

            matching_rules.sort_by(|a, b| cascade_cmp(a, b));

            for rule in &matching_rules {
                self.apply_declarations(&mut styles, &rule.declarations);
            }

            return styles;
        }

        // Slow path: no pre-compiled selectors (legacy)
        let mut matching_rules: Vec<&ParsedRule> = Vec::new();

        for rule in &self.rules {
            for raw_selector in rule.selector.split(',').map(|s| s.trim()) {
                if raw_selector.is_empty() {
                    continue;
                }
                if !Self::contains_hover_pseudo(raw_selector) {
                    continue;
                }
                let base_selector = Self::strip_hover_pseudo(raw_selector);
                if base_selector.is_empty() {
                    matching_rules.push(rule);
                    break;
                }
                if let Ok(parsed_selector) = scraper::Selector::parse(&base_selector)
                    && parsed_selector.matches(element)
                {
                    matching_rules.push(rule);
                    break;
                } else {
                    let el = element.value();
                    let tag_name = el.name().to_lowercase();
                    if Self::simple_selector_matches(&base_selector, &tag_name, el) {
                        matching_rules.push(rule);
                        break;
                    }
                }
            }
        }

        matching_rules.sort_by(|a, b| cascade_cmp(a, b));

        for rule in &matching_rules {
            self.apply_declarations(&mut styles, &rule.declarations);
        }

        styles
    }

    /// Check if a selector string contains the :hover pseudo-class.
    fn contains_hover_pseudo(selector: &str) -> bool {
        // Look for ":hover" that's either at the end or followed by non-alphanumeric
        let mut search_from = 0;
        while let Some(pos) = selector[search_from..].find(":hover") {
            let abs_pos = search_from + pos;
            let after = abs_pos + 6;
            // Make sure it's ":hover" and not ":hover-something"
            if after >= selector.len() || !selector.as_bytes()[after].is_ascii_alphanumeric() {
                // Also make sure it's not "::hover" (pseudo-element)
                if abs_pos == 0 || selector.as_bytes()[abs_pos - 1] != b':' {
                    return true;
                }
            }
            search_from = after;
        }
        false
    }

    /// Strip `:hover` from a selector, returning the base selector.
    /// e.g., "a:hover" → "a", ".nav-link:hover" → ".nav-link",
    /// "nav a:hover" → "nav a", ":hover" → ""
    fn strip_hover_pseudo(selector: &str) -> String {
        let mut result = selector.to_string();
        // Remove ":hover" occurrences (not "::hover")
        while let Some(pos) = result.find(":hover") {
            // Ensure it's not "::hover"
            if pos > 0 && result.as_bytes()[pos - 1] == b':' {
                break;
            }
            let after = pos + 6;
            // Make sure it's ":hover" and not ":hover-something"
            if after < result.len() && result.as_bytes()[after].is_ascii_alphanumeric() {
                break;
            }
            result = format!("{}{}", &result[..pos], &result[after..]);
        }
        result.trim().to_string()
    }

    /// Preprocess CSS selectors containing `:is()`, `:where()`, and `:has()`
    /// that scraper cannot parse natively.
    ///
    /// Strategy:
    /// - `:is(X)` / `:where(X)`: If the argument is a single simple selector,
    ///   replace the pseudo-function with just the argument (e.g., `div:is(.foo)` → `div.foo`).
    ///   If the argument is a selector list, extract the first alternative as a best-effort.
    /// - `:has(...)`: Strip entirely — the base selector still matches the element.
    ///   This is an over-match (less specific), but far better than not matching at all.
    fn preprocess_modern_pseudos(selector: &str) -> String {
        let mut result = selector.to_string();

        // Process :has(...) — strip entirely (best-effort: match base selector)
        loop {
            if let Some(start) = result.find(":has(")
                && let Some(end) = Self::find_matching_paren(&result, start + 4)
            {
                result = format!("{}{}", &result[..start], &result[end + 1..]);
                continue;
            }
            break;
        }

        // Process :is(...) — replace with the contained selector
        loop {
            if let Some(start) = result.find(":is(")
                && let Some(end) = Self::find_matching_paren(&result, start + 3)
            {
                let inner = &result[start + 4..end].trim().to_string();
                // Use the first selector alternative from the list
                let first_alt = inner.split(',').next().unwrap_or("").trim();
                // If the first alt looks like a simple selector (class, tag, id),
                // substitute it directly
                if !first_alt.is_empty() {
                    result = format!("{}{}{}", &result[..start], first_alt, &result[end + 1..]);
                    continue;
                } else {
                    // Empty :is() — remove it
                    result = format!("{}{}", &result[..start], &result[end + 1..]);
                    continue;
                }
            }
            break;
        }

        // Process :where(...) — same as :is() but zero specificity (we don't track that
        // difference yet, but at least the rules won't be dropped)
        loop {
            if let Some(start) = result.find(":where(")
                && let Some(end) = Self::find_matching_paren(&result, start + 6)
            {
                let inner = &result[start + 7..end].trim().to_string();
                let first_alt = inner.split(',').next().unwrap_or("").trim();
                if !first_alt.is_empty() {
                    result = format!("{}{}{}", &result[..start], first_alt, &result[end + 1..]);
                    continue;
                } else {
                    result = format!("{}{}", &result[..start], &result[end + 1..]);
                    continue;
                }
            }
            break;
        }

        let trimmed = result.trim().to_string();
        if trimmed.is_empty() {
            selector.to_string()
        } else {
            trimmed
        }
    }

    /// Find the index of the closing parenthesis that matches the opening paren at `open_pos`.
    fn find_matching_paren(s: &str, open_pos: usize) -> Option<usize> {
        let bytes = s.as_bytes();
        if open_pos >= bytes.len() || bytes[open_pos] != b'(' {
            return None;
        }
        let mut depth = 1;
        for (i, &byte) in bytes.iter().enumerate().skip(open_pos + 1) {
            match byte {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    // ── Section 3: Specificity & legacy matching ────────────────────────

    /// Calculate selector specificity (simplified)
    pub(crate) fn calculate_specificity(&self, selector: &str) -> (u32, u32, u32) {
        let mut elements = 0u32;

        // Count IDs (#)
        let ids = selector.matches('#').count() as u32;

        // Count classes (.), attributes ([]), and pseudo-classes (:not ::)
        let mut classes = selector.matches('.').count() as u32;
        classes += selector.matches('[').count() as u32;
        // Count single colons not followed by another colon (pseudo-classes)
        for (i, c) in selector.char_indices() {
            if c == ':' {
                let next = selector.chars().nth(i + 1);
                if next != Some(':') {
                    classes += 1;
                }
            }
        }

        // Count element selectors (rough approximation)
        let parts: Vec<&str> = selector
            .split(|c: char| c.is_whitespace() || c == '>' || c == '+' || c == '~')
            .filter(|s| !s.is_empty())
            .collect();

        for part in parts {
            // Skip if starts with # or .
            if !part.starts_with('#')
                && !part.starts_with('.')
                && !part.starts_with('[')
                && !part.starts_with(':')
            {
                // It's an element selector
                let elem_part = part.split(['#', '.', '[', ':']).next().unwrap_or("");
                if !elem_part.is_empty() && elem_part != "*" {
                    elements += 1;
                }
            }
        }

        (ids, classes, elements)
    }

    /// Check if two selectors match (simplified)
    fn selectors_match(&self, rule_selector: &str, target_selector: &str) -> bool {
        // Simple exact match for now
        rule_selector == target_selector
    }

    /// Fallback matching for selectors containing pseudo-classes that scraper can't parse.
    /// Handles :link, :visited (match <a> with href), and strips other pseudo-classes
    /// to attempt base selector matching.
    fn matches_pseudo_class_fallback(
        raw_selector: &str,
        tag_name: &str,
        el: &scraper::node::Element,
    ) -> bool {
        // Extract the pseudo-class and base selector
        // Handle selectors like "a:link", ".class:hover", "div a:visited"
        // We find the last pseudo-class in the selector
        if let Some(colon_idx) = raw_selector.rfind(':') {
            let pseudo_part = &raw_selector[colon_idx + 1..];
            let base = raw_selector[..colon_idx].trim();

            // Extract just the pseudo-class name (before any parentheses)
            let pseudo_name = pseudo_part.split('(').next().unwrap_or("").trim();

            match pseudo_name {
                "link" | "visited" | "any-link" => {
                    // :link / :visited / :any-link apply to <a> elements with href
                    if tag_name != "a" || el.attr("href").is_none() {
                        return false;
                    }
                    // Match the base selector (e.g., "a" from "a:link")
                    if base.is_empty() {
                        return true;
                    }
                    // Try to parse and match the base selector
                    // For simple selectors like "a", check tag match
                    Self::simple_selector_matches(base, tag_name, el)
                }
                "hover" | "active" | "focus" | "focus-visible" | "focus-within" => {
                    // Interactive pseudo-classes don't apply during static rendering
                    false
                }
                "target" => {
                    // :target matches the element whose ID equals the URL fragment.
                    // In headless mode we don't have a URL fragment, so never matches.
                    false
                }
                "lang" => {
                    // :lang(xx) matches elements with a lang attribute matching xx
                    let lang_arg = pseudo_part
                        .strip_prefix("lang(")
                        .and_then(|s| s.strip_suffix(')'))
                        .unwrap_or("")
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'');
                    if !lang_arg.is_empty() {
                        // Check the element and ancestors for lang attribute
                        if let Some(el_lang) = el.attr("lang").or_else(|| el.attr("xml:lang")) {
                            let matches_lang = el_lang == lang_arg
                                || el_lang.starts_with(&format!("{}-", lang_arg));
                            if matches_lang {
                                return if base.is_empty() {
                                    true
                                } else {
                                    Self::simple_selector_matches(base, tag_name, el)
                                };
                            }
                        }
                    }
                    false
                }
                "empty" => {
                    // :empty matches elements with no children (no child elements or text nodes)
                    // In our scraper-based model, we can't check children directly
                    // from the Element alone, so match base and let scraper handle it
                    // via the preprocessed selector path. As a fallback, assume false.
                    false
                }
                "enabled" => {
                    // :enabled matches form elements that are not disabled
                    let is_form_el = matches!(
                        tag_name,
                        "input" | "select" | "textarea" | "button" | "fieldset"
                    );
                    if is_form_el && el.attr("disabled").is_none() {
                        if base.is_empty() {
                            return true;
                        }
                        return Self::simple_selector_matches(base, tag_name, el);
                    }
                    false
                }
                "disabled" => {
                    // :disabled matches form elements with disabled attribute
                    if el.attr("disabled").is_some() {
                        if base.is_empty() {
                            return true;
                        }
                        return Self::simple_selector_matches(base, tag_name, el);
                    }
                    false
                }
                "checked" => {
                    // :checked matches checked checkboxes/radio buttons and selected options
                    let is_checked = el.attr("checked").is_some() || el.attr("selected").is_some();
                    if is_checked {
                        if base.is_empty() {
                            return true;
                        }
                        return Self::simple_selector_matches(base, tag_name, el);
                    }
                    false
                }
                "required" => {
                    if el.attr("required").is_some() {
                        if base.is_empty() {
                            return true;
                        }
                        return Self::simple_selector_matches(base, tag_name, el);
                    }
                    false
                }
                "optional" => {
                    let is_form_el = matches!(tag_name, "input" | "select" | "textarea");
                    if is_form_el && el.attr("required").is_none() {
                        if base.is_empty() {
                            return true;
                        }
                        return Self::simple_selector_matches(base, tag_name, el);
                    }
                    false
                }
                "read-only" => {
                    let is_readonly = el.attr("readonly").is_some()
                        || el.attr("disabled").is_some()
                        || !matches!(tag_name, "input" | "textarea");
                    if is_readonly {
                        if base.is_empty() {
                            return true;
                        }
                        return Self::simple_selector_matches(base, tag_name, el);
                    }
                    false
                }
                "read-write" => {
                    let is_editable = matches!(tag_name, "input" | "textarea")
                        && el.attr("readonly").is_none()
                        && el.attr("disabled").is_none();
                    if is_editable {
                        if base.is_empty() {
                            return true;
                        }
                        return Self::simple_selector_matches(base, tag_name, el);
                    }
                    false
                }
                "placeholder-shown" => {
                    // :placeholder-shown matches inputs with placeholder visible (value is empty)
                    // In static rendering, assume placeholder is shown if element has placeholder attr
                    if el.attr("placeholder").is_some() {
                        if base.is_empty() {
                            return true;
                        }
                        return Self::simple_selector_matches(base, tag_name, el);
                    }
                    false
                }
                "first-child" | "last-child" | "only-child" | "first-of-type" | "last-of-type"
                | "only-of-type" => {
                    // Structural pseudo-classes — these should ideally be handled by
                    // scraper's selector matching. If we got here, scraper couldn't
                    // parse the full selector. Try matching just the base.
                    if base.is_empty() {
                        return false;
                    }
                    Self::simple_selector_matches(base, tag_name, el)
                }
                "root" => {
                    // :root matches the document element (html)
                    tag_name == "html"
                }
                // Pseudo-elements create virtual elements inside the target — their styles
                // must NOT apply to the element itself. Both CSS2 (single colon) and CSS3
                // (double colon) syntax end up here because rfind(':') strips the last colon.
                "before"
                | "after"
                | "first-letter"
                | "first-line"
                | "placeholder"
                | "selection"
                | "marker"
                | "backdrop"
                | "cue"
                | "grammar-error"
                | "spelling-error"
                | "target-text"
                | "file-selector-button"
                | ":before"
                | ":after"
                | ":first-letter"
                | ":first-line"
                | ":placeholder"
                | ":selection"
                | ":marker" => false,
                _ => {
                    // Unknown pseudo-class — try stripping it and matching base
                    if base.is_empty() {
                        return false;
                    }
                    Self::simple_selector_matches(base, tag_name, el)
                }
            }
        } else {
            false
        }
    }

    /// Simple selector matching for fallback pseudo-class handling.
    /// Handles tag selectors, class selectors, and ID selectors.
    fn simple_selector_matches(
        selector: &str,
        tag_name: &str,
        el: &scraper::node::Element,
    ) -> bool {
        // Try scraper first (it handles complex selectors)
        if let Ok(parsed) = scraper::Selector::parse(selector) {
            // We can't use parsed.matches() without an ElementRef, so fall back to manual
            // For now, do simple matching
        }

        let selector = selector.trim();

        // Simple tag match: "a", "div", etc.
        if selector == tag_name {
            return true;
        }

        // Class-based match: ".classname" or "tag.classname"
        if selector.contains('.') {
            let classes_attr = el.attr("class").unwrap_or("");
            let el_classes: Vec<&str> = classes_attr.split_whitespace().collect();

            // Split selector into tag and classes
            let parts: Vec<&str> = selector.split('.').collect();
            let sel_tag = parts[0]; // May be empty for ".classname"

            // Check tag match (empty means any tag)
            if !sel_tag.is_empty() && sel_tag != tag_name {
                return false;
            }

            // Check all required classes
            for cls in &parts[1..] {
                if !cls.is_empty() && !el_classes.contains(cls) {
                    return false;
                }
            }
            return true;
        }

        // ID-based match: "#id" or "tag#id"
        if selector.contains('#') {
            let parts: Vec<&str> = selector.splitn(2, '#').collect();
            let sel_tag = parts[0];
            let sel_id = parts.get(1).unwrap_or(&"");

            if !sel_tag.is_empty() && sel_tag != tag_name {
                return false;
            }
            if let Some(el_id) = el.attr("id") {
                return el_id == *sel_id;
            }
            return false;
        }

        false
    }

    /// Strip pseudo-classes and pseudo-elements from a selector for matching.
    /// e.g. "a:link" → "a", "a:hover::after" → "a"
    fn strip_pseudos(selector: &str) -> &str {
        // Find the first ':' that isn't escaped
        if let Some(idx) = selector.find(':') {
            &selector[..idx]
        } else {
            selector
        }
    }

    /// Check if a rule selector applies to a target element (legacy string-based matching)
    fn selector_applies(&self, rule_selector: &str, target: &str) -> bool {
        // Split rule selector by comma for multiple selectors
        for raw_selector in rule_selector.split(',').map(|s| s.trim()) {
            // Strip pseudo-classes (:link, :visited, :hover, ::before, etc.)
            let selector = Self::strip_pseudos(raw_selector);
            if selector.is_empty() {
                continue;
            }

            // Check element type match
            if selector == target {
                return true;
            }

            // Check class match (e.g., ".container" matches "div.container")
            if selector.starts_with('.') && target.contains(selector) {
                return true;
            }

            // Check ID match
            if selector.starts_with('#') && target.contains(selector) {
                return true;
            }

            // Check element match (e.g., "div" matches "div.container")
            let elem = target.split(['.', '#', '[']).next().unwrap_or("");
            if selector == elem {
                return true;
            }
        }

        false
    }
}
