use anyhow::Result;
use serde::{Deserialize, Serialize};
use thalora_dom::{Document, NodeId};
use url::Url;

/// Information about a form that may open new windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormInfo {
    /// CSS selector to identify this form
    pub selector: String,
    /// Form action URL (where it submits to)
    pub action: String,
    /// Form target attribute (e.g., "_blank", "_self")
    pub target: String,
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Submit button selectors within this form
    pub submit_buttons: Vec<String>,
    /// Whether this form will likely open a new window/tab
    pub opens_new_window: bool,
    /// Predicted URL for the new window (based on action)
    pub predicted_url: Option<String>,
}

/// Analyzes HTML content to find forms and predict new window behavior
#[derive(Clone)]
pub struct FormAnalyzer {
    /// Base URL for resolving relative form actions
    base_url: Option<String>,
}

impl Default for FormAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl FormAnalyzer {
    pub fn new() -> Self {
        Self { base_url: None }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }

    /// Analyze a page's document and extract form information
    pub fn analyze_forms(&self, document: &Document) -> Result<Vec<FormInfo>> {
        let mut forms = Vec::new();

        for (index, form_element) in document
            .query_selector_all(document.document(), "form")?
            .into_iter()
            .enumerate()
        {
            let form_info = self.extract_form_info(document, form_element, index)?;
            forms.push(form_info);
        }

        Ok(forms)
    }

    /// Extract detailed information from a single form element
    fn extract_form_info(
        &self,
        document: &Document,
        form_element: NodeId,
        form_index: usize,
    ) -> Result<FormInfo> {
        let form = document.element_data(form_element)?;

        // Extract form attributes
        let action = form.attr("action").unwrap_or("").to_string();

        let target = form.attr("target").unwrap_or("_self").to_string();

        let method = form.attr("method").unwrap_or("GET").to_uppercase();

        // Create a selector for this specific form
        let selector = if let Some(id) = form.attr("id") {
            format!("form#{}", id)
        } else if let Some(name) = form.attr("name") {
            format!("form[name='{}']", name)
        } else {
            format!("form:nth-of-type({})", form_index + 1)
        };

        // Find submit buttons within this form
        let submit_buttons = self.find_submit_buttons(document, form_element)?;

        // Determine if this form opens a new window
        let opens_new_window = target == "_blank" || target == "_new";

        // Predict the result URL if form opens new window
        let predicted_url = if opens_new_window && !action.is_empty() {
            self.resolve_url(&action)?
        } else {
            None
        };

        Ok(FormInfo {
            selector,
            action,
            target,
            method,
            submit_buttons,
            opens_new_window,
            predicted_url,
        })
    }

    /// Find all submit buttons within a form
    fn find_submit_buttons(
        &self,
        document: &Document,
        form_element: NodeId,
    ) -> Result<Vec<String>> {
        let mut buttons = Vec::new();

        let submit_buttons = document.query_selector_all(
            form_element,
            r#"input[type="submit"], button[type="submit"], button:not([type])"#,
        )?;
        for (index, button) in submit_buttons.into_iter().enumerate() {
            let button_element = document.element_data(button)?;
            let button_selector = if let Some(id) = button_element.attr("id") {
                format!("#{}", id)
            } else if let Some(name) = button_element.attr("name") {
                format!(r#"input[name="{}"]"#, name)
            } else {
                // Create a selector based on button position and type
                let tag_name = button_element.local_name();
                let type_attr = button_element.attr("type").unwrap_or("submit");
                format!(
                    r#"{}[type="{}"]:nth-of-type({})"#,
                    tag_name,
                    type_attr,
                    index + 1
                )
            };
            buttons.push(button_selector);
        }

        Ok(buttons)
    }

    /// Resolve relative URLs to absolute URLs
    fn resolve_url(&self, url: &str) -> Result<Option<String>> {
        if url.is_empty() {
            return Ok(None);
        }

        // If URL is already absolute, return it
        if url.starts_with("http://") || url.starts_with("https://") {
            return Ok(Some(url.to_string()));
        }

        // If we have a base URL, resolve relative URLs
        if let Some(ref base) = self.base_url {
            match Url::parse(base) {
                Ok(base_url) => {
                    match base_url.join(url) {
                        Ok(resolved) => Ok(Some(resolved.to_string())),
                        Err(_) => Ok(Some(url.to_string())), // Fallback to original URL
                    }
                }
                Err(_) => Ok(Some(url.to_string())), // Fallback if base URL is invalid
            }
        } else {
            Ok(Some(url.to_string())) // Return as-is if no base URL
        }
    }

    /// Get forms that will open new windows
    pub fn get_new_window_forms(&self, document: &Document) -> Result<Vec<FormInfo>> {
        let all_forms = self.analyze_forms(document)?;
        Ok(all_forms
            .into_iter()
            .filter(|form| form.opens_new_window)
            .collect())
    }

    /// Find form information by submit button selector
    pub fn find_form_by_submit_button(
        &self,
        document: &Document,
        button_selector: &str,
    ) -> Result<Option<FormInfo>> {
        let all_forms = self.analyze_forms(document)?;

        for form in all_forms {
            for submit_button in &form.submit_buttons {
                if submit_button == button_selector
                    || button_selector.contains(&submit_button.replace(r#"""#, ""))
                {
                    return Ok(Some(form));
                }
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_target_blank_form() {
        let html = r#"
        <html>
        <body>
            <form action="formtest_result.asp" method="post" target="_blank">
                <input type="text" name="scriptaddress" size="50">
                <input type="submit" name="continue" value="Test your web form">
            </form>
        </body>
        </html>
        "#;

        let analyzer = FormAnalyzer::new()
            .with_base_url("https://www.twologs.com/en/resources/formtest.asp".to_string());
        let forms = analyzer.analyze_forms(&Document::parse_html(html)).unwrap();

        assert_eq!(forms.len(), 1);
        let form = &forms[0];
        assert_eq!(form.action, "formtest_result.asp");
        assert_eq!(form.target, "_blank");
        assert_eq!(form.method, "POST");
        assert!(form.opens_new_window);
        assert!(form.predicted_url.is_some());
        assert_eq!(
            form.predicted_url.as_ref().unwrap(),
            "https://www.twologs.com/en/resources/formtest_result.asp"
        );
    }

    #[test]
    fn test_find_form_by_submit_button() {
        let html = r#"
        <form action="test.asp" target="_blank">
            <input type="submit" name="continue" value="Submit">
        </form>
        "#;

        let analyzer = FormAnalyzer::new();
        let form = analyzer
            .find_form_by_submit_button(&Document::parse_html(html), r#"input[name="continue"]"#)
            .unwrap();

        assert!(form.is_some());
        let form_info = form.unwrap();
        assert!(form_info.opens_new_window);
        assert_eq!(form_info.action, "test.asp");
    }
}
