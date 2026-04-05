use super::types::{BorderStyles, BoxModel, ComputedStyles, CssProcessor};
use std::collections::HashMap;

impl CssProcessor {
    /// Apply declarations to computed styles, resolving var() references
    pub(crate) fn apply_declarations(
        &self,
        styles: &mut ComputedStyles,
        declarations: &HashMap<String, String>,
    ) {
        for (prop, value) in declarations {
            // Skip custom properties (--*) — they're already collected
            if prop.starts_with("--") {
                continue;
            }

            // Remove !important suffix for storage
            let raw_value = value.trim_end_matches(" !important").to_string();
            // Resolve var() references
            let clean_value = self.resolve_var(&raw_value);

            match prop.as_str() {
                "display" => styles.display = Some(clean_value),
                "position" => styles.position = Some(clean_value),
                "width" => styles.width = Some(clean_value),
                "height" => styles.height = Some(clean_value),
                "background-color" => styles.background_color = Some(clean_value),
                "background" => {
                    // The background shorthand can include color, image, position, etc.
                    // Extract the color component from the shorthand.
                    if let Some(color) = Self::extract_color_from_background(&clean_value) {
                        styles.background_color = Some(color);
                    } else {
                        styles.other.insert("background".to_string(), clean_value);
                    }
                }
                "color" => styles.color = Some(clean_value),
                "font-size" => styles.font_size = Some(clean_value),
                "font-family" => styles.font_family = Some(clean_value),
                "font-weight" => styles.font_weight = Some(clean_value),
                "flex-direction" => styles.flex_direction = Some(clean_value),
                "justify-content" => styles.justify_content = Some(clean_value),
                "align-items" => styles.align_items = Some(clean_value),
                "gap" => styles.gap = Some(clean_value),
                "overflow" => styles.overflow = Some(clean_value),
                "visibility" => styles.visibility = Some(clean_value),
                "opacity" => {
                    if let Ok(val) = clean_value.parse::<f32>() {
                        styles.opacity = Some(val);
                    }
                }
                "z-index" => {
                    if let Ok(val) = clean_value.parse::<i32>() {
                        styles.z_index = Some(val);
                    }
                }
                "margin" => {
                    styles.margin = Some(self.parse_box_model(&clean_value));
                }
                "margin-top" => {
                    let mut margin = styles.margin.clone().unwrap_or_default();
                    margin.top = clean_value;
                    styles.margin = Some(margin);
                }
                "margin-right" => {
                    let mut margin = styles.margin.clone().unwrap_or_default();
                    margin.right = clean_value;
                    styles.margin = Some(margin);
                }
                "margin-bottom" => {
                    let mut margin = styles.margin.clone().unwrap_or_default();
                    margin.bottom = clean_value;
                    styles.margin = Some(margin);
                }
                "margin-left" => {
                    let mut margin = styles.margin.clone().unwrap_or_default();
                    margin.left = clean_value;
                    styles.margin = Some(margin);
                }
                "padding" => {
                    styles.padding = Some(self.parse_box_model(&clean_value));
                }
                "padding-top" => {
                    let mut padding = styles.padding.clone().unwrap_or_default();
                    padding.top = clean_value;
                    styles.padding = Some(padding);
                }
                "padding-right" => {
                    let mut padding = styles.padding.clone().unwrap_or_default();
                    padding.right = clean_value;
                    styles.padding = Some(padding);
                }
                "padding-bottom" => {
                    let mut padding = styles.padding.clone().unwrap_or_default();
                    padding.bottom = clean_value;
                    styles.padding = Some(padding);
                }
                "padding-left" => {
                    let mut padding = styles.padding.clone().unwrap_or_default();
                    padding.left = clean_value;
                    styles.padding = Some(padding);
                }
                // Promoted properties (previously in `other` HashMap)
                "flex-wrap" => styles.flex_wrap = Some(clean_value),
                "align-self" => styles.align_self = Some(clean_value),
                "flex-grow" => styles.flex_grow = Some(clean_value),
                "flex-shrink" => styles.flex_shrink = Some(clean_value),
                "flex-basis" => styles.flex_basis = Some(clean_value),
                "flex" => {
                    // flex shorthand: <grow> [<shrink>] [<basis>]
                    let parts: Vec<&str> = clean_value.split_whitespace().collect();
                    match parts.len() {
                        1 if parts[0] == "none" => {
                            styles.flex_grow = Some("0".to_string());
                            styles.flex_shrink = Some("0".to_string());
                            styles.flex_basis = Some("auto".to_string());
                        }
                        1 if parts[0] == "auto" => {
                            styles.flex_grow = Some("1".to_string());
                            styles.flex_shrink = Some("1".to_string());
                            styles.flex_basis = Some("auto".to_string());
                        }
                        1 => {
                            styles.flex_grow = Some(parts[0].to_string());
                            styles.flex_shrink = Some("1".to_string());
                            styles.flex_basis = Some("0%".to_string());
                        }
                        2 => {
                            styles.flex_grow = Some(parts[0].to_string());
                            styles.flex_shrink = Some(parts[1].to_string());
                            styles.flex_basis = Some("0%".to_string());
                        }
                        _ => {
                            styles.flex_grow = Some(parts[0].to_string());
                            styles.flex_shrink = Some(parts[1].to_string());
                            styles.flex_basis = Some(parts[2].to_string());
                        }
                    }
                }
                "min-width" => styles.min_width = Some(clean_value),
                "min-height" => styles.min_height = Some(clean_value),
                "max-width" => styles.max_width = Some(clean_value),
                "max-height" => styles.max_height = Some(clean_value),
                "font-style" => styles.font_style = Some(clean_value),
                "line-height" => styles.line_height = Some(clean_value),
                "text-align" => styles.text_align = Some(clean_value),
                "text-decoration" => styles.text_decoration = Some(clean_value),
                "text-transform" => styles.text_transform = Some(clean_value),
                "white-space" => styles.white_space = Some(clean_value),
                "letter-spacing" => styles.letter_spacing = Some(clean_value),
                "word-spacing" => styles.word_spacing = Some(clean_value),
                "border-radius" => styles.border_radius = Some(clean_value),
                // Border shorthand: border: <width> <style> <color>
                "border" => {
                    if clean_value.trim() == "none" || clean_value.trim() == "0" {
                        styles.border = Some(BorderStyles {
                            width: "0".into(),
                            style: "none".into(),
                            color: String::new(),
                        });
                    } else {
                        styles.border = Some(Self::parse_border_shorthand(&clean_value));
                    }
                }
                // Per-side border shorthands
                "border-top" => {
                    if clean_value.trim() == "none" || clean_value.trim() == "0" {
                        styles.border_top = Some(BorderStyles {
                            width: "0".into(),
                            style: "none".into(),
                            color: String::new(),
                        });
                    } else {
                        styles.border_top = Some(Self::parse_border_shorthand(&clean_value));
                    }
                }
                "border-right" => {
                    if clean_value.trim() == "none" || clean_value.trim() == "0" {
                        styles.border_right = Some(BorderStyles {
                            width: "0".into(),
                            style: "none".into(),
                            color: String::new(),
                        });
                    } else {
                        styles.border_right = Some(Self::parse_border_shorthand(&clean_value));
                    }
                }
                "border-bottom" => {
                    if clean_value.trim() == "none" || clean_value.trim() == "0" {
                        styles.border_bottom = Some(BorderStyles {
                            width: "0".into(),
                            style: "none".into(),
                            color: String::new(),
                        });
                    } else {
                        styles.border_bottom = Some(Self::parse_border_shorthand(&clean_value));
                    }
                }
                "border-left" => {
                    if clean_value.trim() == "none" || clean_value.trim() == "0" {
                        styles.border_left = Some(BorderStyles {
                            width: "0".into(),
                            style: "none".into(),
                            color: String::new(),
                        });
                    } else {
                        styles.border_left = Some(Self::parse_border_shorthand(&clean_value));
                    }
                }
                // Individual longhand border properties
                "border-width" => {
                    let b = styles.border.get_or_insert_with(BorderStyles::default);
                    b.width = clean_value;
                }
                "border-style" => {
                    let b = styles.border.get_or_insert_with(BorderStyles::default);
                    b.style = clean_value;
                }
                "border-color" => {
                    let b = styles.border.get_or_insert_with(BorderStyles::default);
                    b.color = clean_value;
                }
                "border-top-width" => {
                    styles
                        .border_top
                        .get_or_insert_with(BorderStyles::default)
                        .width = clean_value;
                }
                "border-top-style" => {
                    styles
                        .border_top
                        .get_or_insert_with(BorderStyles::default)
                        .style = clean_value;
                }
                "border-top-color" => {
                    styles
                        .border_top
                        .get_or_insert_with(BorderStyles::default)
                        .color = clean_value;
                }
                "border-right-width" => {
                    styles
                        .border_right
                        .get_or_insert_with(BorderStyles::default)
                        .width = clean_value;
                }
                "border-right-style" => {
                    styles
                        .border_right
                        .get_or_insert_with(BorderStyles::default)
                        .style = clean_value;
                }
                "border-right-color" => {
                    styles
                        .border_right
                        .get_or_insert_with(BorderStyles::default)
                        .color = clean_value;
                }
                "border-bottom-width" => {
                    styles
                        .border_bottom
                        .get_or_insert_with(BorderStyles::default)
                        .width = clean_value;
                }
                "border-bottom-style" => {
                    styles
                        .border_bottom
                        .get_or_insert_with(BorderStyles::default)
                        .style = clean_value;
                }
                "border-bottom-color" => {
                    styles
                        .border_bottom
                        .get_or_insert_with(BorderStyles::default)
                        .color = clean_value;
                }
                "border-left-width" => {
                    styles
                        .border_left
                        .get_or_insert_with(BorderStyles::default)
                        .width = clean_value;
                }
                "border-left-style" => {
                    styles
                        .border_left
                        .get_or_insert_with(BorderStyles::default)
                        .style = clean_value;
                }
                "border-left-color" => {
                    styles
                        .border_left
                        .get_or_insert_with(BorderStyles::default)
                        .color = clean_value;
                }
                "list-style-type" => styles.list_style_type = Some(clean_value),
                "list-style" => {
                    // list-style shorthand
                    let v = clean_value.trim().to_string();
                    if v == "none" || v.starts_with("none ") || v.contains(" none") {
                        styles.list_style_type = Some("none".to_string());
                    } else {
                        for part in v.split_whitespace() {
                            match part {
                                "disc"
                                | "circle"
                                | "square"
                                | "decimal"
                                | "decimal-leading-zero"
                                | "lower-roman"
                                | "upper-roman"
                                | "lower-alpha"
                                | "upper-alpha"
                                | "lower-latin"
                                | "upper-latin"
                                | "lower-greek" => {
                                    styles.list_style_type = Some(part.to_string());
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                "cursor" => styles.cursor = Some(clean_value),
                "grid-template-columns" => styles.grid_template_columns = Some(clean_value),
                "grid-template-rows" => styles.grid_template_rows = Some(clean_value),
                "grid-template-areas" => styles.grid_template_areas = Some(clean_value),
                "grid-area" => styles.grid_area = Some(clean_value),
                "grid-auto-flow" => styles.grid_auto_flow = Some(clean_value),
                "grid-auto-rows" => styles.grid_auto_rows = Some(clean_value),
                "grid-auto-columns" => styles.grid_auto_columns = Some(clean_value),
                "grid-column" => styles.grid_column = Some(clean_value),
                "grid-row" => styles.grid_row = Some(clean_value),
                "grid-column-start" | "grid-column-end" => {
                    let existing = styles.grid_column.clone().unwrap_or_default();
                    if prop.as_str() == "grid-column-start" {
                        if let Some(end) = existing.split('/').nth(1) {
                            styles.grid_column = Some(format!("{} / {}", clean_value, end.trim()));
                        } else {
                            styles.grid_column = Some(clean_value);
                        }
                    } else {
                        let start = existing
                            .split('/')
                            .next()
                            .unwrap_or("auto")
                            .trim()
                            .to_string();
                        styles.grid_column = Some(format!("{} / {}", start, clean_value));
                    }
                }
                "grid-row-start" | "grid-row-end" => {
                    let existing = styles.grid_row.clone().unwrap_or_default();
                    if prop.as_str() == "grid-row-start" {
                        if let Some(end) = existing.split('/').nth(1) {
                            styles.grid_row = Some(format!("{} / {}", clean_value, end.trim()));
                        } else {
                            styles.grid_row = Some(clean_value);
                        }
                    } else {
                        let start = existing
                            .split('/')
                            .next()
                            .unwrap_or("auto")
                            .trim()
                            .to_string();
                        styles.grid_row = Some(format!("{} / {}", start, clean_value));
                    }
                }
                "grid-template" => {
                    // grid-template shorthand: <rows> / <columns>
                    // May also include area names: 'name name' <row-size> / <columns>
                    // Extract the columns portion (everything after the last '/')
                    if let Some(slash_pos) = clean_value.rfind('/') {
                        let columns = clean_value[slash_pos + 1..].trim().to_string();
                        if !columns.is_empty() {
                            styles.grid_template_columns = Some(columns);
                        }
                        let rows_part = clean_value[..slash_pos].trim().to_string();
                        if !rows_part.is_empty() {
                            // The rows part may contain area names like 'name name' <size>
                            // Extract area names (single-quoted strings) and row sizes
                            let mut areas = Vec::new();
                            let mut rows = Vec::new();
                            // Tokenize: area names are in single or double quotes, row sizes are outside
                            let mut in_quote = false;
                            let mut quote_char = '\0';
                            let mut current_area = String::new();
                            let mut current_token = String::new();
                            for ch in rows_part.chars() {
                                if (ch == '\'' || ch == '"') && (!in_quote || ch == quote_char) {
                                    if in_quote {
                                        // End of area name
                                        areas.push(current_area.clone());
                                        current_area.clear();
                                        in_quote = false;
                                        quote_char = '\0';
                                    } else {
                                        // Start of area name — flush any accumulated row size token
                                        let token = current_token.trim().to_string();
                                        if !token.is_empty() {
                                            rows.push(token);
                                        }
                                        current_token.clear();
                                        in_quote = true;
                                        quote_char = ch;
                                    }
                                } else if in_quote {
                                    current_area.push(ch);
                                } else {
                                    current_token.push(ch);
                                }
                            }
                            // Flush any remaining row size token
                            let token = current_token.trim().to_string();
                            if !token.is_empty() {
                                rows.push(token);
                            }
                            if !areas.is_empty() {
                                // Format areas as CSS grid-template-areas value:
                                // "'name1 name2' 'name3 name4'"
                                let areas_str = areas
                                    .iter()
                                    .map(|a| format!("'{}'", a))
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                styles.grid_template_areas = Some(areas_str);
                            }
                            if !rows.is_empty() {
                                styles.grid_template_rows = Some(rows.join(" "));
                            }
                        }
                    }
                }
                "column-gap" => styles.gap = Some(clean_value),
                "row-gap" => {
                    // Only set gap if column-gap hasn't already set it
                    if styles.gap.is_none() {
                        styles.gap = Some(clean_value);
                    }
                }
                "float" => styles.float = Some(clean_value),
                "clear" => styles.clear = Some(clean_value),
                // Visual transform/effect properties
                "transform" | "-webkit-transform" => styles.transform = Some(clean_value),
                "transform-origin" | "-webkit-transform-origin" => {
                    styles.transform_origin = Some(clean_value)
                }
                "filter" | "-webkit-filter" => styles.filter = Some(clean_value),
                "backdrop-filter" | "-webkit-backdrop-filter" => {
                    styles.backdrop_filter = Some(clean_value)
                }
                "animation" | "-webkit-animation" => styles.animation = Some(clean_value),
                "animation-name" | "-webkit-animation-name" => {
                    styles.animation_name = Some(clean_value)
                }
                "animation-duration" | "-webkit-animation-duration" => {
                    styles.animation_duration = Some(clean_value)
                }
                "transition" | "-webkit-transition" => styles.transition = Some(clean_value),
                "clip-path" | "-webkit-clip-path" => styles.clip_path = Some(clean_value),
                "mask" | "-webkit-mask" => styles.mask = Some(clean_value),
                "mix-blend-mode" => styles.mix_blend_mode = Some(clean_value),
                "object-fit" => styles.object_fit = Some(clean_value),
                "object-position" => styles.object_position = Some(clean_value),
                "box-shadow" | "-webkit-box-shadow" => styles.box_shadow = Some(clean_value),
                "text-shadow" => styles.text_shadow = Some(clean_value),
                "outline" => styles.outline = Some(clean_value),
                "overflow-x" => styles.overflow_x = Some(clean_value),
                "overflow-y" => styles.overflow_y = Some(clean_value),
                "text-overflow" => styles.text_overflow = Some(clean_value),
                "word-break" => styles.word_break = Some(clean_value),
                "overflow-wrap" | "word-wrap" => styles.overflow_wrap = Some(clean_value),
                "vertical-align" => styles.vertical_align = Some(clean_value),
                "content" => styles.content = Some(clean_value),
                "pointer-events" => styles.pointer_events = Some(clean_value),
                "user-select" | "-webkit-user-select" | "-moz-user-select" => {
                    styles.user_select = Some(clean_value)
                }
                "appearance" | "-webkit-appearance" | "-moz-appearance" => {
                    styles.appearance = Some(clean_value)
                }
                "will-change" => styles.will_change = Some(clean_value),
                "contain" => styles.contain = Some(clean_value),
                "container-type" => styles.container_type = Some(clean_value),
                "aspect-ratio" => styles.aspect_ratio = Some(clean_value),
                "justify-self" => styles.justify_self = Some(clean_value),
                "place-items" => styles.place_items = Some(clean_value),
                "place-content" => styles.place_content = Some(clean_value),
                "row-gap" => styles.row_gap = Some(clean_value),
                "column-count" => styles.column_count = Some(clean_value),
                "direction" => styles.direction = Some(clean_value),
                "writing-mode" => styles.writing_mode = Some(clean_value),
                "counter-reset" => styles.counter_reset = Some(clean_value),
                "counter-increment" => styles.counter_increment = Some(clean_value),
                // Logical properties → map to physical equivalents
                "inline-size" => styles.width = Some(clean_value),
                "block-size" => styles.height = Some(clean_value),
                "min-inline-size" => styles.min_width = Some(clean_value),
                "min-block-size" => styles.min_height = Some(clean_value),
                "max-inline-size" => styles.max_width = Some(clean_value),
                "max-block-size" => styles.max_height = Some(clean_value),
                "margin-inline-start" | "margin-inline" => {
                    let mut margin = styles.margin.clone().unwrap_or_default();
                    margin.left = clean_value;
                    styles.margin = Some(margin);
                }
                "margin-inline-end" => {
                    let mut margin = styles.margin.clone().unwrap_or_default();
                    margin.right = clean_value;
                    styles.margin = Some(margin);
                }
                "margin-block-start" | "margin-block" => {
                    let mut margin = styles.margin.clone().unwrap_or_default();
                    margin.top = clean_value;
                    styles.margin = Some(margin);
                }
                "margin-block-end" => {
                    let mut margin = styles.margin.clone().unwrap_or_default();
                    margin.bottom = clean_value;
                    styles.margin = Some(margin);
                }
                "padding-inline-start" | "padding-inline" => {
                    let mut padding = styles.padding.clone().unwrap_or_default();
                    padding.left = clean_value;
                    styles.padding = Some(padding);
                }
                "padding-inline-end" => {
                    let mut padding = styles.padding.clone().unwrap_or_default();
                    padding.right = clean_value;
                    styles.padding = Some(padding);
                }
                "padding-block-start" | "padding-block" => {
                    let mut padding = styles.padding.clone().unwrap_or_default();
                    padding.top = clean_value;
                    styles.padding = Some(padding);
                }
                "padding-block-end" => {
                    let mut padding = styles.padding.clone().unwrap_or_default();
                    padding.bottom = clean_value;
                    styles.padding = Some(padding);
                }
                "border-inline-start" => {
                    styles.border_left = Some(Self::parse_border_shorthand(&clean_value));
                }
                "border-inline-end" => {
                    styles.border_right = Some(Self::parse_border_shorthand(&clean_value));
                }
                "border-block-start" => {
                    styles.border_top = Some(Self::parse_border_shorthand(&clean_value));
                }
                "border-block-end" => {
                    styles.border_bottom = Some(Self::parse_border_shorthand(&clean_value));
                }
                "border-inline-start-width" => {
                    let mut b = styles.border_left.clone().unwrap_or_default();
                    b.width = clean_value;
                    styles.border_left = Some(b);
                }
                "border-inline-end-width" => {
                    let mut b = styles.border_right.clone().unwrap_or_default();
                    b.width = clean_value;
                    styles.border_right = Some(b);
                }
                "border-inline-start-style" => {
                    let mut b = styles.border_left.clone().unwrap_or_default();
                    b.style = clean_value;
                    styles.border_left = Some(b);
                }
                "border-inline-end-style" => {
                    let mut b = styles.border_right.clone().unwrap_or_default();
                    b.style = clean_value;
                    styles.border_right = Some(b);
                }
                "border-inline-start-color" => {
                    let mut b = styles.border_left.clone().unwrap_or_default();
                    b.color = clean_value;
                    styles.border_left = Some(b);
                }
                "border-inline-end-color" => {
                    let mut b = styles.border_right.clone().unwrap_or_default();
                    b.color = clean_value;
                    styles.border_right = Some(b);
                }
                "border-block-start-width" => {
                    let mut b = styles.border_top.clone().unwrap_or_default();
                    b.width = clean_value;
                    styles.border_top = Some(b);
                }
                "border-block-end-width" => {
                    let mut b = styles.border_bottom.clone().unwrap_or_default();
                    b.width = clean_value;
                    styles.border_bottom = Some(b);
                }
                "border-block-start-style" => {
                    let mut b = styles.border_top.clone().unwrap_or_default();
                    b.style = clean_value;
                    styles.border_top = Some(b);
                }
                "border-block-end-style" => {
                    let mut b = styles.border_bottom.clone().unwrap_or_default();
                    b.style = clean_value;
                    styles.border_bottom = Some(b);
                }
                "border-block-start-color" => {
                    let mut b = styles.border_top.clone().unwrap_or_default();
                    b.color = clean_value;
                    styles.border_top = Some(b);
                }
                "border-block-end-color" => {
                    let mut b = styles.border_bottom.clone().unwrap_or_default();
                    b.color = clean_value;
                    styles.border_bottom = Some(b);
                }
                "border-inline-width" => {
                    let mut bl = styles.border_left.clone().unwrap_or_default();
                    bl.width = clean_value.clone();
                    styles.border_left = Some(bl);
                    let mut br = styles.border_right.clone().unwrap_or_default();
                    br.width = clean_value;
                    styles.border_right = Some(br);
                }
                "border-block-width" => {
                    let mut bt = styles.border_top.clone().unwrap_or_default();
                    bt.width = clean_value.clone();
                    styles.border_top = Some(bt);
                    let mut bb = styles.border_bottom.clone().unwrap_or_default();
                    bb.width = clean_value;
                    styles.border_bottom = Some(bb);
                }
                "inset-inline-start" | "inset" => {
                    styles.other.insert("left".to_string(), clean_value);
                }
                "inset-inline-end" => {
                    styles.other.insert("right".to_string(), clean_value);
                }
                "inset-block-start" => {
                    styles.other.insert("top".to_string(), clean_value);
                }
                "inset-block-end" => {
                    styles.other.insert("bottom".to_string(), clean_value);
                }
                _ => {
                    styles.other.insert(prop.clone(), clean_value);
                }
            }
        }
    }

    /// Extract a color value from a CSS `background` shorthand.
    ///
    /// The background shorthand can contain: color, image (url/gradient), position, size,
    /// repeat, origin, clip, attachment. lightningcss often serializes values like:
    ///   `#eaecf0 none` or `#fff no-repeat` or `transparent url(...)` or `rgb(0, 0, 0) none`
    ///
    /// This extracts the color token by:
    /// 1. If it's a single token with no spaces, return it directly (simple color value)
    /// 2. If it starts with a color function (rgb/hsl/rgba/hsla), extract the full function call
    /// 3. Scan tokens for hex colors, named colors, or 'transparent'/'inherit'
    /// 4. Filter out known background-specific keywords (none, no-repeat, repeat, etc.)
    pub(crate) fn extract_color_from_background(value: &str) -> Option<String> {
        let v = value.trim();

        // Empty
        if v.is_empty() {
            return None;
        }

        // url(...) backgrounds don't have a simple color to extract
        if v.starts_with("url(") {
            return None;
        }

        // Single token, no spaces: treat as a color directly
        if !v.contains(' ') {
            if v == "none" || v == "initial" || v == "unset" {
                return None;
            }
            return Some(v.to_string());
        }

        // If it starts with rgb/hsl/rgba/hsla, extract the function call
        if v.starts_with("rgb") || v.starts_with("hsl") {
            // Find the matching closing paren
            if let Some(open) = v.find('(') {
                let mut depth = 0;
                for (i, ch) in v[open..].char_indices() {
                    match ch {
                        '(' => depth += 1,
                        ')' => {
                            depth -= 1;
                            if depth == 0 {
                                return Some(v[..open + i + 1].to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
            // Malformed but starts with color function — return entire value
            return Some(v.to_string());
        }

        // Multi-token: scan for a color token
        // Background-specific keywords to filter out
        let bg_keywords = [
            "none",
            "no-repeat",
            "repeat",
            "repeat-x",
            "repeat-y",
            "space",
            "round",
            "scroll",
            "fixed",
            "local",
            "border-box",
            "padding-box",
            "content-box",
            "cover",
            "contain",
            "center",
            "top",
            "bottom",
            "left",
            "right",
            "initial",
            "unset",
        ];

        // Scan tokens — first token that looks like a color wins
        for token in v.split_whitespace() {
            // Skip background-specific keywords
            if bg_keywords.contains(&token.to_lowercase().as_str()) {
                continue;
            }
            // Skip url(...)
            if token.starts_with("url(") {
                return None; // url() background — no simple color
            }
            // Skip percentage/length values (position/size)
            if token.ends_with('%')
                || token.ends_with("px")
                || token.ends_with("em")
                || token.ends_with("rem")
                || token.ends_with("vw")
                || token.ends_with("vh")
            {
                continue;
            }
            // Skip bare numbers (e.g., "0 0" for position)
            if token.parse::<f64>().is_ok() {
                continue;
            }
            // Skip gradient functions
            if token.starts_with("linear-gradient")
                || token.starts_with("radial-gradient")
                || token.starts_with("conic-gradient")
                || token.starts_with("repeating-")
            {
                return None; // gradient background — no simple color
            }

            // Hex color
            if token.starts_with('#') {
                return Some(token.to_string());
            }
            // Named color or transparent/inherit/currentcolor
            if token == "transparent"
                || token == "inherit"
                || token == "currentcolor"
                || token == "currentColor"
            {
                return Some(token.to_string());
            }
            // Anything else that doesn't match bg keywords is likely a named color
            // (e.g., "red", "white", "aliceblue")
            if token.chars().all(|c| c.is_ascii_alphabetic()) {
                return Some(token.to_string());
            }
        }

        None
    }

    /// Parse shorthand box model values (margin/padding)
    pub(crate) fn parse_box_model(&self, value: &str) -> BoxModel {
        let parts: Vec<&str> = value.split_whitespace().collect();
        match parts.len() {
            1 => BoxModel {
                top: parts[0].to_string(),
                right: parts[0].to_string(),
                bottom: parts[0].to_string(),
                left: parts[0].to_string(),
            },
            2 => BoxModel {
                top: parts[0].to_string(),
                right: parts[1].to_string(),
                bottom: parts[0].to_string(),
                left: parts[1].to_string(),
            },
            3 => BoxModel {
                top: parts[0].to_string(),
                right: parts[1].to_string(),
                bottom: parts[2].to_string(),
                left: parts[1].to_string(),
            },
            4 => BoxModel {
                top: parts[0].to_string(),
                right: parts[1].to_string(),
                bottom: parts[2].to_string(),
                left: parts[3].to_string(),
            },
            _ => BoxModel::default(),
        }
    }

    /// Parse a CSS border shorthand value into a BorderStyles struct.
    /// Format: `<width> <style> <color>` — e.g., "1px solid #a2a9b1"
    /// Parts can appear in any order. Missing parts get defaults.
    pub(crate) fn parse_border_shorthand(value: &str) -> BorderStyles {
        let border_style_keywords = [
            "none", "hidden", "dotted", "dashed", "solid", "double", "groove", "ridge", "inset",
            "outset",
        ];

        let mut width = String::new();
        let mut style = String::new();
        let mut color_parts: Vec<String> = Vec::new();
        let mut in_paren = 0;
        let mut current = String::new();

        // Tokenize respecting parentheses (for rgb(), var(), etc.)
        for ch in value.chars() {
            match ch {
                '(' => {
                    in_paren += 1;
                    current.push(ch);
                }
                ')' => {
                    in_paren -= 1;
                    current.push(ch);
                }
                ' ' | '\t' if in_paren == 0 => {
                    if !current.is_empty() {
                        let token = current.trim().to_string();
                        current.clear();
                        if border_style_keywords.contains(&token.to_lowercase().as_str()) {
                            style = token;
                        } else if Self::looks_like_length(&token) {
                            width = token;
                        } else {
                            color_parts.push(token);
                        }
                    }
                }
                _ => {
                    current.push(ch);
                }
            }
        }
        if !current.is_empty() {
            let token = current.trim().to_string();
            if border_style_keywords.contains(&token.to_lowercase().as_str()) {
                style = token;
            } else if Self::looks_like_length(&token) {
                width = token;
            } else {
                color_parts.push(token);
            }
        }

        let color = if color_parts.is_empty() {
            String::new()
        } else {
            color_parts.join(" ")
        };

        BorderStyles {
            width,
            style,
            color,
        }
    }

    /// Check if a token looks like a CSS length value.
    pub(crate) fn looks_like_length(token: &str) -> bool {
        let t = token.to_lowercase();
        t == "0"
            || t == "thin"
            || t == "medium"
            || t == "thick"
            || t.ends_with("px")
            || t.ends_with("em")
            || t.ends_with("rem")
            || t.ends_with("pt")
            || t.ends_with("vw")
            || t.ends_with("vh")
            || t.ends_with("%")
    }
}
