use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Computed CSS styles for an element
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComputedStyles {
    /// Display property (block, flex, grid, inline, etc.)
    pub display: Option<String>,
    /// Position property (static, relative, absolute, fixed, sticky)
    pub position: Option<String>,
    /// Width (px, %, auto, etc.)
    pub width: Option<String>,
    /// Height
    pub height: Option<String>,
    /// Margin (top, right, bottom, left)
    pub margin: Option<BoxModel>,
    /// Padding
    pub padding: Option<BoxModel>,
    /// Border
    pub border: Option<BorderStyles>,
    /// Background color
    pub background_color: Option<String>,
    /// Text color
    pub color: Option<String>,
    /// Font size
    pub font_size: Option<String>,
    /// Font family
    pub font_family: Option<String>,
    /// Font weight
    pub font_weight: Option<String>,
    /// Flex direction
    pub flex_direction: Option<String>,
    /// Justify content
    pub justify_content: Option<String>,
    /// Align items
    pub align_items: Option<String>,
    /// Gap
    pub gap: Option<String>,
    /// Overflow
    pub overflow: Option<String>,
    /// Z-index
    pub z_index: Option<i32>,
    /// Opacity
    pub opacity: Option<f32>,
    /// Visibility
    pub visibility: Option<String>,

    // --- Promoted from `other` HashMap for performance (direct field access, no hashing) ---
    /// Flex wrap: nowrap, wrap, wrap-reverse
    pub flex_wrap: Option<String>,
    /// Align self
    pub align_self: Option<String>,
    /// Flex grow factor
    pub flex_grow: Option<String>,
    /// Flex shrink factor
    pub flex_shrink: Option<String>,
    /// Flex basis
    pub flex_basis: Option<String>,
    /// Min width
    pub min_width: Option<String>,
    /// Min height
    pub min_height: Option<String>,
    /// Max width
    pub max_width: Option<String>,
    /// Max height
    pub max_height: Option<String>,
    /// Font style (normal, italic, oblique)
    pub font_style: Option<String>,
    /// Line height
    pub line_height: Option<String>,
    /// Text alignment
    pub text_align: Option<String>,
    /// Text decoration
    pub text_decoration: Option<String>,
    /// Text transform
    pub text_transform: Option<String>,
    /// White space handling
    pub white_space: Option<String>,
    /// Letter spacing
    pub letter_spacing: Option<String>,
    /// Word spacing
    pub word_spacing: Option<String>,
    /// Border radius
    pub border_radius: Option<String>,
    /// List style type
    pub list_style_type: Option<String>,
    /// Cursor style
    pub cursor: Option<String>,
    /// Grid template columns (e.g., "1fr 3fr", "200px 1fr 200px", "12.25rem minmax(0,1fr)")
    pub grid_template_columns: Option<String>,
    /// Grid template rows (e.g., "min-content 1fr min-content")
    pub grid_template_rows: Option<String>,
    /// Grid template areas (e.g., "'header header' 'sidebar content' 'footer footer'")
    pub grid_template_areas: Option<String>,
    /// Grid area name for child placement (e.g., "pageContent", "sidebar")
    pub grid_area: Option<String>,
    /// Grid auto flow (e.g., "row", "column", "row dense")
    pub grid_auto_flow: Option<String>,
    /// Grid auto rows (e.g., "min-content", "1fr")
    pub grid_auto_rows: Option<String>,
    /// Grid auto columns (e.g., "min-content", "1fr")
    pub grid_auto_columns: Option<String>,
    /// Grid column placement (e.g., "1 / 3", "span 2")
    pub grid_column: Option<String>,
    /// Grid row placement (e.g., "1 / 3", "span 2")
    pub grid_row: Option<String>,

    /// Per-side border overrides (from border-top, border-right, border-bottom, border-left)
    /// These take precedence over the `border` shorthand for the respective side.
    pub border_top: Option<BorderStyles>,
    pub border_right: Option<BorderStyles>,
    pub border_bottom: Option<BorderStyles>,
    pub border_left: Option<BorderStyles>,

    /// Float property (left, right, none)
    pub float: Option<String>,
    /// Clear property (left, right, both, none)
    pub clear: Option<String>,

    // --- Visual transform/effect properties (promoted for getComputedStyle support) ---
    /// CSS transform (e.g., "rotate(45deg)", "translateX(10px)")
    pub transform: Option<String>,
    /// CSS transform-origin
    pub transform_origin: Option<String>,
    /// CSS filter (e.g., "blur(5px)", "brightness(0.8)")
    pub filter: Option<String>,
    /// CSS backdrop-filter
    pub backdrop_filter: Option<String>,
    /// CSS animation shorthand
    pub animation: Option<String>,
    /// CSS animation-name
    pub animation_name: Option<String>,
    /// CSS animation-duration
    pub animation_duration: Option<String>,
    /// CSS transition shorthand
    pub transition: Option<String>,
    /// CSS clip-path
    pub clip_path: Option<String>,
    /// CSS mask / -webkit-mask
    pub mask: Option<String>,
    /// CSS mix-blend-mode
    pub mix_blend_mode: Option<String>,
    /// CSS object-fit (for images/video)
    pub object_fit: Option<String>,
    /// CSS object-position
    pub object_position: Option<String>,
    /// CSS box-shadow
    pub box_shadow: Option<String>,
    /// CSS text-shadow
    pub text_shadow: Option<String>,
    /// CSS outline
    pub outline: Option<String>,
    /// CSS overflow-x
    pub overflow_x: Option<String>,
    /// CSS overflow-y
    pub overflow_y: Option<String>,
    /// CSS text-overflow
    pub text_overflow: Option<String>,
    /// CSS word-break
    pub word_break: Option<String>,
    /// CSS overflow-wrap / word-wrap
    pub overflow_wrap: Option<String>,
    /// CSS vertical-align
    pub vertical_align: Option<String>,
    /// CSS content (for ::before/::after)
    pub content: Option<String>,
    /// CSS pointer-events
    pub pointer_events: Option<String>,
    /// CSS user-select
    pub user_select: Option<String>,
    /// CSS appearance / -webkit-appearance
    pub appearance: Option<String>,
    /// CSS will-change
    pub will_change: Option<String>,
    /// CSS contain
    pub contain: Option<String>,
    /// CSS container-type
    pub container_type: Option<String>,
    /// CSS aspect-ratio
    pub aspect_ratio: Option<String>,
    /// CSS justify-self
    pub justify_self: Option<String>,
    /// CSS place-items
    pub place_items: Option<String>,
    /// CSS place-content
    pub place_content: Option<String>,
    /// CSS gap (alias: column-gap, row-gap handled separately)
    pub row_gap: Option<String>,
    /// CSS column-count
    pub column_count: Option<String>,
    /// CSS direction (ltr, rtl)
    pub direction: Option<String>,
    /// CSS writing-mode
    pub writing_mode: Option<String>,
    /// CSS counter-reset (e.g., "section 0", "item")
    pub counter_reset: Option<String>,
    /// CSS counter-increment (e.g., "section", "item 2")
    pub counter_increment: Option<String>,

    /// All other properties as key-value pairs
    #[serde(flatten)]
    pub other: HashMap<String, String>,
}

/// Box model values (margin, padding)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BoxModel {
    pub top: String,
    pub right: String,
    pub bottom: String,
    pub left: String,
}

/// Border styles
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BorderStyles {
    pub width: String,
    pub style: String,
    pub color: String,
}

/// A parsed CSS rule with selector and declarations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRule {
    /// CSS selector string
    pub selector: String,
    /// Specificity (a, b, c) tuple
    pub specificity: (u32, u32, u32),
    /// Declarations (property: value)
    pub declarations: HashMap<String, String>,
    /// Source order index for tie-breaking specificity
    #[serde(default)]
    pub source_order: usize,
    /// Cascade layer index (None = unlayered, higher index = later-declared layer)
    #[serde(default)]
    pub layer: Option<usize>,
}

/// A pre-compiled CSS rule: selector already parsed into scraper::Selector for fast matching.
/// Rules whose selectors fail to parse are stored with compiled_selector = None and fall back
/// to the slow path (pseudo-class fallback matching).
pub struct CompiledRule {
    /// Index into CssProcessor::rules
    pub rule_index: usize,
    /// Pre-compiled selectors (one per comma-separated selector alternative).
    /// None entries mean scraper couldn't parse that selector alternative.
    pub compiled_selectors: Vec<Option<scraper::Selector>>,
    /// Whether any selector alternative contains :hover (for fast skip in non-hover path)
    pub has_hover: bool,
    /// For hover rules: pre-compiled base selectors with :hover stripped
    pub hover_base_selectors: Vec<Option<scraper::Selector>>,
    /// Key tag names extracted from each selector alternative's rightmost simple selector.
    /// Used for fast pre-filtering: if the element's tag doesn't match any key_tag, skip .matches().
    /// None means the selector has no tag constraint (class-only, universal, etc.) — must always check.
    pub key_tags: Vec<Option<String>>,
    /// Key class name extracted from the rightmost simple selector.
    /// Used with key_tags for indexed rule lookup.
    pub key_classes: Vec<Option<String>>,
    /// Key ID extracted from the rightmost simple selector.
    /// Used with key_tags/key_classes for indexed rule lookup.
    pub key_ids: Vec<Option<String>>,
}

/// Indexed rule lookup for fast CSS matching.
/// Instead of iterating all rules per element, look up candidate rules by the element's
/// tag name, class names, and ID, then only run .matches() on those candidates.
pub struct RuleIndex {
    /// compiled_rule index → for rules whose key selector targets a specific tag
    pub by_tag: HashMap<String, Vec<usize>>,
    /// compiled_rule index → for rules whose key selector targets a specific class
    pub by_class: HashMap<String, Vec<usize>>,
    /// compiled_rule index → for rules whose key selector targets a specific ID
    pub by_id: HashMap<String, Vec<usize>>,
    /// compiled_rule indices for rules that could match any element (universal, complex selectors)
    pub universal: Vec<usize>,
}

/// CSS processor for handling CSS parsing, computation, and optimization
pub struct CssProcessor {
    /// Parsed rules from all stylesheets
    pub(crate) rules: Vec<ParsedRule>,
    /// Raw stylesheet sources for reference
    pub(crate) sources: Vec<String>,
    /// CSS custom properties (--name: value) collected from :root and other selectors
    pub(crate) custom_properties: HashMap<String, String>,
    /// Viewport width for media query evaluation
    pub(crate) viewport_width: f32,
    /// Viewport height for container query evaluation
    pub(crate) viewport_height: f32,
    /// Classes on the <html> element (for scoping custom property selectors)
    pub(crate) html_classes: Vec<String>,
    /// Source order counter
    pub(crate) source_order_counter: usize,
    /// Pre-compiled selectors for fast matching (populated by compile_selectors())
    pub(crate) compiled_rules: Vec<CompiledRule>,
    /// Indexed rule lookup for O(1) candidate selection per element
    pub(crate) rule_index: Option<RuleIndex>,
    /// Known @keyframes: name → serialized CSS declarations per stop (e.g., "from", "to", "50%")
    pub(crate) keyframes: HashMap<String, Vec<(String, HashMap<String, String>)>>,
    /// Known @font-face declarations: font-family → src URL(s) and descriptors
    pub(crate) font_faces: Vec<FontFaceEntry>,
    /// Cascade layer declaration order: layer name → order index
    pub(crate) layer_order: HashMap<String, usize>,
    /// Counter for assigning layer order indices
    pub(crate) layer_order_counter: usize,
    /// Currently active layer name during parsing (None = unlayered)
    pub(crate) current_layer: Option<String>,
}

/// A parsed @font-face rule entry
#[derive(Debug, Clone)]
pub struct FontFaceEntry {
    /// The font-family name
    pub family: String,
    /// The src value (url, format)
    pub src: String,
    /// Font weight (e.g., "400", "bold", "100 900")
    pub weight: Option<String>,
    /// Font style (e.g., "normal", "italic")
    pub style: Option<String>,
    /// Font display (e.g., "swap", "block", "fallback")
    pub display: Option<String>,
    /// Unicode range (e.g., "U+0000-00FF")
    pub unicode_range: Option<String>,
}
