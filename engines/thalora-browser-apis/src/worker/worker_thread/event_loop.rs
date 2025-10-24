//! Worker event loop implementation with timer and microtask support
//!
//! Implements the HTML event loop specification for workers:
//! https://html.spec.whatwg.org/multipage/webappapis.html#event-loops
//!
//! This implementation stores only callback IDs, not JsObjects, making it thread-safe.
//! The actual callbacks are stored in the JavaScript Context.

use std::collections::{BinaryHeap, VecDeque};
use std::cmp::Ordering;
use std::time::{Duration, Instant};

/// A scheduled timer (stores only ID and timing info, not the callback)
#[derive(Debug)]
pub struct ScheduledTimer {
    /// Unique timer ID
    pub id: u32,
    /// When the timer should fire
    pub fire_at: Instant,
    /// Timer delay in milliseconds
    pub delay: u32,
    /// Whether this is a repeating timer (setInterval)
    pub repeating: bool,
    /// Callback ID (references the actual callback stored in Context)
    pub callback_id: u32,
}

impl PartialEq for ScheduledTimer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ScheduledTimer {}

impl PartialOrd for ScheduledTimer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledTimer {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (earliest time first)
        other.fire_at.cmp(&self.fire_at)
    }
}

/// A microtask (stores only callback ID)
#[derive(Debug)]
pub struct Microtask {
    /// Callback ID (references the actual callback stored in Context)
    pub callback_id: u32,
}

/// Worker event loop state
pub struct WorkerEventLoop {
    /// Timer queue (min-heap sorted by fire time)
    timer_queue: BinaryHeap<ScheduledTimer>,
    /// Microtask queue (FIFO)
    microtask_queue: VecDeque<Microtask>,
    /// Next timer ID to assign
    next_timer_id: u32,
    /// Next callback ID to assign
    next_callback_id: u32,
}

impl WorkerEventLoop {
    /// Create a new worker event loop
    pub fn new() -> Self {
        Self {
            timer_queue: BinaryHeap::new(),
            microtask_queue: VecDeque::new(),
            next_timer_id: 1,
            next_callback_id: 1,
        }
    }

    /// Register a callback and get its ID
    pub fn register_callback(&mut self) -> u32 {
        let callback_id = self.next_callback_id;
        self.next_callback_id += 1;
        callback_id
    }

    /// Schedule a timer (setTimeout or setInterval)
    pub fn schedule_timer(
        &mut self,
        callback_id: u32,
        delay: u32,
        repeating: bool,
    ) -> u32 {
        let timer_id = self.next_timer_id;
        self.next_timer_id += 1;

        let fire_at = Instant::now() + Duration::from_millis(delay as u64);

        let timer = ScheduledTimer {
            id: timer_id,
            fire_at,
            delay,
            repeating,
            callback_id,
        };

        self.timer_queue.push(timer);
        timer_id
    }

    /// Cancel a timer
    pub fn cancel_timer(&mut self, timer_id: u32) {
        // Remove timer from queue
        self.timer_queue.retain(|timer| timer.id != timer_id);
    }

    /// Queue a microtask
    pub fn queue_microtask(&mut self, callback_id: u32) {
        let microtask = Microtask { callback_id };
        self.microtask_queue.push_back(microtask);
    }

    /// Get ready timers (returns callback IDs that need to be executed)
    pub fn get_ready_timers(&mut self) -> Vec<(u32, bool)> {
        let now = Instant::now();
        let mut ready_callbacks = Vec::new();
        let mut timers_to_requeue = Vec::new();

        // Collect all timers that are ready
        while let Some(timer) = self.timer_queue.peek() {
            if timer.fire_at > now {
                // Timer not ready yet
                break;
            }

            // Remove the timer from the queue
            let mut timer = self.timer_queue.pop().unwrap();
            ready_callbacks.push((timer.callback_id, timer.repeating));

            // If repeating, reschedule it
            if timer.repeating {
                timer.fire_at = Instant::now() + Duration::from_millis(timer.delay as u64);
                timers_to_requeue.push(timer);
            }
        }

        // Re-add repeating timers
        for timer in timers_to_requeue {
            self.timer_queue.push(timer);
        }

        ready_callbacks
    }

    /// Get all pending microtasks (returns callback IDs)
    pub fn drain_microtasks(&mut self) -> Vec<u32> {
        let mut callback_ids = Vec::new();
        while let Some(microtask) = self.microtask_queue.pop_front() {
            callback_ids.push(microtask.callback_id);
        }
        callback_ids
    }

    /// Run one iteration of the event loop (returns IDs to execute)
    ///
    /// Returns (timer_callbacks, microtask_callbacks, next_timer_delay)
    pub fn get_pending_work(&mut self) -> (Vec<(u32, bool)>, Vec<u32>, Option<Duration>) {
        // 1. Get ready timers
        let timer_callbacks = self.get_ready_timers();

        // 2. Get pending microtasks
        let microtask_callbacks = self.drain_microtasks();

        // 3. Calculate time until next timer
        let next_delay = if let Some(next_timer) = self.timer_queue.peek() {
            let now = Instant::now();
            if next_timer.fire_at > now {
                Some(next_timer.fire_at - now)
            } else {
                Some(Duration::from_millis(0))
            }
        } else {
            None
        };

        (timer_callbacks, microtask_callbacks, next_delay)
    }

    /// Check if there are any pending tasks
    pub fn has_pending_tasks(&self) -> bool {
        !self.timer_queue.is_empty() || !self.microtask_queue.is_empty()
    }

    /// Get the number of pending timers
    pub fn pending_timer_count(&self) -> usize {
        self.timer_queue.len()
    }

    /// Get the number of pending microtasks
    pub fn pending_microtask_count(&self) -> usize {
        self.microtask_queue.len()
    }

    /// Clear all timers and microtasks
    pub fn clear_all(&mut self) {
        self.timer_queue.clear();
        self.microtask_queue.clear();
    }
}

impl Default for WorkerEventLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_scheduling() {
        let mut event_loop = WorkerEventLoop::new();

        // Register a callback
        let callback_id = event_loop.register_callback();

        // Schedule a timer
        let timer_id = event_loop.schedule_timer(callback_id, 100, false);
        assert_eq!(timer_id, 1);
        assert_eq!(event_loop.pending_timer_count(), 1);

        // Cancel the timer
        event_loop.cancel_timer(timer_id);
        assert_eq!(event_loop.pending_timer_count(), 0);
    }

    #[test]
    fn test_microtask_queue() {
        let mut event_loop = WorkerEventLoop::new();

        // Queue a microtask
        let callback_id = event_loop.register_callback();
        event_loop.queue_microtask(callback_id);
        assert_eq!(event_loop.pending_microtask_count(), 1);

        // Drain microtasks
        let callbacks = event_loop.drain_microtasks();
        assert_eq!(callbacks.len(), 1);
        assert_eq!(callbacks[0], callback_id);
        assert_eq!(event_loop.pending_microtask_count(), 0);
    }

    #[test]
    fn test_event_loop_iteration() {
        let mut event_loop = WorkerEventLoop::new();

        // Should have no pending tasks
        assert!(!event_loop.has_pending_tasks());

        // Get pending work with no tasks
        let (timers, microtasks, delay) = event_loop.get_pending_work();
        assert!(timers.is_empty());
        assert!(microtasks.is_empty());
        assert!(delay.is_none());
    }
}
