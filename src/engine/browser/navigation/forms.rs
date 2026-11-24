use anyhow::{Result, anyhow};
use deno_web_page::NavigationRequest;
use std::collections::HashMap;
use thalora_dom::{Document, DomResult, NodeId};

use super::scripts::INTERACTION_SETTLE;
use crate::engine::browser::InteractionResponse;
use crate::engine::js_unavailable;

// Page interaction.
//
// On a page running JavaScript (see `PageDom`) interaction goes through the
// page runtime, as a user's would: a click dispatches the pointer and click
// events and then runs the default action, typing and choosing set the
// control's value and fire `input` and `change`, key presses, hovering and
// scrolling dispatch their events. Navigation and form submission stay with
// the browser: the runtime reports the link or the form a click activated
// (or a script navigated to) and this code fetches it.
//
// On a static page (JavaScript disabled, or the page runtime did not start)
// no events are dispatched and no handlers run. What still works is what a
// browser does natively, on the page's document: typing into named form
// controls, choosing options and checking boxes change the document (static
// mode has no IDL `value`/`checked`/`selected` state, so the `value`,
// `checked` and `selected` content attributes and `<textarea>` text are
// written), form submission builds the entry list from the document, submit
// buttons submit their form, links navigate, and Enter in a single-line
// field submits its form. Anything whose only effect would be to run page
// script fails with the "JavaScript execution is unavailable" error rather
// than appearing to work.

/// Moves the pointer over the element matching the selector.
const HOVER_JS: &str = r#"function (selector) {
  const el = document.querySelector(selector);
  if (!el) return { found: false };
  const Pointer = typeof PointerEvent === "function" ? PointerEvent : MouseEvent;
  const fire = (Ctor, type, bubbles) =>
    el.dispatchEvent(new Ctor(type, { bubbles, cancelable: bubbles, composed: true }));
  fire(Pointer, "pointerover", true);
  fire(Pointer, "pointerenter", false);
  fire(MouseEvent, "mouseover", true);
  fire(MouseEvent, "mouseenter", false);
  fire(Pointer, "pointermove", true);
  fire(MouseEvent, "mousemove", true);
  return { found: true };
}"#;

/// Scrolls an element into view, or the page by `deltaY`, and fires
/// `scroll`. The page runtime has no layout: nothing moves and scroll
/// positions stay 0; the event is what pages (lazy loaders) listen for.
const SCROLL_JS: &str = r#"function (selector, deltaY) {
  if (selector) {
    const el = document.querySelector(selector);
    if (!el) return { found: false };
    if (typeof el.scrollIntoView === "function") el.scrollIntoView();
  } else if (typeof window.scrollBy === "function") {
    window.scrollBy(0, deltaY);
  }
  document.dispatchEvent(new Event("scroll", { bubbles: true }));
  return { found: true };
}"#;

/// Presses a key on the element matching the selector, or on the focused
/// element: `keydown`, `keypress` (printable keys and Enter), the key's
/// default action unless a listener prevented it, `keyup`. Default actions:
/// a printable key or Backspace edits a text field, Enter in a single-line
/// field submits its form (through its default button when it has one),
/// Enter or Space activates a button or link, Space toggles a checkbox.
const PRESS_KEY_JS: &str = r#"function (selector, key) {
  const el = selector
    ? document.querySelector(selector)
    : (document.activeElement || document.body || document.documentElement);
  if (!el) return { found: false };
  if (selector && typeof el.focus === "function") el.focus();
  const named = {
    Enter: 13, Tab: 9, Escape: 27, Backspace: 8, Delete: 46, " ": 32,
    ArrowLeft: 37, ArrowUp: 38, ArrowRight: 39, ArrowDown: 40,
    Home: 36, End: 35, PageUp: 33, PageDown: 34,
  };
  const printable = key.length === 1;
  let code = key;
  let keyCode = named[key] || 0;
  if (key === " ") {
    code = "Space";
  } else if (/^[a-z]$/i.test(key)) {
    code = "Key" + key.toUpperCase();
    keyCode = key.toUpperCase().charCodeAt(0);
  } else if (/^[0-9]$/.test(key)) {
    code = "Digit" + key;
    keyCode = key.charCodeAt(0);
  }
  const init = {
    key, code, keyCode, which: keyCode,
    bubbles: true, cancelable: true, composed: true,
  };
  let proceed = el.dispatchEvent(new KeyboardEvent("keydown", init));
  if (proceed && (printable || key === "Enter")) {
    proceed = el.dispatchEvent(new KeyboardEvent("keypress", init));
  }
  let action = "none";
  if (proceed) {
    const tag = el.localName;
    const type = tag === "input" ? String(el.type || "text").toLowerCase() : "";
    const textual = tag === "textarea" || (tag === "input" &&
      !/^(checkbox|radio|button|submit|reset|image|file|hidden|range|color)$/.test(type));
    const editable = textual && !el.disabled && !el.readOnly;
    const input = (data, inputType) =>
      el.dispatchEvent(new InputEvent("input", { bubbles: true, data, inputType }));
    const clickable = tag === "button" || tag === "a" || tag === "summary" ||
      (tag === "input" && /^(button|submit|reset|image)$/.test(type));
    const checkable = tag === "input" && /^(checkbox|radio)$/.test(type);
    if (editable && printable) {
      el.value = el.value + key;
      input(key, "insertText");
      action = "typed";
    } else if (editable && key === "Backspace") {
      if (el.value.length > 0) {
        el.value = el.value.slice(0, -1);
        input(null, "deleteContentBackward");
      }
      action = "deleted";
    } else if (editable && key === "Enter" && tag === "textarea") {
      el.value = el.value + "\n";
      input(null, "insertLineBreak");
      action = "typed";
    } else if (key === "Enter" && tag === "input" && textual) {
      const form = el.form || el.closest("form");
      if (form) {
        const button = form.querySelector(
          "button:not([type]), button[type=submit], input[type=submit], input[type=image]");
        if (!button) {
          form.requestSubmit();
        } else if (!button.disabled) {
          button.click();
        }
        action = "submit";
      }
    } else if ((clickable && (key === "Enter" || key === " ")) || (checkable && key === " ")) {
      el.click();
      action = "click";
    }
  }
  el.dispatchEvent(new KeyboardEvent("keyup", init));
  return { found: true, prevented: !proceed, action };
}"#;

/// Unchecks a radio button (no click does that) and fires the events.
const UNCHECK_JS: &str = r#"function (selector) {
  const el = document.querySelector(selector);
  if (!el) return { found: false };
  el.checked = false;
  el.dispatchEvent(new Event("input", { bubbles: true }));
  el.dispatchEvent(new Event("change", { bubbles: true }));
  return { found: true };
}"#;

/// Types into a `contenteditable` element.
const EDIT_CONTENT_JS: &str = r#"function (selector, text, clear) {
  const el = document.querySelector(selector);
  if (!el) return { found: false };
  if (typeof el.focus === "function") el.focus();
  el.textContent = clear ? text : el.textContent + text;
  el.dispatchEvent(new InputEvent("input", { bubbles: true, data: text, inputType: "insertText" }));
  return { found: true };
}"#;

/// Submits the n-th `<form>` of the document as its submit button would
/// (the `submit` event fires and can cancel it).
const REQUEST_SUBMIT_JS: &str = r#"function (index) {
  const form = document.querySelectorAll("form")[index];
  if (!form) return { found: false };
  form.requestSubmit();
  return { found: true };
}"#;

impl super::super::HeadlessWebBrowser {
    /// Type text into a form field identified by CSS selector.
    ///
    /// On a page running JavaScript the field's value is set as typing
    /// would (`input` and `change` fire); `<input>`, `<textarea>`,
    /// `<select>` and `contenteditable` elements can be typed into, named or
    /// not, and `clear_first: false` appends to the current value.
    ///
    /// On a static page this sets an `<input>`'s `value` attribute or a
    /// `<textarea>`'s text in the page's document, so it is sent when the
    /// form is submitted; typing replaces the field's value and fails if the
    /// element is not a named `<input>` or `<textarea>` (changing anything
    /// else needs JavaScript).
    pub async fn type_text_into_element(
        &mut self,
        selector: &str,
        text: &str,
        clear_first: bool,
    ) -> Result<InteractionResponse> {
        if self.page.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }
        if self.page.has_js() {
            return self.type_text_with_js(selector, text, clear_first).await;
        }

        let (field, tag, name) = self.page.with_doc(|doc| text_field(doc, selector))?;
        let Some(name) = name else {
            // An unnamed field is not submitted; only script could read it
            return Err(self.js_unavailable());
        };
        set_text_value(self.static_doc()?, field, text)?;
        self.page.clear_secret_value(field);

        Ok(InteractionResponse {
            success: true,
            message: format!("Text entered into {} element: {} = {}", tag, name, text),
            redirect_url: None,
            new_content: None,
        })
    }

    async fn type_text_with_js(
        &mut self,
        selector: &str,
        text: &str,
        clear_first: bool,
    ) -> Result<InteractionResponse> {
        let (field, tag, label, content_editable) = self.page.with_doc(|doc| {
            let el = first_match(doc, selector, "Invalid selector")?
                .ok_or_else(|| anyhow!("Element not found: {}", selector))?;
            let v = doc.element_data(el)?;
            let label = v
                .attr("name")
                .filter(|n| !n.is_empty())
                .or_else(|| v.attr("id").filter(|i| !i.is_empty()))
                .unwrap_or(selector)
                .to_string();
            let content_editable = v
                .attr("contenteditable")
                .is_some_and(|c| !c.eq_ignore_ascii_case("false"));
            Ok::<_, anyhow::Error>((el, v.local_name().to_string(), label, content_editable))
        })?;

        match tag.as_str() {
            "input" | "textarea" | "select" => {
                self.with_js(|js| {
                    let value = if clear_first {
                        text.to_string()
                    } else {
                        let current = js.form_state(field)?.map(|s| s.value).unwrap_or_default();
                        format!("{current}{text}")
                    };
                    js.set_value(field, &value)
                })
                .map_err(|e| not_typable(e, selector))?;
            }
            _ if content_editable => {
                self.call_js(
                    EDIT_CONTENT_JS,
                    serde_json::json!([selector, text, clear_first]),
                )?;
            }
            _ => {
                return Err(anyhow!(
                    "Element is not editable (not an input, textarea, select or contenteditable element): {}",
                    selector
                ));
            }
        }
        self.page.clear_secret_value(field);

        let resp = action_response(format!(
            "Text entered into {} element: {} = {}",
            tag, label, text
        ));
        Ok(self.finish_interaction(resp, true).await)
    }

    /// After an interaction on a page running JavaScript: let the handlers'
    /// follow-up work run (`settle`), perform the navigation the page asked
    /// for, if any, and say so in the response.
    async fn finish_interaction(
        &mut self,
        mut resp: InteractionResponse,
        settle: bool,
    ) -> InteractionResponse {
        if settle {
            self.settle(INTERACTION_SETTLE).await;
        }
        if let Some(content) = self.follow_script_navigations().await {
            let url = self.get_current_url();
            resp.message = format!(
                "{}; the page navigated to {}",
                resp.message,
                url.as_deref().unwrap_or("a new page")
            );
            resp.redirect_url = url;
            resp.new_content = Some(content);
        }
        resp
    }

    /// The document of a static page, for interaction that writes markup.
    fn static_doc(&mut self) -> Result<&mut Document> {
        self.page
            .static_doc_mut()
            .ok_or_else(|| anyhow!("the page runs JavaScript: its document is changed through it"))
    }

    /// Submit a form with the provided field data.
    ///
    /// On a page running JavaScript the fields named in `form_data` are
    /// filled as typing would and the form is submitted as its submit
    /// button would: the `submit` event fires, and if no handler cancels it
    /// the browser sends the form with the page's current values (plus any
    /// `form_data` entry that matched no field). A handler that cancels the
    /// event (a script-driven form) has then done its work; nothing is sent
    /// by the browser.
    ///
    /// On a static page the submitted entry list follows the browser's
    /// behaviour: every successful control of the form (including hidden
    /// inputs such as CSRF tokens and pre-filled defaults), with the values
    /// currently in the page's document (so including values entered via
    /// [`type_text_into_element`](Self::type_text_into_element) and friends),
    /// overlaid with `form_data`.
    pub async fn submit_form(
        &mut self,
        form_selector: &str,
        form_data: HashMap<String, String>,
    ) -> Result<InteractionResponse> {
        if self.page.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }
        let form = self
            .page
            .with_doc(|doc| form_for_selector(doc, form_selector))?;
        self.submit_form_element(form, form_data, None).await
    }

    /// Submit the form that contains the element matching `selector`, with
    /// every field of that form (including values typed on this page).
    pub async fn submit_form_containing(&mut self, selector: &str) -> Result<InteractionResponse> {
        if self.page.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }
        let form = self.page.with_doc(|doc| form_containing(doc, selector))?;
        self.submit_form_element(form, HashMap::new(), None).await
    }

    /// Submit the `<form>` element `form` of the current page, optionally on
    /// behalf of a submit button `(name, value)` (static pages; on a page
    /// running JavaScript the clicked button is part of the submission the
    /// runtime reports).
    pub(crate) async fn submit_form_element(
        &mut self,
        form: NodeId,
        form_data: HashMap<String, String>,
        submitter: Option<(String, String)>,
    ) -> Result<InteractionResponse> {
        if self.page.has_js() {
            return self.submit_form_with_js(form, form_data).await;
        }
        let current_url = self
            .get_current_url()
            .ok_or_else(|| anyhow!("No current page loaded"))?;

        let page = &self.page;
        let (action, method, enctype, mut entries) = page.with_doc(|doc| {
            let form_element = doc
                .as_element(form)
                .filter(|f| f.is_html_named("form"))
                .ok_or_else(|| anyhow!("Form not found"))?;
            Ok::<_, anyhow::Error>((
                form_element.attr("action").unwrap_or("").to_string(),
                form_element.attr("method").unwrap_or("get").to_lowercase(),
                form_element.attr("enctype").unwrap_or("").to_lowercase(),
                collect_form_entries(doc, form, |n| page.secret_value(n).map(str::to_string)),
            ))
        })?;

        // Resolve the action against the page URL (empty action = this page).
        let form_url = url::Url::parse(&current_url)?.join(&action)?;

        merge_entries(&mut entries, form_data);
        if let Some(submitter) = submitter {
            entries.push(submitter);
        }

        self.send_form(form_url, &method, &enctype, entries).await
    }

    /// Submit `form` on a page running JavaScript (see
    /// [`submit_form`](Self::submit_form)).
    async fn submit_form_with_js(
        &mut self,
        form: NodeId,
        form_data: HashMap<String, String>,
    ) -> Result<InteractionResponse> {
        // Fill the named fields; what matches no field is sent as an extra
        // entry.
        let (index, fields) = self.page.with_doc(|doc| {
            if !is_html(doc, form, "form") {
                return Err(anyhow!("Form not found"));
            }
            let index = doc
                .query_selector_all(doc.document(), "form")
                .unwrap_or_default()
                .iter()
                .position(|&f| f == form)
                .ok_or_else(|| anyhow!("Form not found"))?;
            let mut fields: HashMap<String, NodeId> = HashMap::new();
            if let Ok(descendants) = doc.descendant_elements(form) {
                for control in descendants {
                    let Some(el) = doc.as_element(control) else {
                        continue;
                    };
                    let fillable = match &**el.local_name() {
                        "textarea" | "select" => true,
                        "input" => !matches!(
                            input_type(doc, control).as_str(),
                            "checkbox" | "radio" | "submit" | "image" | "button" | "reset" | "file"
                        ),
                        _ => false,
                    };
                    if fillable && let Some(name) = el.attr("name").filter(|n| !n.is_empty()) {
                        fields.entry(name.to_string()).or_insert(control);
                    }
                }
            }
            Ok((index, fields))
        })?;
        let mut extra = Vec::new();
        for (name, value) in form_data {
            match fields.get(&name) {
                // A field a user cannot type into (hidden, disabled, read-only)
                // takes the value as an override of its entry instead
                Some(&field) if self.with_js(|js| js.set_value(field, &value)).is_ok() => {
                    self.page.clear_secret_value(field);
                }
                _ => extra.push((name, value)),
            }
        }

        let found = self.call_js(REQUEST_SUBMIT_JS, serde_json::json!([index]))?;
        if found["found"] != true {
            return Err(anyhow!("Form not found"));
        }
        let resp = match self.page.take_navigation_request() {
            Some(NavigationRequest::Form(mut submission)) => {
                merge_entries(&mut submission.entries, extra);
                self.send_form_submission(submission).await?
            }
            Some(other) => {
                // The submit handler navigated instead
                self.page.push_navigation_request(other);
                action_response("Form submit event dispatched".to_string())
            }
            None => action_response(
                "Form submit event dispatched; a page script handled the submission \
                 (the browser sent nothing)"
                    .to_string(),
            ),
        };
        let navigated = resp.new_content.is_some();
        Ok(self.finish_interaction(resp, !navigated).await)
    }

    /// Send a form: `entries` to `form_url` with `method` (`post`, anything
    /// else is GET) and `enctype`, and make the response the current page.
    pub(super) async fn send_form(
        &mut self,
        mut form_url: url::Url,
        method: &str,
        enctype: &str,
        entries: Vec<(String, String)>,
    ) -> Result<InteractionResponse> {
        form_url.set_fragment(None);

        // SSRF: a page-controlled form action must not target internal hosts
        crate::engine::security::SsrfProtection::new().is_safe_url(form_url.as_str())?;

        // Submit the form
        let response = if method.eq_ignore_ascii_case("post") {
            let request = self.client.post(form_url.as_str());
            if enctype.eq_ignore_ascii_case("multipart/form-data") {
                let mut form = reqwest::multipart::Form::new();
                for (name, value) in entries {
                    form = form.text(name, value);
                }
                request.multipart(form).send().await?
            } else {
                request.form(&entries).send().await?
            }
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

        // Replace the current page if successful
        if status_code.is_success() {
            let scripts_ran = self.load_page(&final_url, &content, false).await;
            let content = if scripts_ran {
                self.page.to_html()
            } else {
                content
            };

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

        if self.page.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }
        let needs_element = !matches!(
            action,
            ElementAction::PressKey(_) | ElementAction::Scroll(_)
        );
        if needs_element && selector.is_none() {
            return Err(anyhow!("selector or ref is required for this action"));
        }
        if self.page.has_js() {
            let resp = self.perform_action_with_js(selector, action)?;
            return Ok(self.finish_interaction(resp, true).await);
        }

        match action {
            ElementAction::SelectOption(wanted) => {
                let sel = selector.unwrap_or_default();
                let found = self.page.with_doc(|doc| find_option(doc, sel, wanted))?;
                // An unnamed control is not submitted; only script could read it
                if found.name.is_none() {
                    return Err(self.js_unavailable());
                }
                select_option(self.static_doc()?, found.select, found.option)?;
                Ok(action_response(format!(
                    "Selected option \"{}\"",
                    found.label
                )))
            }
            ElementAction::SetChecked(checked) => {
                let sel = selector.unwrap_or_default();
                let input = self.page.with_doc(|doc| checkable_input(doc, sel))?;
                if input.name.is_none() {
                    return Err(self.js_unavailable());
                }
                set_checked(self.static_doc()?, input.node, *checked)?;
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
                    && self
                        .page
                        .with_doc(|doc| is_implicit_submit_target(doc, sel))
                {
                    let mut resp = self.submit_form_containing(sel).await?;
                    resp.message = format!("Pressed Enter; {}", resp.message);
                    return Ok(resp);
                }
                Err(self.js_unavailable())
            }
            // Hovering and scrolling only fire events for page script
            ElementAction::Hover | ElementAction::Scroll(_) => Err(self.js_unavailable()),
        }
    }

    /// [`perform_action`](Self::perform_action) on a page running
    /// JavaScript: the action is done through the page runtime, so events
    /// fire and handlers run.
    fn perform_action_with_js(
        &mut self,
        selector: Option<&str>,
        action: &crate::engine::browser::types::ElementAction,
    ) -> Result<InteractionResponse> {
        use crate::engine::browser::types::ElementAction;

        let not_found = || anyhow!("Element not found: {}", selector.unwrap_or_default());
        match action {
            ElementAction::SelectOption(wanted) => {
                let sel = selector.unwrap_or_default();
                let (found, value) = self.page.with_doc(|doc| {
                    let found = find_option(doc, sel, wanted)?;
                    let value = option_value(doc, found.option);
                    Ok::<_, anyhow::Error>((found, value))
                })?;
                self.with_js(|js| js.set_value(found.select, &value))?;
                Ok(action_response(format!(
                    "Selected option \"{}\"",
                    found.label
                )))
            }
            ElementAction::SetChecked(checked) => {
                let sel = selector.unwrap_or_default();
                let input = self.page.with_doc(|doc| checkable_input(doc, sel))?;
                let is_checked = |browser: &mut Self| {
                    browser
                        .with_js(|js| js.form_state(input.node))
                        .map(|state| state.is_some_and(|s| s.checked))
                };
                if is_checked(self)? != *checked {
                    if *checked || !input.is_radio {
                        // As a user would: a click (a radio button unchecks
                        // the others of its group)
                        let outcome = self.with_js(|js| js.click(input.node))?;
                        if let Some(navigation) = outcome.navigation {
                            self.page.push_navigation_request(navigation);
                        }
                    } else {
                        self.call_js(UNCHECK_JS, serde_json::json!([sel]))?;
                    }
                    if is_checked(self)? != *checked {
                        return Ok(InteractionResponse {
                            success: false,
                            message: "The control did not change (it is disabled, or a page \
                                      script prevented the change)"
                                .to_string(),
                            redirect_url: None,
                            new_content: None,
                        });
                    }
                }
                Ok(action_response(if *checked {
                    "Checked".to_string()
                } else {
                    "Unchecked".to_string()
                }))
            }
            ElementAction::PressKey(key) => {
                let result = self.call_js(PRESS_KEY_JS, serde_json::json!([selector, key]))?;
                if result["found"] != true {
                    return Err(not_found());
                }
                let effect = match result["action"].as_str() {
                    _ if result["prevented"] == true => "; a page script handled the key",
                    Some("typed") => "; typed into the field",
                    Some("deleted") => "; deleted the last character",
                    Some("submit") => "; submitted the field's form",
                    Some("click") => "; activated the element",
                    _ => "",
                };
                Ok(action_response(format!("Pressed {key}{effect}")))
            }
            ElementAction::Hover => {
                let result = self.call_js(HOVER_JS, serde_json::json!([selector]))?;
                if result["found"] != true {
                    return Err(not_found());
                }
                Ok(action_response(
                    "Hovered over the element (pointer and mouse events dispatched)".to_string(),
                ))
            }
            ElementAction::Scroll(delta_y) => {
                let result = self.call_js(SCROLL_JS, serde_json::json!([selector, delta_y]))?;
                if result["found"] != true {
                    return Err(not_found());
                }
                // No layout in the page runtime: nothing has a position
                Ok(action_response(
                    "Scroll event dispatched (the page has no layout: scroll positions do not change)"
                        .to_string(),
                ))
            }
        }
    }

    /// Put a secret (e.g. a stored password) into the field matching
    /// `selector`, to be sent when its form is submitted. Unlike
    /// `type_text_into_element` the value is never echoed back in results or
    /// logs, and it is not written to the document or given to page
    /// JavaScript: serialized HTML, snapshots, the accessibility tree and
    /// evaluations never contain it (see
    /// [`PageDom`](crate::engine::browser::PageDom)). A page whose script
    /// reads the field itself (instead of letting the form submit) therefore
    /// sees it empty.
    pub async fn fill_secret(&mut self, selector: &str, secret: &str) -> Result<()> {
        if self.page.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }
        let field = match self.page.with_doc(|doc| text_field(doc, selector)) {
            Ok((field, _, Some(_))) => field,
            _ => return Err(anyhow!("Could not fill the field {}", selector)),
        };
        self.page.set_secret_value(field, secret);
        Ok(())
    }

    /// Click the element matching a CSS selector.
    ///
    /// On a page running JavaScript the click is a user's: pointer, mouse
    /// and `click` events are dispatched, handlers run, and unless one
    /// prevents it the default action follows — checkboxes and radio buttons
    /// change state, a submit button submits its form and a link is
    /// followed (both performed by the browser).
    ///
    /// On a static page only the default action is performed: submit buttons
    /// submit their form, links navigate and checkboxes/radio buttons change
    /// their checked state (in the page's document). No click event is
    /// dispatched, so clicking an element whose behaviour comes from a
    /// script handler fails with the "JavaScript execution is unavailable"
    /// error.
    pub async fn click_element(&mut self, selector: &str) -> Result<InteractionResponse> {
        if self.page.is_empty() {
            return Err(anyhow!("No current page loaded"));
        }

        tracing::debug!("click_element - attempting to click selector: {}", selector);

        if !self.element_exists(selector)? {
            return Err(anyhow!("Element not found: {}", selector));
        }
        if self.page.has_js() {
            return self.click_element_with_js(selector).await;
        }

        let page = &self.page;
        match page.with_doc(|doc| click_default_action(doc, page.url(), selector)) {
            ClickDefaultAction::SubmitForm { form, submitter } => {
                let mut resp = self
                    .submit_form_element(form, HashMap::new(), submitter)
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
                let Ok(input) = self.page.with_doc(|doc| checkable_input(doc, selector)) else {
                    return Err(self.js_unavailable());
                };
                let Some(name) = input.name else {
                    return Err(self.js_unavailable());
                };
                let was_checked = self.page.with_doc(|doc| {
                    doc.as_element(input.node)
                        .is_some_and(|e| e.has_attr("checked"))
                });
                let checked = input.is_radio || !was_checked;
                set_checked(self.static_doc()?, input.node, checked)?;
                Ok(action_response(format!(
                    "Clicked {} element: {}, checked: {}",
                    if input.is_radio { "radio" } else { "checkbox" },
                    name,
                    checked
                )))
            }
        }
    }

    /// [`click_element`](Self::click_element) on a page running JavaScript.
    async fn click_element_with_js(&mut self, selector: &str) -> Result<InteractionResponse> {
        let (node, checkable) = self.page.with_doc(|doc| {
            let node = first_match(doc, selector, "Invalid selector")?
                .ok_or_else(|| anyhow!("Element not found: {}", selector))?;
            Ok::<_, anyhow::Error>((node, checkable_input(doc, selector).ok()))
        })?;
        let outcome = self.with_js(|js| js.click(node))?;

        let resp = match outcome.navigation {
            Some(NavigationRequest::Form(submission)) => {
                let mut resp = self.send_form_submission(submission).await?;
                resp.message = format!("Clicked submit button; {}", resp.message);
                resp
            }
            Some(link @ NavigationRequest::Link { .. }) => {
                match self.perform_navigation_request(link).await? {
                    Some(content) => {
                        let url = self.get_current_url();
                        InteractionResponse {
                            success: true,
                            message: format!(
                                "Clicked link; navigated to {}",
                                url.as_deref().unwrap_or_default()
                            ),
                            redirect_url: url,
                            new_content: Some(content),
                        }
                    }
                    None => action_response(
                        "Clicked link (same-document or non-http(s) target: nothing to load)"
                            .to_string(),
                    ),
                }
            }
            Some(other) => {
                // A reload or a location change: followed below
                self.page.push_navigation_request(other);
                action_response("Clicked element".to_string())
            }
            None => {
                let prevented = if outcome.default_prevented {
                    " (a page script prevented the default action)"
                } else {
                    ""
                };
                match checkable {
                    Some(input) => {
                        let checked = self
                            .with_js(|js| js.form_state(input.node))?
                            .is_some_and(|state| state.checked);
                        action_response(format!(
                            "Clicked {} element: {}, checked: {}{}",
                            if input.is_radio { "radio" } else { "checkbox" },
                            input.name.as_deref().unwrap_or(selector),
                            checked,
                            prevented
                        ))
                    }
                    None => action_response(format!("Clicked element{prevented}")),
                }
            }
        };
        let navigated = resp.new_content.is_some();
        Ok(self.finish_interaction(resp, !navigated).await)
    }
}

/// The error of typing into a control that takes no typed value.
fn not_typable(error: anyhow::Error, selector: &str) -> anyhow::Error {
    use crate::engine::browser::page_js::JsError;
    match error.downcast_ref::<JsError>() {
        Some(JsError::Other(_)) => anyhow!(
            "Element cannot be typed into (disabled, read-only, or not a text field): {}",
            selector
        ),
        _ => error,
    }
}

/// Whether `node` is an HTML element with local name `local`.
fn is_html(doc: &Document, node: NodeId, local: &str) -> bool {
    doc.as_element(node).is_some_and(|e| e.is_html_named(local))
}

/// Attribute `name` of element `node`.
fn attr<'a>(doc: &'a Document, node: NodeId, name: &str) -> Option<&'a str> {
    doc.as_element(node).and_then(|e| e.attr(name))
}

/// The lowercased `type` of an `<input>` (`text` if absent).
fn input_type(doc: &Document, node: NodeId) -> String {
    attr(doc, node, "type")
        .unwrap_or("text")
        .to_ascii_lowercase()
}

/// The nearest `<form>` ancestor of `node` (its form owner in static mode;
/// the `form` content attribute is not honoured).
pub(super) fn form_of(doc: &Document, node: NodeId) -> Option<NodeId> {
    doc.ancestor_elements(node)
        .ok()?
        .find(|&a| is_html(doc, a, "form"))
}

/// The first element matching `selector`, or `invalid` as the error if the
/// selector does not parse.
fn first_match(doc: &Document, selector: &str, invalid: &str) -> Result<Option<NodeId>> {
    doc.query_selector(doc.document(), selector)
        .map_err(|_| anyhow!("{}", invalid))
}

/// For an `<input>` or `<textarea>` matching `selector`, return the node,
/// its tag name and (non-empty) `name`. Any other element cannot take typed
/// text on a static page (nor a secret, on any page).
fn text_field(doc: &Document, selector: &str) -> Result<(NodeId, String, Option<String>)> {
    let el = first_match(doc, selector, "Invalid selector")?
        .ok_or_else(|| anyhow!("Element not found: {}", selector))?;
    let v = doc.element_data(el)?;
    let tag = &**v.local_name();
    if !matches!(tag, "input" | "textarea") {
        return Err(js_unavailable());
    }
    Ok((
        el,
        tag.to_string(),
        v.attr("name").filter(|n| !n.is_empty()).map(str::to_string),
    ))
}

/// The `<form>` element matched by `selector`.
fn form_for_selector(doc: &Document, selector: &str) -> Result<NodeId> {
    let target = first_match(doc, selector, "Invalid form selector")?
        .ok_or_else(|| anyhow!("Form not found"))?;
    if !is_html(doc, target, "form") {
        return Err(anyhow!(
            "Form not found: selector does not match a <form> element"
        ));
    }
    Ok(target)
}

/// The form containing the first element matching `selector`.
fn form_containing(doc: &Document, selector: &str) -> Result<NodeId> {
    let target = first_match(doc, selector, "Invalid selector")?
        .ok_or_else(|| anyhow!("Element not found"))?;
    form_of(doc, target).ok_or_else(|| anyhow!("Element is not inside a <form>"))
}

/// The `<option>` descendants of `select`, in tree order.
fn options_of(doc: &Document, select: NodeId) -> Vec<NodeId> {
    doc.descendant_elements(select)
        .map(|it| it.filter(|&o| is_html(doc, o, "option")).collect())
        .unwrap_or_default()
}

/// An option's value: its `value` attribute, else its trimmed text.
fn option_value(doc: &Document, option: NodeId) -> String {
    attr(doc, option, "value")
        .map(str::to_string)
        .unwrap_or_else(|| option_label(doc, option))
}

/// An option's trimmed text.
fn option_label(doc: &Document, option: NodeId) -> String {
    doc.text_content(option)
        .ok()
        .flatten()
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// Collect a form's successful controls with their values from the
/// document, approximating the HTML spec's "constructing the entry list":
/// disabled controls, buttons and unchecked checkboxes/radios are skipped.
/// `secret` gives a control's secret value (see `PageDom`), which wins
/// over the markup value.
pub(crate) fn collect_form_entries(
    doc: &Document,
    form: NodeId,
    secret: impl Fn(NodeId) -> Option<String>,
) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let Ok(descendants) = doc.descendant_elements(form) else {
        return entries;
    };
    for control in descendants {
        let Some(el) = doc.as_element(control) else {
            continue;
        };
        let tag = &**el.local_name();
        if !el.is_html() || !matches!(tag, "input" | "select" | "textarea") {
            continue;
        }
        let Some(name) = el.attr("name").filter(|n| !n.is_empty()) else {
            continue;
        };
        if el.has_attr("disabled") {
            continue;
        }
        match tag {
            "input" => {
                let input_type = input_type(doc, control);
                match input_type.as_str() {
                    "submit" | "image" | "button" | "reset" | "file" => {}
                    "checkbox" | "radio" => {
                        if el.has_attr("checked") {
                            let value = el.attr("value").unwrap_or("on");
                            entries.push((name.to_string(), value.to_string()));
                        }
                    }
                    _ => {
                        let value = secret(control)
                            .unwrap_or_else(|| el.attr("value").unwrap_or("").to_string());
                        entries.push((name.to_string(), value));
                    }
                }
            }
            "textarea" => {
                let value = secret(control).unwrap_or_else(|| {
                    doc.text_content(control).ok().flatten().unwrap_or_default()
                });
                entries.push((name.to_string(), value));
            }
            "select" => {
                let options = options_of(doc, control);
                let multiple = el.has_attr("multiple");
                let selected: Vec<NodeId> = options
                    .iter()
                    .copied()
                    .filter(|&o| attr(doc, o, "selected").is_some())
                    .collect();
                let chosen: Vec<NodeId> = if selected.is_empty() && !multiple {
                    options.first().copied().into_iter().collect()
                } else if multiple {
                    selected
                } else {
                    selected.into_iter().take(1).collect()
                };
                for option in chosen {
                    entries.push((name.to_string(), option_value(doc, option)));
                }
            }
            _ => {}
        }
    }
    entries
}

/// Overlay `overrides` onto `entries`: an existing entry with the same name
/// takes the new value, otherwise the pair is appended.
pub(crate) fn merge_entries(
    entries: &mut Vec<(String, String)>,
    overrides: impl IntoIterator<Item = (String, String)>,
) {
    for (name, value) in overrides {
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

/// Set the value of a text field in static mode: an `<input>`'s `value`
/// content attribute (there is no separate IDL value without a script
/// engine) or a `<textarea>`'s text.
fn set_text_value(doc: &mut Document, field: NodeId, value: &str) -> DomResult<()> {
    if is_html(doc, field, "textarea") {
        doc.set_text_content(field, value)
    } else {
        doc.set_attribute(field, "value", value)
    }
}

/// Make `option` the selected option of `select` (like setting
/// `select.value`): every other option loses its `selected` attribute.
fn select_option(doc: &mut Document, select: NodeId, option: NodeId) -> DomResult<()> {
    for other in options_of(doc, select) {
        if other != option {
            doc.remove_attribute(other, "selected")?;
        }
    }
    doc.set_attribute(option, "selected", "")
}

/// Check or uncheck a checkbox/radio by its `checked` attribute. Checking a
/// radio button unchecks the others of its group.
fn set_checked(doc: &mut Document, input: NodeId, checked: bool) -> DomResult<()> {
    if !checked {
        doc.remove_attribute(input, "checked")?;
        return Ok(());
    }
    if input_type(doc, input) == "radio" {
        for other in radio_group(doc, input) {
            doc.remove_attribute(other, "checked")?;
        }
    }
    doc.set_attribute(input, "checked", "")
}

/// The other radio buttons in `radio`'s group: same (non-empty) `name`, same
/// form owner (or none).
fn radio_group(doc: &Document, radio: NodeId) -> Vec<NodeId> {
    let Some(name) = attr(doc, radio, "name").filter(|n| !n.is_empty()) else {
        return Vec::new();
    };
    let owner = form_of(doc, radio);
    let scope = owner.unwrap_or_else(|| doc.document());
    let Ok(candidates) = doc.descendant_elements(scope) else {
        return Vec::new();
    };
    candidates
        .filter(|&n| {
            n != radio
                && is_html(doc, n, "input")
                && input_type(doc, n) == "radio"
                && attr(doc, n, "name") == Some(name)
                && form_of(doc, n) == owner
        })
        .collect()
}

/// A `<select>` option chosen by [`find_option`].
#[derive(Debug)]
struct FoundOption {
    select: NodeId,
    option: NodeId,
    /// The select's `name`.
    name: Option<String>,
    /// The option's trimmed text.
    label: String,
}

/// For a `<select>` matching `selector`, find the option whose value or
/// trimmed text equals `wanted`.
fn find_option(doc: &Document, selector: &str, wanted: &str) -> Result<FoundOption> {
    let select = first_match(doc, selector, "Invalid selector")?
        .ok_or_else(|| anyhow!("Element not found: {}", selector))?;
    if !is_html(doc, select, "select") {
        return Err(anyhow!("Element is not a <select>"));
    }
    let mut available = Vec::new();
    for option in options_of(doc, select) {
        let label = option_label(doc, option);
        let value = option_value(doc, option);
        if value == wanted || label == wanted {
            return Ok(FoundOption {
                select,
                option,
                name: attr(doc, select, "name").map(str::to_string),
                label,
            });
        }
        available.push(label);
    }
    Err(anyhow!(
        "No option matching \"{}\". Options: {}",
        wanted,
        available.join(", ")
    ))
}

/// A checkbox or radio button found by [`checkable_input`].
#[derive(Debug, PartialEq)]
struct Checkable {
    node: NodeId,
    /// Its non-empty `name`.
    name: Option<String>,
    is_radio: bool,
}

/// The checkbox/radio matching `selector`.
fn checkable_input(doc: &Document, selector: &str) -> Result<Checkable> {
    let input = first_match(doc, selector, "Invalid selector")?
        .ok_or_else(|| anyhow!("Element not found: {}", selector))?;
    let input_type = attr(doc, input, "type").unwrap_or("").to_ascii_lowercase();
    if !is_html(doc, input, "input") || !matches!(input_type.as_str(), "checkbox" | "radio") {
        return Err(anyhow!("Element is not a checkbox or radio button"));
    }
    Ok(Checkable {
        node: input,
        name: attr(doc, input, "name")
            .filter(|n| !n.is_empty())
            .map(str::to_string),
        is_radio: input_type == "radio",
    })
}

/// Whether Enter on the element implicitly submits its form (single-line
/// inputs inside a form; not textareas, buttons or checkboxes).
fn is_implicit_submit_target(doc: &Document, selector: &str) -> bool {
    let Ok(Some(el)) = first_match(doc, selector, "Invalid selector") else {
        return false;
    };
    let single_line = is_html(doc, el, "input")
        && !matches!(
            input_type(doc, el).as_str(),
            "checkbox" | "radio" | "button" | "submit" | "reset" | "image" | "file" | "hidden"
        );
    single_line && form_of(doc, el).is_some()
}

/// What a click on an element does by default.
#[derive(Debug, PartialEq)]
pub(crate) enum ClickDefaultAction {
    /// Submit the `<form>` element `form`, with the button's name/value if any.
    SubmitForm {
        form: NodeId,
        submitter: Option<(String, String)>,
    },
    /// Navigate to an absolute http(s) URL.
    FollowLink(String),
    None,
}

/// Work out the default action of clicking the first element matching
/// `selector` in `doc`: submit buttons submit their form, links navigate.
pub(crate) fn click_default_action(
    doc: &Document,
    page_url: Option<&str>,
    selector: &str,
) -> ClickDefaultAction {
    let Ok(Some(target)) = first_match(doc, selector, "Invalid selector") else {
        return ClickDefaultAction::None;
    };

    // Walk from the target up through its ancestors: the first submit button
    // or link found decides (e.g. a <span> inside an <a> follows the link).
    let chain = std::iter::once(target).chain(
        doc.ancestor_elements(target)
            .map(|a| a.collect::<Vec<_>>())
            .unwrap_or_default(),
    );
    for el in chain {
        let Some(v) = doc.as_element(el) else {
            break;
        };
        let is_submit = match &**v.local_name() {
            "button" => v
                .attr("type")
                .is_none_or(|t| t.eq_ignore_ascii_case("submit")),
            "input" => v.attr("type").is_some_and(|t| {
                t.eq_ignore_ascii_case("submit") || t.eq_ignore_ascii_case("image")
            }),
            _ => false,
        };
        if is_submit {
            if v.has_attr("disabled") {
                return ClickDefaultAction::None;
            }
            let Some(form) = form_of(doc, el) else {
                return ClickDefaultAction::None;
            };
            let submitter = v
                .attr("name")
                .filter(|n| !n.is_empty())
                .map(|name| (name.to_string(), v.attr("value").unwrap_or("").to_string()));
            return ClickDefaultAction::SubmitForm { form, submitter };
        }
        if &**v.local_name() == "a"
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
    }
    ClickDefaultAction::None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::browser::HeadlessWebBrowser;
    use crate::engine::browser::types::ElementAction;

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
            <input type="radio" name="plan" value="free" checked>
            <input type="radio" name="plan" value="pro" id="pro">
            <input id="anon">
            <button name="action" value="signin" id="go"><span id="label">Sign in</span></button>
            <button type="button" id="noop">Help</button>
        </form>
        <input type="radio" name="plan" value="outside" id="outside" checked>
        <a href="/about" id="about"><b id="bold">About</b></a>
        <a href="javascript:void(0)" id="js">JS</a>
        <a href="#top" id="frag">Top</a>
    "##;

    fn doc() -> Document {
        Document::parse_html(LOGIN)
    }

    fn form(doc: &Document, n: usize) -> NodeId {
        doc.query_selector_all(doc.document(), "form").unwrap()[n]
    }

    fn entries_of(doc: &Document, n: usize) -> Vec<(String, String)> {
        collect_form_entries(doc, form(doc, n), |_| None)
    }

    fn pairs(entries: &[(String, String)]) -> Vec<(&str, &str)> {
        entries
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_str()))
            .collect()
    }

    /// A browser with `LOGIN` loaded as its current page (no network).
    fn browser() -> HeadlessWebBrowser {
        let mut browser = std::rc::Rc::try_unwrap(HeadlessWebBrowser::new())
            .ok()
            .expect("sole owner")
            .into_inner()
            .unwrap();
        browser
            .page
            .load(Some("https://example.com/dir/page".to_string()), LOGIN);
        browser
    }

    fn login_entries(b: &HeadlessWebBrowser) -> Vec<(String, String)> {
        b.page.with_doc(|doc| {
            collect_form_entries(doc, form(doc, 1), |n| {
                b.page.secret_value(n).map(str::to_string)
            })
        })
    }

    #[test]
    fn collects_successful_controls() {
        let doc = doc();
        assert_eq!(
            pairs(&entries_of(&doc, 1)),
            vec![
                ("csrf", "tok123"),
                ("user", "prefilled"),
                ("pass", ""),
                ("remember", "yes"),
                ("lang", "fr"),
                ("bio", "hello"),
                ("plan", "free"),
            ]
        );
    }

    #[test]
    fn merge_overrides_and_appends() {
        let mut entries = entries_of(&doc(), 1);
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
    fn form_containing_lookup() {
        let doc = doc();
        assert_eq!(
            form_containing(&doc, "input[name=pass]").unwrap(),
            form(&doc, 1)
        );
        assert_eq!(
            form_containing(&doc, "input[name=q]").unwrap(),
            form(&doc, 0)
        );
        assert!(form_containing(&doc, "#about").is_err());
    }

    #[test]
    fn form_lookup() {
        let doc = doc();
        assert_eq!(form_for_selector(&doc, "#login").unwrap(), form(&doc, 1));
        assert_eq!(form_for_selector(&doc, "form").unwrap(), form(&doc, 0));
        assert!(form_for_selector(&doc, "#about").is_err());
        assert!(form_for_selector(&doc, "#missing").is_err());
    }

    #[test]
    fn click_default_actions() {
        let doc = doc();
        let url = Some("https://example.com/dir/page");
        assert_eq!(
            click_default_action(&doc, url, "#label"),
            ClickDefaultAction::SubmitForm {
                form: form(&doc, 1),
                submitter: Some(("action".to_string(), "signin".to_string())),
            }
        );
        assert_eq!(
            click_default_action(&doc, url, "#noop"),
            ClickDefaultAction::None
        );
        assert_eq!(
            click_default_action(&doc, url, "#bold"),
            ClickDefaultAction::FollowLink("https://example.com/about".to_string())
        );
        assert_eq!(
            click_default_action(&doc, url, "#js"),
            ClickDefaultAction::None
        );
        assert_eq!(
            click_default_action(&doc, url, "#frag"),
            ClickDefaultAction::None
        );
        assert_eq!(
            click_default_action(&doc, url, "#missing"),
            ClickDefaultAction::None
        );
    }

    #[test]
    fn option_lookup_by_value_or_label() {
        let doc = doc();
        let found = find_option(&doc, "select[name=lang]", "French").unwrap();
        assert_eq!(found.name.as_deref(), Some("lang"));
        assert_eq!(option_value(&doc, found.option), "fr");
        assert_eq!(found.label, "French");
        let en = find_option(&doc, "select[name=lang]", "en").unwrap();
        assert_eq!(en.label, "English");
        let err = find_option(&doc, "select[name=lang]", "German").unwrap_err();
        assert!(err.to_string().contains("English"), "{err}");
        assert!(find_option(&doc, "#about", "x").is_err());
    }

    #[test]
    fn checkable_and_implicit_submit_targets() {
        let doc = doc();
        let remember = checkable_input(&doc, "input[name=remember]").unwrap();
        assert_eq!(remember.name.as_deref(), Some("remember"));
        assert_eq!(attr(&doc, remember.node, "value"), Some("yes"));
        assert!(!remember.is_radio);
        assert!(checkable_input(&doc, "#pro").unwrap().is_radio);
        assert!(checkable_input(&doc, "input[name=user]").is_err());
        assert!(is_implicit_submit_target(&doc, "input[name=pass]"));
        assert!(!is_implicit_submit_target(&doc, "input[name=remember]"));
        assert!(!is_implicit_submit_target(&doc, "#about"));
    }

    #[test]
    fn text_fields_need_an_input_or_textarea() {
        let doc = doc();
        let (_, tag, name) = text_field(&doc, "input[name=pass]").unwrap();
        assert_eq!((tag.as_str(), name.as_deref()), ("input", Some("pass")));
        let (_, tag, name) = text_field(&doc, "textarea").unwrap();
        assert_eq!((tag.as_str(), name.as_deref()), ("textarea", Some("bio")));
        let err = text_field(&doc, "#about").unwrap_err();
        assert_eq!(err.to_string(), crate::engine::JS_UNAVAILABLE);
        assert!(text_field(&doc, "#missing").is_err());
    }

    #[test]
    fn radio_groups_are_scoped_to_the_form_owner() {
        let doc = doc();
        let pro = doc.get_element_by_id("pro").unwrap();
        let group = radio_group(&doc, pro);
        // The "free" radio in the same form, not the one outside it
        assert_eq!(group.len(), 1);
        assert_eq!(attr(&doc, group[0], "value"), Some("free"));
    }

    // ----- static form interaction mutates the document -------------------------

    #[tokio::test]
    async fn typing_sets_the_value_attribute_and_textarea_text() {
        let mut b = browser();
        b.type_text_into_element("input[name=user]", "ann", true)
            .await
            .unwrap();
        b.type_text_into_element("textarea", "new bio", true)
            .await
            .unwrap();

        let html = b.get_current_content();
        assert!(
            html.contains(r#"<input name="user" value="ann">"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<textarea name="bio">new bio</textarea>"#),
            "{html}"
        );
        let entries = login_entries(&b);
        assert!(entries.contains(&("user".into(), "ann".into())));
        assert!(entries.contains(&("bio".into(), "new bio".into())));
    }

    #[tokio::test]
    async fn unnamed_fields_still_need_javascript() {
        let mut b = browser();
        let err = b
            .type_text_into_element("#anon", "x", true)
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), crate::engine::JS_UNAVAILABLE);
        assert!(!b.get_current_content().contains(r#"id="anon" value"#));
    }

    #[tokio::test]
    async fn selecting_an_option_moves_the_selected_attribute() {
        let mut b = browser();
        b.perform_action(
            Some("select[name=lang]"),
            &ElementAction::SelectOption("English".into()),
        )
        .await
        .unwrap();
        let html = b.get_current_content();
        assert!(
            html.contains(r#"<option value="en" selected="">English</option>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<option value="fr">French</option>"#),
            "{html}"
        );
        assert!(login_entries(&b).contains(&("lang".into(), "en".into())));
    }

    #[tokio::test]
    async fn checking_and_unchecking_boxes_and_radios() {
        let mut b = browser();
        b.perform_action(
            Some("input[name=remember]"),
            &ElementAction::SetChecked(false),
        )
        .await
        .unwrap();
        b.perform_action(Some("input[name=news]"), &ElementAction::SetChecked(true))
            .await
            .unwrap();
        b.perform_action(Some("#pro"), &ElementAction::SetChecked(true))
            .await
            .unwrap();

        let entries = login_entries(&b);
        assert!(!entries.iter().any(|(n, _)| n == "remember"), "{entries:?}");
        assert!(entries.contains(&("news".into(), "on".into())));
        assert!(entries.contains(&("plan".into(), "pro".into())));
        assert!(!entries.contains(&("plan".into(), "free".into())));

        let html = b.get_current_content();
        assert!(
            html.contains(r#"<input type="checkbox" name="news" checked="">"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<input type="radio" name="plan" value="free">"#),
            "{html}"
        );
        // A radio of the same name outside the form keeps its state
        assert!(
            html.contains(r#"value="outside" id="outside" checked="""#),
            "{html}"
        );
    }

    #[tokio::test]
    async fn clicking_toggles_checkboxes_and_checks_radios() {
        let mut b = browser();
        let resp = b.click_element("input[name=remember]").await.unwrap();
        assert!(resp.message.ends_with("checked: false"), "{}", resp.message);
        let resp = b.click_element("input[name=remember]").await.unwrap();
        assert!(resp.message.ends_with("checked: true"), "{}", resp.message);
        let resp = b.click_element("#pro").await.unwrap();
        assert!(resp.message.ends_with("checked: true"), "{}", resp.message);
        let resp = b.click_element("#pro").await.unwrap();
        assert!(
            resp.message.ends_with("checked: true"),
            "radios stay checked"
        );

        let entries = login_entries(&b);
        assert!(entries.contains(&("remember".into(), "yes".into())));
        assert!(entries.contains(&("plan".into(), "pro".into())));
        assert_eq!(entries.iter().filter(|(n, _)| n == "plan").count(), 1);
    }

    #[tokio::test]
    async fn secrets_are_submitted_but_never_serialized() {
        let mut b = browser();
        b.fill_secret("input[name=pass]", "hunter2").await.unwrap();
        assert!(login_entries(&b).contains(&("pass".into(), "hunter2".into())));
        assert!(!b.get_current_content().contains("hunter2"));
        let snapshot = b.snapshot(Default::default(), None).unwrap().text;
        assert!(!snapshot.contains("hunter2"), "{snapshot}");

        // Typing over the field replaces the secret
        b.type_text_into_element("input[name=pass]", "typed", true)
            .await
            .unwrap();
        assert!(login_entries(&b).contains(&("pass".into(), "typed".into())));
    }

    #[tokio::test]
    async fn filling_keeps_snapshot_refs_valid() {
        let mut b = browser();
        let snapshot = b.snapshot(Default::default(), None).unwrap().text;
        let line = snapshot
            .lines()
            .find(|l| l.contains("textbox") && l.contains("prefilled"))
            .unwrap_or_else(|| panic!("{snapshot}"));
        let r = line.rsplit("[ref=").next().unwrap().trim_end_matches(']');
        let selector = b.resolve_ref(r).unwrap();
        b.type_text_into_element(&selector, "ann", true)
            .await
            .unwrap();
        assert_eq!(b.resolve_ref(r).unwrap(), selector);
        let snapshot = b.snapshot(Default::default(), None).unwrap().text;
        assert!(snapshot.contains(r#"value="ann""#), "{snapshot}");
    }
}
