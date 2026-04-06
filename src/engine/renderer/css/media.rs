use super::types::CssProcessor;
use lightningcss::rules::supports::SupportsCondition;

impl CssProcessor {
    /// Evaluate a media query string against the current viewport
    pub(crate) fn evaluate_media_query(&self, query: &str) -> bool {
        // Handle multiple queries separated by comma (OR logic)
        for single_query in query.split(',') {
            if self.evaluate_single_media_query(single_query.trim()) {
                return true;
            }
        }
        false
    }

    /// Evaluate a single media query
    fn evaluate_single_media_query(&self, query: &str) -> bool {
        let query = query.trim();

        // Empty query matches everything
        if query.is_empty() {
            return true;
        }

        // Handle "not" prefix
        let (negated, query) = if let Some(rest) = query.strip_prefix("not ") {
            (true, rest)
        } else {
            (false, query)
        };

        let result = self.evaluate_media_query_inner(query);

        if negated { !result } else { result }
    }

    /// Inner media query evaluation
    fn evaluate_media_query_inner(&self, query: &str) -> bool {
        let query = query.trim();

        // "all" always matches
        if query == "all" {
            return true;
        }

        // "screen" matches (we're a screen renderer)
        if query == "screen" {
            return true;
        }

        // "print" never matches
        if query == "print" {
            return false;
        }

        // Handle "only screen and (...)", "screen and (...)", or just "(...)"
        // The "only" keyword is for backwards compatibility and should be ignored.
        let conditions_str = if let Some(rest) = query.strip_prefix("only screen and ") {
            rest
        } else if query.starts_with("only screen") {
            return true; // "only screen" with no conditions
        } else if let Some(rest) = query.strip_prefix("screen and ") {
            rest
        } else if let Some(rest) = query.strip_prefix("all and ") {
            rest
        } else {
            query
        };

        // Parse individual conditions: "(max-width: 700px)" etc.
        // Handle "and" joined conditions
        let mut all_match = true;
        for part in conditions_str.split(" and ") {
            let part = part.trim().trim_start_matches('(').trim_end_matches(')');
            if !self.evaluate_media_feature(part) {
                all_match = false;
                break;
            }
        }

        all_match
    }

    /// Evaluate a single media feature like "max-width: 700px" or "width <= 700px"
    fn evaluate_media_feature(&self, feature: &str) -> bool {
        // Handle modern range syntax: "width <= 700px", "width >= 768px", "700px <= width"
        if let Some(result) = self.evaluate_range_media_feature(feature) {
            return result;
        }

        // Handle legacy syntax: "max-width: 700px"
        let parts: Vec<&str> = feature.splitn(2, ':').collect();
        if parts.len() != 2 {
            // Features without values (e.g., "color") — assume true
            return true;
        }

        let name = parts[0].trim();
        let value_str = parts[1].trim();

        match name {
            "max-width" => {
                if let Some(px) = Self::parse_media_length(value_str) {
                    self.viewport_width <= px
                } else {
                    true
                }
            }
            "min-width" => {
                if let Some(px) = Self::parse_media_length(value_str) {
                    self.viewport_width >= px
                } else {
                    true
                }
            }
            "max-height" | "min-height" => {
                // We don't track viewport height precisely — assume true
                true
            }
            "prefers-color-scheme" => {
                // Default to light mode
                value_str == "light"
            }
            "prefers-reduced-motion" => value_str == "no-preference",
            "prefers-contrast" => value_str == "no-preference",
            "prefers-reduced-transparency" => value_str == "no-preference",
            "forced-colors" => value_str == "none",
            "orientation" => {
                // Assume landscape for a headless viewport wider than tall
                value_str == "landscape"
            }
            "hover" => value_str == "hover",
            "any-hover" => value_str == "hover",
            "pointer" => value_str == "fine",
            "any-pointer" => value_str == "fine",
            "color" => true,
            "color-gamut" => value_str == "srgb",
            "display-mode" => value_str == "browser",
            "scripting" => value_str == "enabled",
            "update" => value_str == "fast",
            _ => true, // Unknown features — assume match to be permissive
        }
    }

    /// Evaluate CSS Media Queries Level 4 range syntax
    /// e.g., "width <= 700px", "width >= 768px", "width < 1200px", "700px <= width"
    fn evaluate_range_media_feature(&self, feature: &str) -> Option<bool> {
        let feature = feature.trim();

        // Try patterns: "prop <= val", "prop >= val", "prop < val", "prop > val"
        for (op, op_str) in &[("<=", "<="), (">=", ">="), ("<", "<"), (">", ">")] {
            if let Some(pos) = feature.find(op_str) {
                let left = feature[..pos].trim();
                let right = feature[pos + op.len()..].trim();

                // Determine which side is the property name
                let (prop, value, reversed) = if left
                    .chars()
                    .next()
                    .map(|c| c.is_alphabetic())
                    .unwrap_or(false)
                {
                    (left, right, false)
                } else {
                    (right, left, true)
                };

                let viewport_val = match prop {
                    "width" => Some(self.viewport_width),
                    "height" => return Some(true), // Not tracked precisely
                    _ => None,
                };

                if let (Some(vp), Some(px)) = (viewport_val, Self::parse_media_length(value)) {
                    let result = if reversed {
                        // "700px <= width" means width >= 700px
                        match *op_str {
                            "<=" => vp >= px,
                            ">=" => vp <= px,
                            "<" => vp > px,
                            ">" => vp < px,
                            _ => true,
                        }
                    } else {
                        // "width <= 700px"
                        match *op_str {
                            "<=" => vp <= px,
                            ">=" => vp >= px,
                            "<" => vp < px,
                            ">" => vp > px,
                            _ => true,
                        }
                    };
                    return Some(result);
                }
            }
        }

        None // Not a range expression
    }

    /// Parse a media query length value like "700px" to f32
    fn parse_media_length(value: &str) -> Option<f32> {
        let v = value.trim();
        // Handle calc() expressions: calc(640px - 1px) → 639
        if v.starts_with("calc(") && v.ends_with(")") {
            return Self::evaluate_calc_expr(&v[5..v.len() - 1]);
        }
        if v.ends_with("px") {
            v.trim_end_matches("px").parse::<f32>().ok()
        } else if v.ends_with("em") || v.ends_with("rem") {
            // 1em/rem = 16px for media queries (always relative to initial value)
            v.trim_end_matches("em")
                .trim_end_matches("r")
                .parse::<f32>()
                .ok()
                .map(|n| n * 16.0)
        } else {
            v.parse::<f32>().ok()
        }
    }

    /// Evaluate a simple calc() expression in media queries.
    /// Handles: calc(Npx +/- Mpx), calc(Nem +/- Mem), etc.
    fn evaluate_calc_expr(expr: &str) -> Option<f32> {
        let expr = expr.trim();

        // Try to find + or - operator (not inside a sub-expression)
        // Handle: "640px - 1px", "100vw - 2rem"
        for op in [" - ", " + "] {
            if let Some(pos) = expr.find(op) {
                let left = expr[..pos].trim();
                let right = expr[pos + op.len()..].trim();
                let left_val = Self::parse_media_length(left)?;
                let right_val = Self::parse_media_length(right)?;
                return Some(if op == " + " {
                    left_val + right_val
                } else {
                    left_val - right_val
                });
            }
        }

        // Fallback: try parsing the whole expression as a single value
        Self::parse_media_length(expr)
    }

    /// Evaluate an `@supports` condition.
    /// A headless browser that uses lightningcss for parsing can claim support for any
    /// property/value that lightningcss successfully parses. For `not`, `and`, `or`
    /// combinators we evaluate recursively.
    pub(crate) fn evaluate_supports_condition(&self, condition: &SupportsCondition) -> bool {
        match condition {
            SupportsCondition::Not(inner) => !self.evaluate_supports_condition(inner),
            SupportsCondition::And(conditions) => conditions
                .iter()
                .all(|c| self.evaluate_supports_condition(c)),
            SupportsCondition::Or(conditions) => conditions
                .iter()
                .any(|c| self.evaluate_supports_condition(c)),
            SupportsCondition::Declaration {
                property_id: _,
                value: _,
            } => {
                // If lightningcss parsed this declaration successfully, we "support" it.
                // The mere presence of a Declaration variant means it was parseable.
                true
            }
            SupportsCondition::Selector(_) => {
                // selector() function in @supports — be permissive
                true
            }
            SupportsCondition::Unknown(_) => {
                // Unknown conditions (e.g., future syntax) — be conservative
                false
            }
        }
    }
}
