use anyhow::{Result, anyhow};
use std::collections::HashMap;

use crate::engine::browser::InteractionResponse;
use crate::engine::js_unavailable;

// Page interaction without a JavaScript engine.
//
// Boa was removed and V8 is not wired in yet, so no events are dispatched
// and no handlers run. What still works is what a browser does natively,
// worked out from the page's static HTML: values entered into named form
// controls are remembered and sent on submit, submit buttons submit their
// form, links navigate, and Enter in a single-line field submits its form.
// Anything whose only effect would be to run page script fails with the
// "JavaScript execution is unavailable" error rather than appearing to work.

impl super::super::HeadlessWebBrowser {
    /// Type text into a form input element identified by CSS selector.
    ///
    /// The value is remembered for the field's `name` and sent when the form
    /// is submitted. Typing replaces the field's value (`clear_first` has no
    /// further effect). Fails if the element is not a named `<input>` or
    /// `<textarea>`: changing anything else needs JavaScript.
    pub async fn type_text_into_element(
        &mut self,
        selector: &str,
        text: &str,
        _clear_first: bool,
    ) -> Result<InteractionResponse> {
        if self.current_content.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }

        let (tag, name) = text_field(&self.current_content, selector)?;
        let Some(name) = name else {
            // An unnamed field is not submitted; only script could read it
            return Err(js_unavailable());
        };
        self.record_filled_value(&name, text);

        Ok(InteractionResponse {
            success: true,
            message: format!("Text entered into {} element: {} = {}", tag, name, text),
            redirect_url: None,
            new_content: None,
        })
    }

    /// Submit a form with the provided field data.
    ///
    /// The submitted entry list follows the browser's behaviour: every
    /// successful control of the form (including hidden inputs such as CSRF
    /// tokens and pre-filled defaults) is sent, overlaid with values entered
    /// via [`type_text_into_element`](Self::type_text_into_element) on this
    /// page and finally with `form_data`.
    pub async fn submit_form(
        &mut self,
        form_selector: &str,
        form_data: HashMap<String, String>,
    ) -> Result<InteractionResponse> {
        if self.current_content.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }
        let form_index = form_index_for_selector(&self.current_content, form_selector)?;
        self.submit_form_at_index(form_index, form_data, None).await
    }

    /// Submit the form that contains the element matching `selector`, with
    /// every field of that form (including values typed on this page).
    pub async fn submit_form_containing(&mut self, selector: &str) -> Result<InteractionResponse> {
        if self.current_content.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }
        let form_index = form_index_containing(&self.current_content, selector)?;
        self.submit_form_at_index(form_index, HashMap::new(), None)
            .await
    }

    /// Submit the `form_index`-th `<form>` of the current page, optionally on
    /// behalf of a submit button `(name, value)`.
    pub(crate) async fn submit_form_at_index(
        &mut self,
        form_index: usize,
        form_data: HashMap<String, String>,
        submitter: Option<(String, String)>,
    ) -> Result<InteractionResponse> {
        let current_url = self
            .current_url
            .clone()
            .ok_or_else(|| anyhow!("No current page loaded"))?;

        let (action, method, mut entries) = {
            let document = scraper::Html::parse_document(&self.current_content);
            let form_element = document
                .select(&FORM_SELECTOR)
                .nth(form_index)
                .ok_or_else(|| anyhow!("Form not found"))?;
            (
                form_element
                    .value()
                    .attr("action")
                    .unwrap_or("")
                    .to_string(),
                form_element
                    .value()
                    .attr("method")
                    .unwrap_or("get")
                    .to_lowercase(),
                collect_form_entries(form_element),
            )
        };

        // Resolve the action against the page URL (empty action = this page).
        let mut form_url = url::Url::parse(&current_url)?.join(&action)?;
        form_url.set_fragment(None);

        // SSRF: a page-controlled form action must not target internal hosts
        crate::engine::security::SsrfProtection::new().is_safe_url(form_url.as_str())?;

        let filled = self.filled_values_for_current_page();
        merge_entries(&mut entries, filled);
        merge_entries(&mut entries, form_data);
        if let Some(submitter) = submitter {
            entries.push(submitter);
        }

        // Submit the form
        let response = if method == "post" {
            self.client
                .post(form_url.as_str())
                .form(&entries)
                .send()
                .await?
        } else {
            // GET replaces the action's query string with the form data
            form_url.set_query(None);
            self.client
                .get(form_url.as_str())
                .query(&entries)
                .send()
                .await?
        };

        let status_code = response.status();
        let final_url = response.url().to_string();
        let content = response.text().await?;

        // Update current content if successful
        if status_code.is_success() {
            self.current_content = content.clone();
            self.current_url = Some(final_url.clone());
            self.filled_values.clear();

            // Report where we ended up if the server redirected us
            let redirect_url = if final_url != form_url.as_str() {
                Some(final_url)
            } else {
                None
            };

            Ok(InteractionResponse {
                success: true,
                message: "Form submitted successfully".to_string(),
                redirect_url,
                new_content: Some(content),
            })
        } else {
            Ok(InteractionResponse {
                success: false,
                message: format!("Form submission failed: {}", status_code),
                redirect_url: None,
                new_content: Some(content),
            })
        }
    }

    /// Perform `action` on the element matching `selector` (optional for
    /// key presses and page scrolling).
    pub async fn perform_action(
        &mut self,
        selector: Option<&str>,
        action: &crate::engine::browser::types::ElementAction,
    ) -> Result<InteractionResponse> {
        use crate::engine::browser::types::ElementAction;

        if self.current_content.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }
        let needs_element = !matches!(
            action,
            ElementAction::PressKey(_) | ElementAction::Scroll(_)
        );
        if needs_element && selector.is_none() {
            return Err(anyhow!("selector or ref is required for this action"));
        }

        match action {
            ElementAction::SelectOption(wanted) => {
                let sel = selector.unwrap_or_default();
                let (name, value, label) = find_option(&self.current_content, sel, wanted)?;
                // An unnamed control is not submitted; only script could read it
                let name = name.ok_or_else(js_unavailable)?;
                self.record_filled_value(&name, &value);
                Ok(action_response(format!("Selected option \"{label}\"")))
            }
            ElementAction::SetChecked(checked) => {
                let sel = selector.unwrap_or_default();
                let (name, value) = checkable_input(&self.current_content, sel)?;
                let name = name.ok_or_else(js_unavailable)?;
                let recorded = if *checked { value.as_str() } else { UNCHECKED };
                self.record_filled_value(&name, recorded);
                Ok(action_response(if *checked {
                    "Checked".to_string()
                } else {
                    "Unchecked".to_string()
                }))
            }
            ElementAction::PressKey(key) => {
                // Implicit submission: Enter in a single-line field submits its form.
                // Every other key press only matters to page script.
                if key == "Enter"
                    && let Some(sel) = selector
                    && is_implicit_submit_target(&self.current_content, sel)
                {
                    let mut resp = self.submit_form_containing(sel).await?;
                    resp.message = format!("Pressed Enter; {}", resp.message);
                    return Ok(resp);
                }
                Err(js_unavailable())
            }
            // Hovering and scrolling only fire events for page script
            ElementAction::Hover | ElementAction::Scroll(_) => Err(js_unavailable()),
        }
    }

    /// Put a secret (e.g. a stored password) into the field matching
    /// `selector`, to be sent when its form is submitted. Unlike
    /// `type_text_into_element` the value is never echoed back in results or
    /// logs.
    pub async fn fill_secret(&mut self, selector: &str, secret: &str) -> Result<()> {
        if self.current_content.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }
        let name = match text_field(&self.current_content, selector) {
            Ok((_, Some(name))) => name,
            _ => return Err(anyhow!("Could not fill the field {}", selector)),
        };
        self.record_filled_value(&name, secret);
        Ok(())
    }

    /// Remember a value typed into the field `name` on the current page.
    pub(crate) fn record_filled_value(&mut self, name: &str, value: &str) {
        if self.filled_values_url != self.current_url {
            self.filled_values.clear();
            self.filled_values_url = self.current_url.clone();
        }
        self.filled_values
            .insert(name.to_string(), value.to_string());
    }

    /// Values typed into fields of the current page (empty after navigation).
    fn filled_values_for_current_page(&self) -> HashMap<String, String> {
        if self.filled_values_url == self.current_url {
            self.filled_values.clone()
        } else {
            HashMap::new()
        }
    }

    /// Click the element matching a CSS selector and perform the click's
    /// default action: submit buttons submit their form, links navigate and
    /// checkboxes/radio buttons change their checked state.
    ///
    /// No click event is dispatched, so clicking an element whose behaviour
    /// comes from a script handler fails with the "JavaScript execution is
    /// unavailable" error.
    pub async fn click_element(&mut self, selector: &str) -> Result<InteractionResponse> {
        if self.current_content.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }

        tracing::debug!("click_element - attempting to click selector: {}", selector);

        if !self.wait_for_element(selector, 0).await? {
            return Err(anyhow!("Element not found: {}", selector));
        }

        match click_default_action(&self.current_content, self.current_url.as_deref(), selector) {
            ClickDefaultAction::SubmitForm {
                form_index,
                submitter,
            } => {
                let mut resp = self
                    .submit_form_at_index(form_index, HashMap::new(), submitter)
                    .await?;
                resp.message = format!("Clicked submit button; {}", resp.message);
                Ok(resp)
            }
            ClickDefaultAction::FollowLink(url) => {
                let content = self.navigate_to_with_js_option(&url, false, true).await?;
                Ok(InteractionResponse {
                    success: true,
                    message: format!("Clicked link; navigated to {}", url),
                    redirect_url: Some(url),
                    new_content: Some(content),
                })
            }
            ClickDefaultAction::None => {
                // Checkboxes toggle and radio buttons become checked
                let Ok((Some(name), value)) = checkable_input(&self.current_content, selector)
                else {
                    return Err(js_unavailable());
                };
                let is_radio = first_match_attr(&self.current_content, selector, "type")
                    .is_some_and(|t| t.eq_ignore_ascii_case("radio"));
                let was_checked = match self.filled_values_for_current_page().get(&name) {
                    Some(recorded) => recorded == &value,
                    None => has_attr(&self.current_content, selector, "checked"),
                };
                let checked = is_radio || !was_checked;
                let recorded = if checked { value.as_str() } else { UNCHECKED };
                self.record_filled_value(&name, recorded);
                Ok(action_response(format!(
                    "Clicked {} element: {}, checked: {}",
                    if is_radio { "radio" } else { "checkbox" },
                    name,
                    checked
                )))
            }
        }
    }
}

static FORM_SELECTOR: std::sync::LazyLock<scraper::Selector> =
    std::sync::LazyLock::new(|| scraper::Selector::parse("form").expect("valid selector"));
static CONTROL_SELECTOR: std::sync::LazyLock<scraper::Selector> = std::sync::LazyLock::new(|| {
    scraper::Selector::parse("input, select, textarea").expect("valid selector")
});
static OPTION_SELECTOR: std::sync::LazyLock<scraper::Selector> =
    std::sync::LazyLock::new(|| scraper::Selector::parse("option").expect("valid selector"));

/// Whether the first element matching `selector` in `html` has attribute `attr`.
fn has_attr(html: &str, selector: &str, attr: &str) -> bool {
    let Ok(selector) = scraper::Selector::parse(selector) else {
        return false;
    };
    scraper::Html::parse_document(html)
        .select(&selector)
        .next()
        .is_some_and(|el| el.value().attr(attr).is_some())
}

/// For an `<input>` or `<textarea>` matching `selector`, return its tag name
/// and (non-empty) `name`. Any other element cannot take typed text without
/// JavaScript.
fn text_field(html: &str, selector: &str) -> Result<(String, Option<String>)> {
    let document = scraper::Html::parse_document(html);
    let parsed = scraper::Selector::parse(selector).map_err(|_| anyhow!("Invalid selector"))?;
    let el = document
        .select(&parsed)
        .next()
        .ok_or_else(|| anyhow!("Element not found: {}", selector))?;
    let v = el.value();
    if !matches!(v.name(), "input" | "textarea") {
        return Err(js_unavailable());
    }
    Ok((
        v.name().to_string(),
        v.attr("name").filter(|n| !n.is_empty()).map(str::to_string),
    ))
}

/// Attribute `attr` of the first element matching `selector` in `html`.
fn first_match_attr(html: &str, selector: &str, attr: &str) -> Option<String> {
    let selector = scraper::Selector::parse(selector).ok()?;
    let document = scraper::Html::parse_document(html);
    document
        .select(&selector)
        .next()?
        .value()
        .attr(attr)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

/// Index (among all `<form>` elements) of the first form matching `selector`.
fn form_index_for_selector(html: &str, selector: &str) -> Result<usize> {
    let document = scraper::Html::parse_document(html);
    let selector =
        scraper::Selector::parse(selector).map_err(|_| anyhow!("Invalid form selector"))?;
    let target = document
        .select(&selector)
        .next()
        .ok_or_else(|| anyhow!("Form not found"))?;
    document
        .select(&FORM_SELECTOR)
        .position(|form| form == target)
        .ok_or_else(|| anyhow!("Form not found: selector does not match a <form> element"))
}

/// Index (among all `<form>` elements) of the form containing the first
/// element matching `selector`.
fn form_index_containing(html: &str, selector: &str) -> Result<usize> {
    let document = scraper::Html::parse_document(html);
    let selector = scraper::Selector::parse(selector).map_err(|_| anyhow!("Invalid selector"))?;
    let target = document
        .select(&selector)
        .next()
        .ok_or_else(|| anyhow!("Element not found"))?;
    let form = target
        .ancestors()
        .filter_map(scraper::ElementRef::wrap)
        .find(|a| a.value().name() == "form")
        .ok_or_else(|| anyhow!("Element is not inside a <form>"))?;
    document
        .select(&FORM_SELECTOR)
        .position(|f| f == form)
        .ok_or_else(|| anyhow!("Form not found"))
}

/// Collect a form's successful controls with their values from the HTML,
/// approximating the HTML spec's "constructing the entry list": disabled
/// controls, buttons and unchecked checkboxes/radios are skipped.
pub(crate) fn collect_form_entries(form: scraper::ElementRef) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    for control in form.select(&CONTROL_SELECTOR) {
        let el = control.value();
        let Some(name) = el.attr("name").filter(|n| !n.is_empty()) else {
            continue;
        };
        if el.attr("disabled").is_some() {
            continue;
        }
        match el.name() {
            "input" => {
                let input_type = el.attr("type").unwrap_or("text").to_ascii_lowercase();
                match input_type.as_str() {
                    "submit" | "image" | "button" | "reset" | "file" => {}
                    "checkbox" | "radio" => {
                        if el.attr("checked").is_some() {
                            let value = el.attr("value").unwrap_or("on");
                            entries.push((name.to_string(), value.to_string()));
                        }
                    }
                    _ => {
                        let value = el.attr("value").unwrap_or("");
                        entries.push((name.to_string(), value.to_string()));
                    }
                }
            }
            "textarea" => entries.push((name.to_string(), control.text().collect())),
            "select" => {
                let options: Vec<_> = control.select(&OPTION_SELECTOR).collect();
                let multiple = el.attr("multiple").is_some();
                let selected: Vec<_> = options
                    .iter()
                    .filter(|o| o.value().attr("selected").is_some())
                    .collect();
                let chosen: Vec<_> = if selected.is_empty() && !multiple {
                    options.first().into_iter().collect()
                } else if multiple {
                    selected
                } else {
                    selected.into_iter().take(1).collect()
                };
                for option in chosen {
                    let value = option
                        .value()
                        .attr("value")
                        .map(str::to_string)
                        .unwrap_or_else(|| option.text().collect::<String>().trim().to_string());
                    entries.push((name.to_string(), value));
                }
            }
            _ => {}
        }
    }
    entries
}

/// Recorded value meaning "this checkbox was unchecked": removes the field
/// from the submitted entries.
const UNCHECKED: &str = "\u{0}thalora:unchecked";

/// Overlay `overrides` onto `entries`: an existing entry with the same name
/// takes the new value, otherwise the pair is appended. An [`UNCHECKED`]
/// value removes the field instead.
pub(crate) fn merge_entries(
    entries: &mut Vec<(String, String)>,
    overrides: impl IntoIterator<Item = (String, String)>,
) {
    for (name, value) in overrides {
        if value == UNCHECKED {
            entries.retain(|(n, _)| *n != name);
            continue;
        }
        if let Some(entry) = entries.iter_mut().find(|(n, _)| *n == name) {
            entry.1 = value;
        } else {
            entries.push((name, value));
        }
    }
}

fn action_response(message: String) -> InteractionResponse {
    InteractionResponse {
        success: true,
        message,
        redirect_url: None,
        new_content: None,
    }
}

/// For a `<select>` matching `selector`, find the option whose value or
/// trimmed text equals `wanted`. Returns (select name, option value, label).
fn find_option(
    html: &str,
    selector: &str,
    wanted: &str,
) -> Result<(Option<String>, String, String)> {
    let document = scraper::Html::parse_document(html);
    let parsed = scraper::Selector::parse(selector).map_err(|_| anyhow!("Invalid selector"))?;
    let select = document
        .select(&parsed)
        .next()
        .ok_or_else(|| anyhow!("Element not found: {}", selector))?;
    if select.value().name() != "select" {
        return Err(anyhow!("Element is not a <select>"));
    }
    let mut available = Vec::new();
    for option in select.select(&OPTION_SELECTOR) {
        let label = option.text().collect::<String>().trim().to_string();
        let value = option
            .value()
            .attr("value")
            .map(str::to_string)
            .unwrap_or_else(|| label.clone());
        if value == wanted || label == wanted {
            let name = select.value().attr("name").map(str::to_string);
            return Ok((name, value, label));
        }
        available.push(label);
    }
    Err(anyhow!(
        "No option matching \"{}\". Options: {}",
        wanted,
        available.join(", ")
    ))
}

/// For a checkbox/radio matching `selector`, return (name, value).
fn checkable_input(html: &str, selector: &str) -> Result<(Option<String>, String)> {
    let document = scraper::Html::parse_document(html);
    let parsed = scraper::Selector::parse(selector).map_err(|_| anyhow!("Invalid selector"))?;
    let input = document
        .select(&parsed)
        .next()
        .ok_or_else(|| anyhow!("Element not found: {}", selector))?;
    let v = input.value();
    let input_type = v.attr("type").unwrap_or("").to_ascii_lowercase();
    if v.name() != "input" || !matches!(input_type.as_str(), "checkbox" | "radio") {
        return Err(anyhow!("Element is not a checkbox or radio button"));
    }
    Ok((
        v.attr("name").filter(|n| !n.is_empty()).map(str::to_string),
        v.attr("value").unwrap_or("on").to_string(),
    ))
}

/// Whether Enter on the element implicitly submits its form (single-line
/// inputs inside a form; not textareas, buttons or checkboxes).
fn is_implicit_submit_target(html: &str, selector: &str) -> bool {
    let Ok(parsed) = scraper::Selector::parse(selector) else {
        return false;
    };
    let document = scraper::Html::parse_document(html);
    let Some(el) = document.select(&parsed).next() else {
        return false;
    };
    let v = el.value();
    let single_line = v.name() == "input"
        && !matches!(
            v.attr("type")
                .unwrap_or("text")
                .to_ascii_lowercase()
                .as_str(),
            "checkbox" | "radio" | "button" | "submit" | "reset" | "image" | "file" | "hidden"
        );
    single_line
        && el
            .ancestors()
            .filter_map(scraper::ElementRef::wrap)
            .any(|a| a.value().name() == "form")
}

/// What a click on an element does by default.
#[derive(Debug, PartialEq)]
pub(crate) enum ClickDefaultAction {
    /// Submit the `form_index`-th form, with the button's name/value if any.
    SubmitForm {
        form_index: usize,
        submitter: Option<(String, String)>,
    },
    /// Navigate to an absolute http(s) URL.
    FollowLink(String),
    None,
}

/// Work out the default action of clicking the first element matching
/// `selector` in `html`: submit buttons submit their form, links navigate.
pub(crate) fn click_default_action(
    html: &str,
    page_url: Option<&str>,
    selector: &str,
) -> ClickDefaultAction {
    let Ok(selector) = scraper::Selector::parse(selector) else {
        return ClickDefaultAction::None;
    };
    let document = scraper::Html::parse_document(html);
    let Some(target) = document.select(&selector).next() else {
        return ClickDefaultAction::None;
    };

    // Walk from the target up through its ancestors: the first submit button
    // or link found decides (e.g. a <span> inside an <a> follows the link).
    let mut node = Some(target);
    while let Some(el) = node {
        let v = el.value();
        let is_submit = match v.name() {
            "button" => v
                .attr("type")
                .is_none_or(|t| t.eq_ignore_ascii_case("submit")),
            "input" => v.attr("type").is_some_and(|t| {
                t.eq_ignore_ascii_case("submit") || t.eq_ignore_ascii_case("image")
            }),
            _ => false,
        };
        if is_submit {
            if v.attr("disabled").is_some() {
                return ClickDefaultAction::None;
            }
            let form = el
                .ancestors()
                .filter_map(scraper::ElementRef::wrap)
                .find(|a| a.value().name() == "form");
            let Some(form) = form else {
                return ClickDefaultAction::None;
            };
            let Some(form_index) = document.select(&FORM_SELECTOR).position(|f| f == form) else {
                return ClickDefaultAction::None;
            };
            let submitter = v
                .attr("name")
                .filter(|n| !n.is_empty())
                .map(|name| (name.to_string(), v.attr("value").unwrap_or("").to_string()));
            return ClickDefaultAction::SubmitForm {
                form_index,
                submitter,
            };
        }
        if v.name() == "a"
            && let Some(href) = v.attr("href")
        {
            let href = href.trim();
            if href.is_empty()
                || href.starts_with('#')
                || href.to_ascii_lowercase().starts_with("javascript:")
            {
                return ClickDefaultAction::None;
            }
            let resolved = match page_url.map(url::Url::parse) {
                Some(Ok(base)) => base.join(href),
                _ => url::Url::parse(href),
            };
            return match resolved {
                Ok(u) if u.scheme() == "http" || u.scheme() == "https" => {
                    ClickDefaultAction::FollowLink(u.to_string())
                }
                _ => ClickDefaultAction::None,
            };
        }
        node = el.parent().and_then(scraper::ElementRef::wrap);
    }
    ClickDefaultAction::None
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOGIN: &str = r##"
        <form id="search" action="/search"><input name="q"></form>
        <form id="login" action="/login" method="post">
            <input type="hidden" name="csrf" value="tok123">
            <input name="user" value="prefilled">
            <input type="password" name="pass">
            <input type="checkbox" name="remember" value="yes" checked>
            <input type="checkbox" name="news">
            <input name="off" value="x" disabled>
            <select name="lang"><option value="en">English</option><option value="fr" selected>French</option></select>
            <textarea name="bio">hello</textarea>
            <button name="action" value="signin" id="go"><span id="label">Sign in</span></button>
            <button type="button" id="noop">Help</button>
        </form>
        <a href="/about" id="about"><b id="bold">About</b></a>
        <a href="javascript:void(0)" id="js">JS</a>
        <a href="#top" id="frag">Top</a>
    "##;

    fn login_form_entries() -> Vec<(String, String)> {
        let doc = scraper::Html::parse_document(LOGIN);
        let form = doc.select(&FORM_SELECTOR).nth(1).unwrap();
        collect_form_entries(form)
    }

    #[test]
    fn collects_successful_controls() {
        let entries = login_form_entries();
        let pairs: Vec<(&str, &str)> = entries
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_str()))
            .collect();
        assert_eq!(
            pairs,
            vec![
                ("csrf", "tok123"),
                ("user", "prefilled"),
                ("pass", ""),
                ("remember", "yes"),
                ("lang", "fr"),
                ("bio", "hello"),
            ]
        );
    }

    #[test]
    fn merge_overrides_and_appends() {
        let mut entries = login_form_entries();
        merge_entries(
            &mut entries,
            [
                ("pass".to_string(), "s3cret".to_string()),
                ("extra".to_string(), "1".to_string()),
            ],
        );
        assert!(entries.contains(&("pass".to_string(), "s3cret".to_string())));
        assert_eq!(
            entries.last().unwrap(),
            &("extra".to_string(), "1".to_string())
        );
        assert!(entries.contains(&("csrf".to_string(), "tok123".to_string())));
    }

    #[test]
    fn form_index_containing_lookup() {
        assert_eq!(form_index_containing(LOGIN, "input[name=pass]").unwrap(), 1);
        assert_eq!(form_index_containing(LOGIN, "input[name=q]").unwrap(), 0);
        assert!(form_index_containing(LOGIN, "#about").is_err());
    }

    #[test]
    fn form_index_lookup() {
        assert_eq!(form_index_for_selector(LOGIN, "#login").unwrap(), 1);
        assert_eq!(form_index_for_selector(LOGIN, "form").unwrap(), 0);
        assert!(form_index_for_selector(LOGIN, "#about").is_err());
        assert!(form_index_for_selector(LOGIN, "#missing").is_err());
    }

    #[test]
    fn click_default_actions() {
        let url = Some("https://example.com/dir/page");
        assert_eq!(
            click_default_action(LOGIN, url, "#label"),
            ClickDefaultAction::SubmitForm {
                form_index: 1,
                submitter: Some(("action".to_string(), "signin".to_string())),
            }
        );
        assert_eq!(
            click_default_action(LOGIN, url, "#noop"),
            ClickDefaultAction::None
        );
        assert_eq!(
            click_default_action(LOGIN, url, "#bold"),
            ClickDefaultAction::FollowLink("https://example.com/about".to_string())
        );
        assert_eq!(
            click_default_action(LOGIN, url, "#js"),
            ClickDefaultAction::None
        );
        assert_eq!(
            click_default_action(LOGIN, url, "#frag"),
            ClickDefaultAction::None
        );
        assert_eq!(
            click_default_action(LOGIN, url, "#missing"),
            ClickDefaultAction::None
        );
    }

    #[test]
    fn unchecked_override_removes_the_field() {
        let mut entries = login_form_entries();
        merge_entries(
            &mut entries,
            [("remember".to_string(), UNCHECKED.to_string())],
        );
        assert!(!entries.iter().any(|(n, _)| n == "remember"));
    }

    #[test]
    fn option_lookup_by_value_or_label() {
        let (name, value, label) = find_option(LOGIN, "select[name=lang]", "French").unwrap();
        assert_eq!(name.as_deref(), Some("lang"));
        assert_eq!(value, "fr");
        assert_eq!(label, "French");
        assert_eq!(
            find_option(LOGIN, "select[name=lang]", "en").unwrap().1,
            "en"
        );
        let err = find_option(LOGIN, "select[name=lang]", "German").unwrap_err();
        assert!(err.to_string().contains("English"), "{err}");
        assert!(find_option(LOGIN, "#about", "x").is_err());
    }

    #[test]
    fn checkable_and_implicit_submit_targets() {
        assert_eq!(
            checkable_input(LOGIN, "input[name=remember]").unwrap(),
            (Some("remember".to_string()), "yes".to_string())
        );
        assert!(checkable_input(LOGIN, "input[name=user]").is_err());
        assert!(is_implicit_submit_target(LOGIN, "input[name=pass]"));
        assert!(!is_implicit_submit_target(LOGIN, "input[name=remember]"));
        assert!(!is_implicit_submit_target(LOGIN, "#about"));
    }

    #[test]
    fn text_fields_need_an_input_or_textarea() {
        assert_eq!(
            text_field(LOGIN, "input[name=pass]").unwrap(),
            ("input".to_string(), Some("pass".to_string()))
        );
        assert_eq!(
            text_field(LOGIN, "textarea").unwrap(),
            ("textarea".to_string(), Some("bio".to_string()))
        );
        let err = text_field(LOGIN, "#about").unwrap_err();
        assert_eq!(err.to_string(), crate::engine::JS_UNAVAILABLE);
        assert!(text_field(LOGIN, "#missing").is_err());
        assert!(has_attr(LOGIN, "input[name=remember]", "checked"));
        assert!(!has_attr(LOGIN, "input[name=news]", "checked"));
    }
}
