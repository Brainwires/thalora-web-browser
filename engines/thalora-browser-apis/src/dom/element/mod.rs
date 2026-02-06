//! Element Web API implementation for Boa
//!
//! Real native implementation of Element standard with actual DOM tree functionality
//! https://dom.spec.whatwg.org/#interface-element

mod types;
mod intrinsic;
mod properties;
mod dom_manipulation;
mod layout;
mod query_and_events;
mod scripts;
mod automation;
mod helpers;

pub use types::*;
pub use intrinsic::*;
pub use scripts::execute_script_element;
pub use dom_manipulation::parse_html_elements_with_context;
pub use query_and_events::can_have_shadow_root;
pub(crate) use helpers::*;
