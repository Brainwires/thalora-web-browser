//! Intl API
//!
//! The Intl API provides language-sensitive string comparison, number formatting,
//! and date/time formatting. This is built into Boa engine with the `intl_bundled`
//! feature enabled.
//!
//! Available Intl constructors (provided by Boa):
//! - Intl.Collator - Language-sensitive string comparison
//! - Intl.DateTimeFormat - Date and time formatting
//! - Intl.DisplayNames - Display names of language, region, and script
//! - Intl.ListFormat - Language-sensitive list formatting
//! - Intl.Locale - Locale identifier
//! - Intl.NumberFormat - Number formatting
//! - Intl.PluralRules - Plural-sensitive formatting
//! - Intl.RelativeTimeFormat - Relative time formatting
//! - Intl.Segmenter - Text segmentation
//!
//! The ICU data for internationalization is bundled with boa_icu_provider.

#[cfg(test)]
mod tests;
