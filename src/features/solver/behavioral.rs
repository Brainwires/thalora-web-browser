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
    /// This is a fallback when we can't find the actual widget
    fn estimate_widget_position(
        &self,
        viewport_width: f64,
        viewport_height: f64,
        _challenge: &DetectedChallenge,
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

    /// Generate JavaScript code that finds the actual Turnstile widget and returns
    /// the coordinates where the checkbox should be clicked.
    ///
    /// For WASM-rendered widgets, the checkbox isn't a DOM element - it's pixels
    /// drawn at a specific location. This function calculates that location.
    ///
    /// Returns JavaScript that sets `window._widgetClickTarget = {x, y, found: bool}`
    pub fn generate_widget_detection_js(challenge: &DetectedChallenge) -> String {
        let widget_selector = challenge.widget_selector.clone()
            .unwrap_or_else(|| ".cf-turnstile, [data-cf-turnstile], iframe[src*='challenges.cloudflare.com']".to_string());

        format!(r#"
(function() {{
    console.log('[Turnstile] Starting widget detection...');
    window._widgetClickTarget = {{ found: false, x: 0, y: 0, width: 0, height: 0, reason: 'not searched' }};

    // Selectors to try for finding the Turnstile widget
    var selectors = '{}';

    // Debug: Log what's in the DOM
    console.log('[Turnstile] DOM body children:', document.body ? document.body.children.length : 'no body');

    // Debug: List all elements with 'cf' or 'turnstile' in class/id
    var allElements = document.querySelectorAll('*');
    var cfElements = [];
    for (var i = 0; i < allElements.length; i++) {{
        var el = allElements[i];
        var id = el.id || '';
        var className = el.className || '';
        if (typeof className !== 'string') className = '';
        if (id.indexOf('cf') !== -1 || id.indexOf('turnstile') !== -1 ||
            className.indexOf('cf') !== -1 || className.indexOf('turnstile') !== -1) {{
            cfElements.push({{
                tag: el.tagName,
                id: id,
                className: className
            }});
        }}
    }}
    console.log('[Turnstile] Found ' + cfElements.length + ' elements with cf/turnstile in id/class:', JSON.stringify(cfElements));

    // Debug: List all iframes
    var allIframes = document.querySelectorAll('iframe');
    console.log('[Turnstile] Found ' + allIframes.length + ' iframes total');
    for (var j = 0; j < allIframes.length; j++) {{
        var iframe = allIframes[j];
        console.log('[Turnstile] iframe[' + j + ']: src=' + (iframe.src || 'empty') + ', id=' + (iframe.id || 'none'));
    }}

    // Try to find the widget container
    var widget = document.querySelector(selectors);
    console.log('[Turnstile] querySelector result for "' + selectors + '":', widget ? widget.tagName : 'null');

    if (!widget) {{
        // Try finding any iframe that might be Turnstile
        var iframes = document.querySelectorAll('iframe');
        for (var i = 0; i < iframes.length; i++) {{
            var src = iframes[i].src || '';
            if (src.indexOf('turnstile') !== -1 || src.indexOf('challenges.cloudflare') !== -1) {{
                widget = iframes[i];
                console.log('[Turnstile] Found Turnstile iframe by src:', src);
                break;
            }}
        }}
    }}

    if (!widget) {{
        // Last resort: look for div with data attributes
        var divs = document.querySelectorAll('div[data-sitekey], div[data-callback]');
        console.log('[Turnstile] Found ' + divs.length + ' divs with data-sitekey or data-callback');
        if (divs.length > 0) {{
            widget = divs[0];
            console.log('[Turnstile] Using first div with data attributes');
        }}
    }}

    if (!widget) {{
        window._widgetClickTarget = {{ found: false, x: 0, y: 0, reason: 'widget not found', cfElements: cfElements.length, totalIframes: allIframes.length }};
        console.log('[Turnstile] Widget detection failed:', window._widgetClickTarget);
        return;
    }}

    // Get the widget's bounding rectangle
    var rect = widget.getBoundingClientRect();
    console.log('[Turnstile] Widget bounding rect:', JSON.stringify(rect));

    if (rect.width === 0 || rect.height === 0) {{
        window._widgetClickTarget = {{ found: false, x: 0, y: 0, reason: 'widget has zero size', tagName: widget.tagName }};
        console.log('[Turnstile] Widget has zero size');
        return;
    }}

    // Calculate the checkbox click position
    // Turnstile checkbox is typically:
    // - About 20-30px from the left edge
    // - Vertically centered
    // - The clickable area is roughly a 20x20 circle
    //
    // Widget layout (approximately):
    // [  (checkbox)  |  "Verify you are human"  |  logo  ]
    // |<-- ~25px -->|

    var checkboxOffsetX = 25;  // Distance from left edge to checkbox center
    var checkboxOffsetY = rect.height / 2;  // Vertically centered

    // Add small random jitter to appear human (±3px)
    var jitterX = (Math.random() - 0.5) * 6;
    var jitterY = (Math.random() - 0.5) * 6;

    var clickX = rect.left + checkboxOffsetX + jitterX;
    var clickY = rect.top + checkboxOffsetY + jitterY;

    window._widgetClickTarget = {{
        found: true,
        x: clickX,
        y: clickY,
        widgetLeft: rect.left,
        widgetTop: rect.top,
        width: rect.width,
        height: rect.height,
        tagName: widget.tagName,
        reason: 'success'
    }};

    console.log('[Turnstile] Widget found:', JSON.stringify(window._widgetClickTarget));
}})();
"#, widget_selector)
    }

    /// Generate JavaScript that waits for the Turnstile widget to be ready/interactive
    /// before attempting to click.
    pub fn generate_widget_ready_wait_js() -> String {
        r#"
(async function() {
    console.log('[Turnstile] Waiting for widget to be ready...');

    // Wait for widget to be potentially interactive
    // Turnstile typically needs a moment after rendering to accept clicks

    var maxWait = 5000;  // Max 5 seconds
    var checkInterval = 200;
    var waited = 0;

    while (waited < maxWait) {
        // Check if widget has rendered and is potentially interactive
        var widget = document.querySelector('.cf-turnstile, [data-cf-turnstile], iframe[src*="challenges.cloudflare"]');

        if (widget) {
            console.log('[Turnstile] Widget element found after', waited, 'ms:', widget.tagName);
            var rect = widget.getBoundingClientRect();
            console.log('[Turnstile] Widget rect: width=' + rect.width + ', height=' + rect.height);

            if (rect.width > 0 && rect.height > 0) {
                // Widget has size, give it a bit more time to become interactive
                console.log('[Turnstile] Widget has size, waiting 500ms for interactivity...');
                await new Promise(r => setTimeout(r, 500));
                console.log('[Turnstile] Widget appears ready after', waited + 500, 'ms');
                return true;
            }
        } else {
            // Log what we CAN find for debugging
            if (waited % 1000 === 0) {  // Every second
                var allDivs = document.querySelectorAll('div');
                var iframes = document.querySelectorAll('iframe');
                console.log('[Turnstile] Still waiting... divs=' + allDivs.length + ', iframes=' + iframes.length);
            }
        }

        await new Promise(r => setTimeout(r, checkInterval));
        waited += checkInterval;
    }

    console.log('[Turnstile] Widget ready timeout after', waited, 'ms');

    // Final debug dump
    var finalWidgetCheck = document.querySelector('.cf-turnstile, [data-cf-turnstile], iframe[src*="challenges.cloudflare"]');
    console.log('[Turnstile] Final widget check:', finalWidgetCheck ? finalWidgetCheck.tagName : 'null');

    return false;
})()
"#.to_string()
    }

    /// Generate a click at the detected widget position (or fallback to estimated)
    pub fn generate_widget_click_js(fallback_x: f64, fallback_y: f64) -> String {
        format!(r#"
(function() {{
    var target = window._widgetClickTarget;
    var clickX, clickY;

    if (target && target.found) {{
        clickX = target.x;
        clickY = target.y;
        console.log('[Turnstile] Clicking at detected position:', clickX, clickY);
    }} else {{
        // Fallback to estimated position
        clickX = {};
        clickY = {};
        console.log('[Turnstile] Clicking at fallback position:', clickX, clickY);
    }}

    // Dispatch the click events (mousedown, mouseup, click)
    var eventTypes = ['mousedown', 'mouseup', 'click'];

    eventTypes.forEach(function(eventType) {{
        if (typeof window.__dispatchTrustedMouseEvent === 'function') {{
            window.__dispatchTrustedMouseEvent(eventType, clickX, clickY, {{
                button: 0,
                buttons: eventType === 'mouseup' ? 0 : 1
            }});
        }} else {{
            // Fallback to standard event
            var target = document.elementFromPoint(clickX, clickY) || document.body;
            var event = new MouseEvent(eventType, {{
                bubbles: true,
                cancelable: true,
                view: window,
                clientX: clickX,
                clientY: clickY,
                button: 0,
                buttons: eventType === 'mouseup' ? 0 : 1
            }});
            target.dispatchEvent(event);
        }}
    }});

    return {{ clickedAt: {{ x: clickX, y: clickY }}, usedDetection: target && target.found }};
}})();
"#, fallback_x, fallback_y)
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

    /// Convert an interaction result to JavaScript with smart widget detection.
    ///
    /// This version is specifically designed for WASM-rendered widgets like Turnstile
    /// where the checkbox isn't a DOM element but pixels drawn at a specific location.
    ///
    /// Flow:
    /// 1. Wait for widget to be ready
    /// 2. Detect widget position via DOM measurement
    /// 3. Execute scanning movements
    /// 4. Click at detected position (or fallback)
    pub fn to_javascript_with_widget_detection(
        &self,
        result: &InteractionResult,
        challenge: &DetectedChallenge,
    ) -> String {
        let mut js_parts = Vec::new();

        // Phase 0: Wait for widget to be ready
        js_parts.push("// Phase 0: Wait for widget to be ready".to_string());
        js_parts.push(Self::generate_widget_ready_wait_js());

        // Phase 0.5: Detect widget position
        js_parts.push("// Phase 0.5: Detect widget position".to_string());
        js_parts.push(Self::generate_widget_detection_js(challenge));

        // Phase 1: Scanning movements (build anticipation, look human)
        js_parts.push("// Phase 1: Scanning movements".to_string());
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

        // Phase 2: Move mouse towards widget area (using detected or fallback position)
        // We generate mouse movement towards the detected position
        js_parts.push("// Phase 2: Move towards widget".to_string());
        js_parts.push(r#"
await (async function() {
    var target = window._widgetClickTarget;
    if (target && target.found) {
        // Generate a few mouse movements towards the widget
        var currentX = 100, currentY = 100;  // Approximate current position
        var targetX = target.x, targetY = target.y;

        // Move in 3-4 steps
        var steps = 3 + Math.floor(Math.random() * 2);
        for (var i = 1; i <= steps; i++) {
            var progress = i / steps;
            // Add slight curve (ease-out)
            progress = 1 - Math.pow(1 - progress, 2);

            var x = currentX + (targetX - currentX) * progress + (Math.random() - 0.5) * 10;
            var y = currentY + (targetY - currentY) * progress + (Math.random() - 0.5) * 10;

            if (typeof window.__dispatchTrustedMouseEvent === 'function') {
                window.__dispatchTrustedMouseEvent('mousemove', x, y, { button: 0, buttons: 0 });
            }

            await new Promise(r => setTimeout(r, 30 + Math.random() * 50));
        }
    }
})();
"#.to_string());

        // Brief pause before click (human hesitation)
        js_parts.push("await new Promise(r => setTimeout(r, 100 + Math.random() * 200));".to_string());

        // Phase 3: Click at detected position
        js_parts.push("// Phase 3: Click at widget".to_string());
        let fallback = result.click_target.unwrap_or((200.0, 200.0));
        js_parts.push(Self::generate_widget_click_js(fallback.0, fallback.1));

        // Wrap in async IIFE
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
    /// Uses __dispatchTrustedMouseEvent for isTrusted: true events
    fn event_action_to_js(&self, event: &EventAction) -> String {
        let event_type = event.event_type.as_str();

        // Use the native trusted event dispatcher if available, fall back to JS MouseEvent
        format!(
            r#"(function() {{
    // Try to use trusted event dispatcher (isTrusted: true)
    if (typeof window.__dispatchTrustedMouseEvent === 'function') {{
        window.__dispatchTrustedMouseEvent('{}', {}, {}, {{
            button: {},
            buttons: {}
        }});
    }} else {{
        // Fallback to standard MouseEvent (isTrusted: false)
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
    }}
}})()"#,
            event_type,
            event.coords.client_x,
            event.coords.client_y,
            event.button as i16,
            event.buttons,
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
