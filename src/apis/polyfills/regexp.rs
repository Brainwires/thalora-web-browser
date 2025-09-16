use boa_engine::{Context, JsResult, JsValue, NativeFunction, js_string};

/// Setup enhanced RegExp implementation with Chrome 136 features
pub fn setup_regexp(context: &mut Context) -> JsResult<()> {
    // Ensure RegExp constructor exists first (it should be built-in)
    let global = context.global_object();
    let regexp_constructor = global.get(js_string!("RegExp"), context)?;

    // If RegExp doesn't exist for some reason, skip
    if regexp_constructor.is_undefined() {
        return Ok(());
    }

    if let Some(regexp_obj) = regexp_constructor.as_object() {
        // Chrome 136: RegExp.escape static method
        let escape_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            let input = args.first().cloned().unwrap_or(JsValue::undefined());
            let input_str = input.to_string(context)?;

            // Escape special regex characters to make them literal
            // Escapes: . * + ? ^ $ { } ( ) | [ ] \
            let escaped = input_str.to_std_string()
                .map_err(|_| boa_engine::JsError::from_native(boa_engine::JsNativeError::typ().with_message("Failed to convert string")))?
                .chars()
                .map(|c| match c {
                    '.' | '*' | '+' | '?' | '^' | '$' | '{' | '}' | '(' | ')' | '|' | '[' | ']' | '\\' => format!("\\{}", c),
                    _ => c.to_string(),
                })
                .collect::<String>();

            Ok(JsValue::from(js_string!(escaped)))
        });

        regexp_obj.set(
            js_string!("escape"),
            JsValue::from(escape_fn.to_js_function(context.realm())),
            false,
            context
        )?;
    }

    Ok(())
}