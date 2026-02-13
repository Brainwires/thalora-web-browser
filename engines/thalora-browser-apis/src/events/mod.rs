//! Event APIs - Event, EventTarget, MessageEvent, UI Events

pub mod event;
pub mod event_target;
pub mod message_event;
pub mod pageswap_event;
pub mod ui_events;
pub mod custom_event;
pub mod error_event;
pub mod progress_event;
pub mod hash_change_event;
pub mod pop_state_event;
pub mod close_event;
pub mod abort_signal;

#[cfg(test)]
mod tests;
