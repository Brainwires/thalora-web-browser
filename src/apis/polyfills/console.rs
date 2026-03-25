use thalora_browser_apis::boa_engine::{
    Context, JsObject, JsResult, JsValue, NativeFunction, js_string, property::Attribute,
};

/// Setup enhanced console implementation with native functions
pub fn setup_console(context: &mut Context) -> JsResult<()> {
    // Create console object
    let console = JsObject::with_object_proto(context.intrinsics());

    // console.log
    let log_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
        let _message = _args.first().cloned().unwrap_or(JsValue::undefined());
        // In real implementation, would forward to Rust logging
        Ok(JsValue::undefined())
    });
    console.set(
        js_string!("log"),
        JsValue::from(log_fn.to_js_function(context.realm())),
        false,
        context,
    )?;

    // console.error
    let error_fn = NativeFunction::from_fn_ptr(|_, args, _context| {
        let _message = args.first().cloned().unwrap_or(JsValue::undefined());
        // In real implementation, would forward to Rust logging
        Ok(JsValue::undefined())
    });
    console.set(
        js_string!("error"),
        JsValue::from(error_fn.to_js_function(context.realm())),
        false,
        context,
    )?;

    // console.warn
    let warn_fn = NativeFunction::from_fn_ptr(|_, args, _context| {
        let _message = args.first().cloned().unwrap_or(JsValue::undefined());
        Ok(JsValue::undefined())
    });
    console.set(
        js_string!("warn"),
        JsValue::from(warn_fn.to_js_function(context.realm())),
        false,
        context,
    )?;

    // console.info
    let info_fn = NativeFunction::from_fn_ptr(|_, args, _context| {
        let _message = args.first().cloned().unwrap_or(JsValue::undefined());
        Ok(JsValue::undefined())
    });
    console.set(
        js_string!("info"),
        JsValue::from(info_fn.to_js_function(context.realm())),
        false,
        context,
    )?;

    // console.debug
    let debug_fn = NativeFunction::from_fn_ptr(|_, args, _context| {
        let _message = args.first().cloned().unwrap_or(JsValue::undefined());
        Ok(JsValue::undefined())
    });
    console.set(
        js_string!("debug"),
        JsValue::from(debug_fn.to_js_function(context.realm())),
        false,
        context,
    )?;

    // console.trace
    let trace_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
        // In real implementation, would provide stack trace
        Ok(JsValue::undefined())
    });
    console.set(
        js_string!("trace"),
        JsValue::from(trace_fn.to_js_function(context.realm())),
        false,
        context,
    )?;

    // Chrome 134: console.timeStamp - Enhanced with options support
    let timestamp_fn = NativeFunction::from_fn_ptr(|_, args, _context| {
        let _label = args.first().cloned().unwrap_or(JsValue::undefined());
        let _options = args.get(1).cloned().unwrap_or(JsValue::undefined());

        // Mock implementation - in real implementation would add performance marker
        // For now, just log the timestamp event
        Ok(JsValue::undefined())
    });
    console.set(
        js_string!("timeStamp"),
        JsValue::from(timestamp_fn.to_js_function(context.realm())),
        false,
        context,
    )?;

    // Register console as global property (proper Boa pattern)
    context.register_global_property(
        js_string!("console"),
        JsValue::from(console),
        Attribute::all(),
    )?;

    Ok(())
}
