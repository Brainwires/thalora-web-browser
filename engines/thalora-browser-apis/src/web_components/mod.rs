//! Web Components API
//!
//! This module implements the Web Components standard:
//! - CustomElementRegistry (customElements)
//! - HTMLTemplateElement (<template>)

pub mod custom_element_registry;
pub mod html_template_element;

pub use custom_element_registry::CustomElementRegistry;
pub use html_template_element::HTMLTemplateElement;

#[cfg(test)]
mod tests;
