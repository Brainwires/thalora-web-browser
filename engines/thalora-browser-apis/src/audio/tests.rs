//! Comprehensive test suite for Audio APIs
//! Tests HTMLAudioElement and AudioContext

use boa_engine::string::JsString;
use boa_engine::{Context, JsValue, Source};

// Helper to initialize context with browser APIs
fn create_test_context() -> Context {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
    context
}

// ============================================================================
// HTMLAudioElement Constructor Tests
// ============================================================================

#[test]
fn test_htmlaudioelement_constructor_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes("typeof HTMLAudioElement"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_htmlaudioelement_constructor_creates_instance() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement();
        audio !== null && audio !== undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlaudioelement_constructor_with_src() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement('test.mp3');
        audio.src === 'test.mp3';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// HTMLAudioElement Properties Tests
// ============================================================================

#[test]
fn test_htmlaudioelement_src_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement();
        audio.src = 'audio.mp3';
        audio.src === 'audio.mp3';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlaudioelement_volume_default() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement();
        audio.volume === 1;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlaudioelement_volume_setter() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement();
        audio.volume = 0.5;
        audio.volume === 0.5;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlaudioelement_muted_default() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement();
        audio.muted === false;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlaudioelement_muted_setter() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement();
        audio.muted = true;
        audio.muted === true;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlaudioelement_loop_default() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement();
        audio.loop === false;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlaudioelement_autoplay_default() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement();
        audio.autoplay === false;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlaudioelement_paused_default() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement();
        audio.paused === true;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlaudioelement_currenttime_default() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement();
        audio.currentTime === 0;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlaudioelement_duration_default() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement();
        Number.isNaN(audio.duration) || audio.duration === 0;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// HTMLAudioElement Methods Tests
// ============================================================================

#[test]
fn test_htmlaudioelement_play_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement();
        typeof audio.play === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlaudioelement_play_returns_promise() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement();
        let result = audio.play();
        result instanceof Promise;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlaudioelement_pause_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement();
        typeof audio.pause === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlaudioelement_load_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement();
        typeof audio.load === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlaudioelement_canplaytype_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement();
        typeof audio.canPlayType === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlaudioelement_canplaytype_returns_string() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let audio = new HTMLAudioElement();
        typeof audio.canPlayType('audio/mpeg') === 'string';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// AudioContext Constructor Tests
// ============================================================================

#[test]
fn test_audiocontext_constructor_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes("typeof AudioContext"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_audiocontext_constructor_creates_instance() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        ctx !== null && ctx !== undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// AudioContext Properties Tests
// ============================================================================

#[test]
fn test_audiocontext_state_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        typeof ctx.state === 'string';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_audiocontext_samplerate_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        typeof ctx.sampleRate === 'number' && ctx.sampleRate > 0;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_audiocontext_currenttime_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        typeof ctx.currentTime === 'number' && ctx.currentTime >= 0;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_audiocontext_destination_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        ctx.destination !== null && ctx.destination !== undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// AudioContext State Methods Tests
// ============================================================================

#[test]
fn test_audiocontext_resume_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        typeof ctx.resume === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_audiocontext_resume_returns_promise() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        ctx.resume() instanceof Promise;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_audiocontext_suspend_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        typeof ctx.suspend === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_audiocontext_close_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        typeof ctx.close === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// AudioContext Node Creation Methods Tests
// ============================================================================

#[test]
fn test_audiocontext_creategain_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        typeof ctx.createGain === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_audiocontext_createoscillator_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        typeof ctx.createOscillator === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_audiocontext_createbuffersource_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        typeof ctx.createBufferSource === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_audiocontext_createanalyser_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        typeof ctx.createAnalyser === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_audiocontext_createbuffer_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        typeof ctx.createBuffer === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// AudioContext Decode Methods Tests
// ============================================================================

#[test]
fn test_audiocontext_decodeaudiodata_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        typeof ctx.decodeAudioData === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// GainNode Tests
// ============================================================================

#[test]
fn test_gainnode_creation() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        let gain = ctx.createGain();
        gain !== null && gain !== undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_gainnode_gain_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        let gain = ctx.createGain();
        gain.gain !== null && gain.gain !== undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_gainnode_connect_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        let gain = ctx.createGain();
        typeof gain.connect === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// OscillatorNode Tests
// ============================================================================

#[test]
fn test_oscillatornode_creation() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        let osc = ctx.createOscillator();
        osc !== null && osc !== undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_oscillatornode_frequency_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        let osc = ctx.createOscillator();
        osc.frequency !== null && osc.frequency !== undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_oscillatornode_type_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        let osc = ctx.createOscillator();
        typeof osc.type === 'string';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_oscillatornode_start_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        let osc = ctx.createOscillator();
        typeof osc.start === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_oscillatornode_stop_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let ctx = new AudioContext();
        let osc = ctx.createOscillator();
        typeof osc.stop === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}
