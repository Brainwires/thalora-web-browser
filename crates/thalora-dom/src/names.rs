//! Name validation per the DOM Standard (2025 relaxed rules).

use html5ever::{LocalName, Namespace, Prefix, QualName, ns};

use crate::DomError;

fn is_ascii_ws_nul(c: char) -> bool {
    c.is_ascii_whitespace() || c == '\0'
}

/// <https://dom.spec.whatwg.org/#valid-element-local-name>
pub(crate) fn is_valid_element_local_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if first.is_ascii_alphabetic() {
        !name
            .chars()
            .any(|c| is_ascii_ws_nul(c) || c == '/' || c == '>')
    } else if first == ':' || first == '_' || u32::from(first) >= 0x80 {
        chars.all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | ':' | '_') || u32::from(c) >= 0x80
        })
    } else {
        false
    }
}

/// <https://dom.spec.whatwg.org/#valid-attribute-local-name>
pub(crate) fn is_valid_attribute_local_name(name: &str) -> bool {
    !name.is_empty()
        && !name
            .chars()
            .any(|c| is_ascii_ws_nul(c) || matches!(c, '/' | '=' | '>'))
}

/// <https://dom.spec.whatwg.org/#valid-namespace-prefix>
fn is_valid_namespace_prefix(prefix: &str) -> bool {
    !prefix.is_empty()
        && !prefix
            .chars()
            .any(|c| is_ascii_ws_nul(c) || matches!(c, '/' | '>'))
}

/// <https://dom.spec.whatwg.org/#valid-doctype-name>
pub(crate) fn is_valid_doctype_name(name: &str) -> bool {
    !name.chars().any(|c| is_ascii_ws_nul(c) || c == '>')
}

/// Which local-name rule "validate and extract" applies.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum NameContext {
    Element,
    Attribute,
}

/// <https://dom.spec.whatwg.org/#validate-and-extract>
pub(crate) fn validate_and_extract(
    namespace: Option<&str>,
    qualified_name: &str,
    context: NameContext,
) -> Result<QualName, DomError> {
    let namespace = namespace.filter(|ns| !ns.is_empty());
    let (prefix, local) = match qualified_name.split_once(':') {
        Some((p, l)) => (Some(p), l),
        None => (None, qualified_name),
    };
    if let Some(p) = prefix
        && !is_valid_namespace_prefix(p)
    {
        return Err(DomError::InvalidCharacter);
    }
    let local_ok = match context {
        NameContext::Attribute => is_valid_attribute_local_name(local),
        NameContext::Element => is_valid_element_local_name(local),
    };
    if !local_ok {
        return Err(DomError::InvalidCharacter);
    }
    let ns_url = namespace.map(Namespace::from).unwrap_or(ns!());
    if prefix.is_some() && namespace.is_none() {
        return Err(DomError::Namespace);
    }
    if prefix == Some("xml") && ns_url != ns!(xml) {
        return Err(DomError::Namespace);
    }
    let is_xmlns = qualified_name == "xmlns" || prefix == Some("xmlns");
    if is_xmlns != (ns_url == ns!(xmlns)) {
        return Err(DomError::Namespace);
    }
    Ok(QualName::new(
        prefix.map(Prefix::from),
        ns_url,
        LocalName::from(local),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn element_names() {
        assert!(is_valid_element_local_name("div"));
        assert!(is_valid_element_local_name("my-element"));
        assert!(is_valid_element_local_name("a:b"));
        assert!(!is_valid_element_local_name(""));
        assert!(!is_valid_element_local_name("1abc"));
        assert!(!is_valid_element_local_name("a b"));
        assert!(!is_valid_element_local_name("a>"));
    }

    #[test]
    fn attribute_names() {
        assert!(is_valid_attribute_local_name("data-x"));
        assert!(is_valid_attribute_local_name("@click"));
        assert!(!is_valid_attribute_local_name("a=b"));
        assert!(!is_valid_attribute_local_name(""));
    }

    #[test]
    fn extract_namespaces() {
        let q = validate_and_extract(
            Some("http://www.w3.org/2000/svg"),
            "svg:rect",
            NameContext::Element,
        )
        .unwrap();
        assert_eq!(&*q.local, "rect");
        assert_eq!(q.prefix.as_deref(), Some("svg"));
        assert_eq!(
            validate_and_extract(None, "a:b", NameContext::Element),
            Err(DomError::Namespace)
        );
        assert_eq!(
            validate_and_extract(Some("urn:x"), "xmlns", NameContext::Attribute),
            Err(DomError::Namespace)
        );
        assert!(
            validate_and_extract(
                Some("http://www.w3.org/2000/xmlns/"),
                "xmlns:foo",
                NameContext::Attribute
            )
            .is_ok()
        );
    }
}
