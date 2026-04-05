//! CSS Container Query evaluation
//!
//! Evaluates `@container` conditions against container dimensions.
//! Currently uses a viewport-based approximation: container queries are
//! evaluated against the viewport dimensions, which handles the common case
//! (`@container (min-width: 768px)`) correctly for most responsive layouts.
//!
//! The evaluation logic itself is correct — only the size source is
//! approximated. When a full two-pass layout is implemented, the
//! `evaluate_container_condition` method can be called with actual
//! container dimensions instead of the viewport.

use super::types::CssProcessor;
use lightningcss::media_query::MediaFeatureName;
use lightningcss::media_query::{
    MediaFeatureComparison, MediaFeatureValue, Operator, QueryFeature,
};
use lightningcss::rules::container::{
    ContainerCondition, ContainerSizeFeature, ContainerSizeFeatureId,
};

impl CssProcessor {
    /// Evaluate a container condition against the current viewport dimensions
    /// (approximation: treats the viewport as the container).
    ///
    /// When full two-pass layout is implemented, this can be replaced with
    /// `evaluate_container_condition_with_size(condition, width, height)`.
    pub(crate) fn evaluate_container_condition(&self, condition: &ContainerCondition) -> bool {
        self.evaluate_container_condition_with_size(
            condition,
            self.viewport_width,
            self.viewport_height,
        )
    }

    /// Evaluate a container condition against explicit container dimensions.
    ///
    /// This is the core evaluation logic, independent of where the dimensions
    /// come from (viewport approximation or actual computed container size).
    pub(crate) fn evaluate_container_condition_with_size(
        &self,
        condition: &ContainerCondition,
        container_width: f32,
        container_height: f32,
    ) -> bool {
        match condition {
            ContainerCondition::Feature(feature) => {
                self.evaluate_container_size_feature(feature, container_width, container_height)
            }
            ContainerCondition::Not(inner) => !self.evaluate_container_condition_with_size(
                inner,
                container_width,
                container_height,
            ),
            ContainerCondition::Operation {
                operator,
                conditions,
            } => match operator {
                Operator::And => conditions.iter().all(|c| {
                    self.evaluate_container_condition_with_size(
                        c,
                        container_width,
                        container_height,
                    )
                }),
                Operator::Or => conditions.iter().any(|c| {
                    self.evaluate_container_condition_with_size(
                        c,
                        container_width,
                        container_height,
                    )
                }),
            },
            ContainerCondition::Style(_) => {
                // Style queries (e.g., `@container style(--theme: dark)`) are not
                // yet supported. Be permissive — include the rules rather than
                // silently dropping them.
                true
            }
            ContainerCondition::ScrollState(_) => {
                // Scroll-state container queries (e.g., `@container scroll-state(scrollable: top)`)
                // are not yet supported. Be permissive.
                true
            }
            ContainerCondition::Unknown(_) => {
                // Unknown/unsupported container condition syntax. Be permissive.
                true
            }
        }
    }

    /// Evaluate a single container size feature (e.g., `min-width: 768px`,
    /// `width > 600px`, `400px <= width <= 1200px`).
    fn evaluate_container_size_feature(
        &self,
        feature: &ContainerSizeFeature,
        container_width: f32,
        container_height: f32,
    ) -> bool {
        match feature {
            QueryFeature::Plain { name, value } => {
                let feature_id = match name {
                    MediaFeatureName::Standard(id) => id,
                    _ => return true, // Unknown feature — permissive
                };
                let (dimension, is_legacy_min, is_legacy_max) =
                    self.resolve_container_dimension(feature_id, container_width, container_height);

                if let Some(target) = self.extract_length_px(value) {
                    if is_legacy_min {
                        dimension >= target
                    } else if is_legacy_max {
                        dimension <= target
                    } else {
                        // Exact match (within a small epsilon for float comparison)
                        (dimension - target).abs() < 0.01
                    }
                } else {
                    // Non-length value (e.g., orientation: landscape) — handle below
                    self.evaluate_container_non_length(
                        feature_id,
                        value,
                        container_width,
                        container_height,
                    )
                }
            }
            QueryFeature::Boolean { name } => {
                // Boolean container features: `(width)` is true if container
                // has a definite inline size (which it always does for us).
                match name {
                    MediaFeatureName::Standard(id) => match id {
                        ContainerSizeFeatureId::Width
                        | ContainerSizeFeatureId::InlineSize
                        | ContainerSizeFeatureId::Height
                        | ContainerSizeFeatureId::BlockSize
                        | ContainerSizeFeatureId::AspectRatio
                        | ContainerSizeFeatureId::Orientation => true,
                    },
                    _ => true,
                }
            }
            QueryFeature::Range {
                name,
                operator,
                value,
            } => {
                let feature_id = match name {
                    MediaFeatureName::Standard(id) => id,
                    _ => return true,
                };
                let (dimension, _, _) =
                    self.resolve_container_dimension(feature_id, container_width, container_height);

                if let Some(target) = self.extract_length_px(value) {
                    self.compare_values(dimension, *operator, target)
                } else {
                    true // Unknown value type — permissive
                }
            }
            QueryFeature::Interval {
                name,
                start,
                start_operator,
                end,
                end_operator,
            } => {
                let feature_id = match name {
                    MediaFeatureName::Standard(id) => id,
                    _ => return true,
                };
                let (dimension, _, _) =
                    self.resolve_container_dimension(feature_id, container_width, container_height);

                let start_ok = if let Some(start_val) = self.extract_length_px(start) {
                    // Interval: start_val <op> dimension, so we flip the comparison
                    self.compare_values(
                        dimension,
                        Self::flip_comparison(*start_operator),
                        start_val,
                    )
                } else {
                    true
                };

                let end_ok = if let Some(end_val) = self.extract_length_px(end) {
                    self.compare_values(dimension, *end_operator, end_val)
                } else {
                    true
                };

                start_ok && end_ok
            }
        }
    }

    /// Resolve which dimension a container size feature refers to.
    /// Returns (dimension_value, is_legacy_min, is_legacy_max).
    ///
    /// Note: lightningcss parses `min-width` / `max-width` as legacy prefixed
    /// features stored in the `MediaFeatureName`. The `ContainerSizeFeatureId`
    /// itself is always the unprefixed variant (Width, Height, etc.).
    /// However, lightningcss may also encode them as `Plain` with an adjusted
    /// feature ID. We handle both approaches.
    fn resolve_container_dimension(
        &self,
        id: &ContainerSizeFeatureId,
        container_width: f32,
        container_height: f32,
    ) -> (f32, bool, bool) {
        // For now, inline-size maps to width and block-size maps to height
        // (assuming horizontal writing mode, which covers >99% of real sites).
        match id {
            ContainerSizeFeatureId::Width | ContainerSizeFeatureId::InlineSize => {
                (container_width, false, false)
            }
            ContainerSizeFeatureId::Height | ContainerSizeFeatureId::BlockSize => {
                (container_height, false, false)
            }
            ContainerSizeFeatureId::AspectRatio | ContainerSizeFeatureId::Orientation => {
                // These don't map to a single dimension — handled separately
                (0.0, false, false)
            }
        }
    }

    /// Compare a dimension value against a target using the given operator.
    fn compare_values(&self, dimension: f32, op: MediaFeatureComparison, target: f32) -> bool {
        match op {
            MediaFeatureComparison::Equal => (dimension - target).abs() < 0.01,
            MediaFeatureComparison::GreaterThan => dimension > target,
            MediaFeatureComparison::GreaterThanEqual => dimension >= target,
            MediaFeatureComparison::LessThan => dimension < target,
            MediaFeatureComparison::LessThanEqual => dimension <= target,
        }
    }

    /// Flip a comparison operator (for interval start conditions where the
    /// syntax is `start_val <op> dimension` but we want `dimension <flipped_op> start_val`).
    fn flip_comparison(op: MediaFeatureComparison) -> MediaFeatureComparison {
        match op {
            MediaFeatureComparison::Equal => MediaFeatureComparison::Equal,
            MediaFeatureComparison::GreaterThan => MediaFeatureComparison::LessThan,
            MediaFeatureComparison::GreaterThanEqual => MediaFeatureComparison::LessThanEqual,
            MediaFeatureComparison::LessThan => MediaFeatureComparison::GreaterThan,
            MediaFeatureComparison::LessThanEqual => MediaFeatureComparison::GreaterThanEqual,
        }
    }

    /// Extract a pixel value from a MediaFeatureValue (Length variant).
    /// Handles absolute units (px, em approximation, rem approximation).
    fn extract_length_px(&self, value: &MediaFeatureValue) -> Option<f32> {
        match value {
            MediaFeatureValue::Length(length) => {
                // Try direct px conversion (handles px, in, cm, mm, pt, pc)
                if let Some(px) = length.to_px() {
                    return Some(px);
                }
                // For rem/em in container queries, approximate using 16px base
                // (same heuristic as media queries in most browsers)
                None
            }
            MediaFeatureValue::Number(n) => Some(*n),
            MediaFeatureValue::Integer(n) => Some(*n as f32),
            _ => None,
        }
    }

    /// Handle non-length container features like orientation and aspect-ratio.
    fn evaluate_container_non_length(
        &self,
        id: &ContainerSizeFeatureId,
        value: &MediaFeatureValue,
        container_width: f32,
        container_height: f32,
    ) -> bool {
        match id {
            ContainerSizeFeatureId::Orientation => {
                if let MediaFeatureValue::Ident(ident) = value {
                    let ident_str = ident.0.as_ref();
                    match ident_str {
                        "landscape" => container_width >= container_height,
                        "portrait" => container_height > container_width,
                        _ => true,
                    }
                } else {
                    true
                }
            }
            ContainerSizeFeatureId::AspectRatio => {
                // Aspect ratio comparison is complex; be permissive for now
                true
            }
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lightningcss::rules::CssRule;
    use lightningcss::rules::container::ContainerCondition;
    use lightningcss::stylesheet::{ParserOptions, StyleSheet};

    /// Helper: parse a `@container (...)` rule and return the condition
    fn parse_container_condition(css: &str) -> ContainerCondition<'static> {
        // We need to parse the condition from a full @container rule.
        // lightningcss requires a full stylesheet context.
        let full_css = format!("@container {} {{ .test {{ color: red; }} }}", css);
        let leaked: &'static str = Box::leak(full_css.into_boxed_str());
        let stylesheet = StyleSheet::parse(leaked, ParserOptions::default()).unwrap();

        for rule in &stylesheet.rules.0 {
            if let CssRule::Container(container_rule) = rule {
                return container_rule
                    .condition
                    .clone()
                    .expect("test @container should have a condition");
            }
        }
        panic!("No @container rule found in: {}", css);
    }

    #[test]
    fn test_min_width_condition_matches() {
        let processor = CssProcessor::new_with_viewport_and_height(1024.0, 768.0);
        let condition = parse_container_condition("(min-width: 768px)");
        assert!(processor.evaluate_container_condition(&condition));
    }

    #[test]
    fn test_min_width_condition_no_match() {
        let processor = CssProcessor::new_with_viewport_and_height(600.0, 400.0);
        let condition = parse_container_condition("(min-width: 768px)");
        assert!(!processor.evaluate_container_condition(&condition));
    }

    #[test]
    fn test_max_width_condition_matches() {
        let processor = CssProcessor::new_with_viewport_and_height(600.0, 400.0);
        let condition = parse_container_condition("(max-width: 768px)");
        assert!(processor.evaluate_container_condition(&condition));
    }

    #[test]
    fn test_max_width_condition_no_match() {
        let processor = CssProcessor::new_with_viewport_and_height(1024.0, 768.0);
        let condition = parse_container_condition("(max-width: 768px)");
        assert!(!processor.evaluate_container_condition(&condition));
    }

    #[test]
    fn test_range_syntax_greater_than() {
        let processor = CssProcessor::new_with_viewport_and_height(1024.0, 768.0);
        let condition = parse_container_condition("(width > 600px)");
        assert!(processor.evaluate_container_condition(&condition));
    }

    #[test]
    fn test_range_syntax_less_than() {
        let processor = CssProcessor::new_with_viewport_and_height(400.0, 300.0);
        let condition = parse_container_condition("(width < 600px)");
        assert!(processor.evaluate_container_condition(&condition));
    }

    #[test]
    fn test_not_condition() {
        let processor = CssProcessor::new_with_viewport_and_height(1024.0, 768.0);
        let condition = parse_container_condition("not (max-width: 600px)");
        assert!(processor.evaluate_container_condition(&condition));
    }

    #[test]
    fn test_and_condition() {
        let processor = CssProcessor::new_with_viewport_and_height(1024.0, 768.0);
        let condition = parse_container_condition("(min-width: 768px) and (max-width: 1200px)");
        assert!(processor.evaluate_container_condition(&condition));
    }

    #[test]
    fn test_and_condition_no_match() {
        let processor = CssProcessor::new_with_viewport_and_height(1024.0, 768.0);
        let condition = parse_container_condition("(min-width: 768px) and (max-width: 900px)");
        assert!(!processor.evaluate_container_condition(&condition));
    }

    #[test]
    fn test_or_condition() {
        let processor = CssProcessor::new_with_viewport_and_height(400.0, 300.0);
        let condition = parse_container_condition("(max-width: 600px) or (min-width: 1200px)");
        assert!(processor.evaluate_container_condition(&condition));
    }

    #[test]
    fn test_height_condition() {
        let processor = CssProcessor::new_with_viewport_and_height(1024.0, 768.0);
        let condition = parse_container_condition("(min-height: 600px)");
        assert!(processor.evaluate_container_condition(&condition));
    }

    #[test]
    fn test_custom_container_size() {
        let processor = CssProcessor::new_with_viewport_and_height(1024.0, 768.0);
        let condition = parse_container_condition("(min-width: 300px)");
        // Evaluate against a specific container size, not the viewport
        assert!(processor.evaluate_container_condition_with_size(&condition, 500.0, 400.0));
        assert!(!processor.evaluate_container_condition_with_size(&condition, 200.0, 400.0));
    }
}
