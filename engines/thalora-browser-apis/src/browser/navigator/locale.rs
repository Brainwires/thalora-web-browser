//! OS locale detection for `navigator.language` / `navigator.languages`.
//!
//! Queries the host OS for its preferred UI locale and returns a
//! BCP-47-formatted primary tag plus a fallback list suitable for
//! the `languages` array. Falls back to `en-US` when detection fails.

/// Detect the OS-preferred BCP-47 locale and derived languages list.
///
/// Returns `(primary, languages)` where `languages` always contains
/// `primary` as its first entry plus the bare language subtag (e.g.
/// `en` for `en-US`) when that differs. The result is always non-empty.
pub fn detect() -> (String, Vec<String>) {
    let primary = sys_locale::get_locale()
        .map(normalize_bcp47)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "en-US".to_string());

    let mut languages = vec![primary.clone()];
    if let Some((base, _)) = primary.split_once('-') {
        if !base.is_empty() && base != primary {
            languages.push(base.to_string());
        }
    }
    (primary, languages)
}

/// Convert a raw OS locale (e.g. `en_US.UTF-8`) into a BCP-47 tag.
///
/// - Strips `.codeset` / `@modifier` suffixes (POSIX form).
/// - Replaces `_` with `-` (IETF form).
fn normalize_bcp47(raw: String) -> String {
    let trimmed = raw
        .split(|c| c == '.' || c == '@')
        .next()
        .unwrap_or(&raw)
        .trim();
    trimmed.replace('_', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_posix_suffixes() {
        assert_eq!(normalize_bcp47("en_US.UTF-8".to_string()), "en-US");
        assert_eq!(normalize_bcp47("de_DE@euro".to_string()), "de-DE");
        assert_eq!(normalize_bcp47("fr-CA".to_string()), "fr-CA");
        assert_eq!(normalize_bcp47("en".to_string()), "en");
    }

    #[test]
    fn detect_returns_non_empty_primary_and_languages() {
        let (primary, languages) = detect();
        assert!(!primary.is_empty());
        assert_eq!(languages[0], primary);
        // Primary must look like a BCP-47 tag: starts with a letter.
        assert!(primary.chars().next().unwrap().is_ascii_alphabetic());
    }

    #[test]
    fn detect_appends_base_when_region_present() {
        // We can't force the OS locale, but we can exercise the logic by
        // asserting the invariant: if primary contains '-', languages has len >= 2.
        let (primary, languages) = detect();
        if primary.contains('-') {
            assert!(languages.len() >= 2);
            assert_eq!(languages[1], primary.split('-').next().unwrap());
        } else {
            assert_eq!(languages.len(), 1);
        }
    }
}
