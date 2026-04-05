//! Event APIs - Event, EventTarget, MessageEvent, UI Events

pub mod abort_signal;
pub mod close_event;
pub mod custom_event;
pub mod error_event;
pub mod event;
pub mod event_target;
pub mod fetch_event;
pub mod hash_change_event;
pub mod message_event;
pub mod pageswap_event;
pub mod pointer_event;
pub mod pop_state_event;
pub mod progress_event;
pub mod touch_event;
pub mod ui_events;

#[cfg(test)]
mod tests;
