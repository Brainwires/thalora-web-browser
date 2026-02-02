//! Behavioral Simulation Module
//!
//! Generates human-like interaction patterns for web challenges.
//! This is not about "bypassing" security - it's about making the browser
//! behave like a real browser with a real user interacting with it.
//!
//! Key behaviors simulated:
//! - Page scanning (moving mouse around to "look" at the page)
//! - Fitts' Law movement timing
//! - Human-like click patterns
//! - Random micro-delays and jitter
//! - Realistic event dispatch sequences

use anyhow::Result;
use rand::Rng;
use std::time::Duration;

use crate::features::mouse_simulation::{MousePath, MousePathConfig, ClickSequence};
use crate::features::event_dispatcher::{EventSequence, EventAction, EventCoords, MouseEventType, MouseButton};
use super::detector::DetectedChallenge;

/// Configuration for behavioral simulation
#[derive(Debug, Clone)]
pub struct BehavioralConfig {
    /// Whether to simulate page scanning before interaction
    pub simulate_page_scan: bool,
    /// Number of scanning movements (1-5)
    pub scan_movements: usize,
    /// Minimum delay between actions (ms)
    pub min_action_delay_ms: u64,
    /// Maximum delay between actions (ms)
    pub max_action_delay_ms: u64,
    /// Whether to add random micro-delays
    pub add_micro_delays: bool,
    /// Whether to add mouse movement jitter
    pub add_jitter: bool,
    /// Jitter amount in pixels
    pub jitter_amount: f64,
    /// Mouse path configuration
    pub path_config: MousePathConfig,
}

impl Default for BehavioralConfig {
    fn default() -> Self {
        Self {
            simulate_page_scan: true,
            scan_movements: 2,
            min_action_delay_ms: 100,
            max_action_delay_ms: 500,
            add_micro_delays: true,
            add_jitter: true,
            jitter_amount: 2.0,
            path_config: MousePathConfig::default(),
        }
    }
}

/// Result of behavioral simulation
#[derive(Debug, Clone)]
pub struct InteractionResult {
    /// Sequence of events to dispatch
    pub event_sequence: EventSequence,
    /// Page scanning movements (before main interaction)
    pub scan_movements: Vec<EventSequence>,
    /// Total estimated duration
    pub total_duration: Duration,
    /// Random delays to inject between phases
    pub phase_delays: Vec<Duration>,
    /// The click target position (if applicable)
    pub click_target: Option<(f64, f64)>,
}

/// Behavioral simulator for human-like page interaction
pub struct BehavioralSimulator {
    config: BehavioralConfig,
}

impl BehavioralSimulator {
    /// Create a new behavioral simulator with default config
    pub fn new() -> Self {
        Self {
            config: BehavioralConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: BehavioralConfig) -> Self {
        Self { config }
    }

    /// Generate a complete interaction sequence for a challenge
    pub fn generate_interaction(
        &self,
        challenge: &DetectedChallenge,
        viewport_width: f64,
        viewport_height: f64,
    ) -> Result<InteractionResult> {
        let mut rng = rand::thread_rng();

        // Start position: somewhere near the top-left (as if user just loaded the page)
        let start_pos = (
            50.0 + rng.r#gen::<f64>() * 100.0,
            50.0 + rng.r#gen::<f64>() * 100.0,
        );

        // Phase 1: Page scanning (look around the page)
        let scan_movements = if self.config.simulate_page_scan {
            self.generate_scan_movements(start_pos, viewport_width, viewport_height)
        } else {
            vec![]
        };

        // Calculate position after scanning
        let post_scan_pos = scan_movements.last()
            .and_then(|seq| seq.final_position())
            .unwrap_or(start_pos);

        // Phase 2: Generate click target position
        // If challenge requires interaction, move to the widget
        let (click_target, event_sequence) = if challenge.requires_interaction {
            // Calculate a realistic position for the Turnstile/captcha widget
            // Widgets are typically centered horizontally, in the upper-middle area
            let widget_center = self.estimate_widget_position(viewport_width, viewport_height, challenge);
            let click_sequence = EventSequence::click_at(post_scan_pos, widget_center);

            (Some(widget_center), click_sequence)
        } else {
            // For non-interactive challenges, just generate some passive movement
            // to show "presence" on the page
            let end_pos = (
                viewport_width / 2.0 + rng.r#gen::<f64>() * 200.0 - 100.0,
                viewport_height / 2.0 + rng.r#gen::<f64>() * 100.0 - 50.0,
            );
            let move_sequence = EventSequence::move_to(post_scan_pos, end_pos, 800);

            (None, move_sequence)
        };

        // Generate phase delays
        let phase_delays = self.generate_phase_delays(&mut rng);

        // Calculate total duration
        let scan_duration: Duration = scan_movements.iter()
            .map(|s| s.total_duration())
            .sum();
        let event_duration = event_sequence.total_duration();
        let delay_duration: Duration = phase_delays.iter().sum();
        let total_duration = scan_duration + event_duration + delay_duration;

        Ok(InteractionResult {
            event_sequence,
            scan_movements,
            total_duration,
            phase_delays,
            click_target,
        })
    }

    /// Generate page scanning movements
    fn generate_scan_movements(
        &self,
        start_pos: (f64, f64),
        viewport_width: f64,
        viewport_height: f64,
    ) -> Vec<EventSequence> {
        let mut rng = rand::thread_rng();
        let mut movements = Vec::new();
        let mut current_pos = start_pos;

        for i in 0..self.config.scan_movements {
            // Generate a random target within the viewport
            // Bias towards the center and upper areas (where content typically is)
            let target = self.generate_scan_target(&mut rng, viewport_width, viewport_height, i);

            // Calculate duration based on Fitts' Law
            let duration_ms = self.fitts_law_duration(current_pos, target);

            // Generate mouse movement (no click)
            let movement = EventSequence::move_to(current_pos, target, duration_ms);

            current_pos = target;
            movements.push(movement);
        }

        movements
    }

    /// Generate a scan target position based on typical human scanning patterns
    fn generate_scan_target(
        &self,
        rng: &mut impl Rng,
        viewport_width: f64,
        viewport_height: f64,
        scan_index: usize,
    ) -> (f64, f64) {
        // Humans typically scan in an F-pattern or Z-pattern
        // First scan: look at top area
        // Second scan: look at middle/left
        // Third+: more random

        match scan_index {
            0 => {
                // First scan: top-center area (where logos/headers are)
                let x = viewport_width * 0.3 + rng.r#gen::<f64>() * viewport_width * 0.4;
                let y = 100.0 + rng.r#gen::<f64>() * 150.0;
                (x, y)
            }
            1 => {
                // Second scan: middle area (where main content/challenges appear)
                let x = viewport_width * 0.3 + rng.r#gen::<f64>() * viewport_width * 0.4;
                let y = viewport_height * 0.3 + rng.r#gen::<f64>() * viewport_height * 0.3;
                (x, y)
            }
            _ => {
                // Later scans: more random, but avoid edges
                let x = viewport_width * 0.2 + rng.r#gen::<f64>() * viewport_width * 0.6;
                let y = viewport_height * 0.2 + rng.r#gen::<f64>() * viewport_height * 0.5;
                (x, y)
            }
        }
    }

    /// Calculate movement duration using Fitts' Law
    /// Fitts' Law: MT = a + b * log2(2D/W)
    /// Where MT is movement time, D is distance, W is target width
    fn fitts_law_duration(&self, start: (f64, f64), end: (f64, f64)) -> u64 {
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let distance = (dx * dx + dy * dy).sqrt();

        // Fitts' Law constants (empirically determined for mouse movement)
        let a = 200.0; // Base time in ms
        let b = 100.0; // Scaling factor
        let target_width = 20.0; // Assumed target width in pixels

        // Avoid log of zero or negative
        let index_of_difficulty = if distance > 0.0 {
            ((2.0 * distance) / target_width + 1.0).log2()
        } else {
            0.0
        };

        let duration = a + b * index_of_difficulty;

        // Clamp to reasonable range
        (duration as u64).clamp(100, 2000)
    }

    /// Estimate the position of a challenge widget on the page
    fn estimate_widget_position(
        &self,
        viewport_width: f64,
        viewport_height: f64,
        challenge: &DetectedChallenge,
    ) -> (f64, f64) {
        let mut rng = rand::thread_rng();

        // Turnstile/captcha widgets are typically:
        // - Centered horizontally
        // - In the middle-upper portion of the page
        // - About 300x65 pixels in size

        let widget_width = 300.0;
        let widget_height = 65.0;

        // Widget center X (centered with small random offset)
        let center_x = viewport_width / 2.0 + rng.r#gen::<f64>() * 20.0 - 10.0;

        // Widget center Y (typically 30-50% down the page)
        let center_y = viewport_height * 0.35 + rng.r#gen::<f64>() * viewport_height * 0.1;

        // Add small offset to simulate clicking within the widget
        // (not perfectly centered - humans don't click perfectly)
        let click_offset_x = rng.r#gen::<f64>() * widget_width * 0.3 - widget_width * 0.15;
        let click_offset_y = rng.r#gen::<f64>() * widget_height * 0.3 - widget_height * 0.15;

        (center_x + click_offset_x, center_y + click_offset_y)
    }

    /// Generate random delays between phases
    fn generate_phase_delays(&self, rng: &mut impl Rng) -> Vec<Duration> {
        if !self.config.add_micro_delays {
            return vec![];
        }

        // Generate delays between scan movements and before main interaction
        let num_delays = self.config.scan_movements + 1;
        (0..num_delays)
            .map(|_| {
                let delay_ms = rng.gen_range(
                    self.config.min_action_delay_ms..=self.config.max_action_delay_ms
                );
                Duration::from_millis(delay_ms)
            })
            .collect()
    }

    /// Convert an interaction result to JavaScript code that can be executed
    pub fn to_javascript(&self, result: &InteractionResult) -> String {
        let mut js_parts = Vec::new();

        // Add phase 1: scanning movements
        for (i, scan) in result.scan_movements.iter().enumerate() {
            if i < result.phase_delays.len() {
                let delay = result.phase_delays[i].as_millis();
                js_parts.push(format!("await new Promise(r => setTimeout(r, {}));", delay));
            }

            js_parts.push(self.event_sequence_to_js(scan));
        }

        // Add final delay before main interaction
        if let Some(delay) = result.phase_delays.last() {
            js_parts.push(format!("await new Promise(r => setTimeout(r, {}));", delay.as_millis()));
        }

        // Add phase 2: main interaction
        js_parts.push(self.event_sequence_to_js(&result.event_sequence));

        // Wrap in async IIFE since we use await
        format!("(async function() {{\n{}\n}})()", js_parts.join("\n"))
    }

    /// Convert an event sequence to JavaScript
    fn event_sequence_to_js(&self, sequence: &EventSequence) -> String {
        let mut js_lines = Vec::new();

        for event in &sequence.events {
            // Add delay if specified
            if event.delay.as_millis() > 0 {
                js_lines.push(format!(
                    "await new Promise(r => setTimeout(r, {}));",
                    event.delay.as_millis()
                ));
            }

            // Generate event dispatch code
            js_lines.push(self.event_action_to_js(event));
        }

        js_lines.join("\n")
    }

    /// Convert a single event action to JavaScript
    fn event_action_to_js(&self, event: &EventAction) -> String {
        let event_type = event.event_type.as_str();

        format!(
            r#"(function() {{
    var target = document.elementFromPoint({}, {}) || document.body;
    var event = new MouseEvent('{}', {{
        bubbles: {},
        cancelable: {},
        view: window,
        detail: 1,
        screenX: {},
        screenY: {},
        clientX: {},
        clientY: {},
        movementX: {},
        movementY: {},
        button: {},
        buttons: {},
        ctrlKey: false,
        shiftKey: false,
        altKey: false,
        metaKey: false
    }});
    target.dispatchEvent(event);
}})()"#,
            event.coords.client_x,
            event.coords.client_y,
            event_type,
            event.event_type.bubbles(),
            event.event_type.cancelable(),
            event.coords.screen_x,
            event.coords.screen_y,
            event.coords.client_x,
            event.coords.client_y,
            event.movement.0,
            event.movement.1,
            event.button as i16,
            event.buttons,
        )
    }
}

impl Default for BehavioralSimulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::solver::detector::ChallengeType;

    fn create_test_challenge(requires_interaction: bool) -> DetectedChallenge {
        DetectedChallenge {
            challenge_type: if requires_interaction {
                ChallengeType::CloudflareTurnstile
            } else {
                ChallengeType::CloudflareJS
            },
            confidence: 0.9,
            widget_selector: if requires_interaction {
                Some(".cf-turnstile".to_string())
            } else {
                None
            },
            click_target_selector: if requires_interaction {
                Some("input[type='checkbox']".to_string())
            } else {
                None
            },
            requires_interaction,
            expected_resolution_time_ms: 5000,
            detected_markers: vec!["test".to_string()],
        }
    }

    #[test]
    fn test_behavioral_simulator_creation() {
        let simulator = BehavioralSimulator::new();
        assert!(simulator.config.simulate_page_scan);
        assert!(simulator.config.add_micro_delays);
    }

    #[test]
    fn test_generate_interaction_non_interactive() {
        let simulator = BehavioralSimulator::new();
        let challenge = create_test_challenge(false);

        let result = simulator.generate_interaction(&challenge, 1366.0, 768.0);
        assert!(result.is_ok());

        let interaction = result.unwrap();
        assert!(!interaction.event_sequence.events.is_empty());
        assert!(interaction.click_target.is_none()); // No click for non-interactive
    }

    #[test]
    fn test_generate_interaction_interactive() {
        let simulator = BehavioralSimulator::new();
        let challenge = create_test_challenge(true);

        let result = simulator.generate_interaction(&challenge, 1366.0, 768.0);
        assert!(result.is_ok());

        let interaction = result.unwrap();
        assert!(interaction.click_target.is_some()); // Should have click target

        // Should have click events in sequence
        let has_click = interaction.event_sequence.events.iter()
            .any(|e| e.event_type == MouseEventType::Click);
        assert!(has_click);
    }

    #[test]
    fn test_fitts_law_duration() {
        let simulator = BehavioralSimulator::new();

        // Short distance should be faster
        let short_duration = simulator.fitts_law_duration((0.0, 0.0), (100.0, 0.0));
        let long_duration = simulator.fitts_law_duration((0.0, 0.0), (1000.0, 0.0));

        assert!(short_duration < long_duration);
        assert!(short_duration >= 100); // Minimum
        assert!(long_duration <= 2000); // Maximum
    }

    #[test]
    fn test_to_javascript() {
        let simulator = BehavioralSimulator::new();
        let challenge = create_test_challenge(true);

        let result = simulator.generate_interaction(&challenge, 1366.0, 768.0).unwrap();
        let js = simulator.to_javascript(&result);

        assert!(js.contains("MouseEvent"));
        assert!(js.contains("dispatchEvent"));
        assert!(js.contains("setTimeout")); // Should have delays
    }

    #[test]
    fn test_scan_movements_generated() {
        let simulator = BehavioralSimulator::new();
        let challenge = create_test_challenge(true);

        let result = simulator.generate_interaction(&challenge, 1366.0, 768.0).unwrap();

        // Should have scan movements when enabled
        assert!(!result.scan_movements.is_empty());
        assert_eq!(result.scan_movements.len(), simulator.config.scan_movements);
    }
}
