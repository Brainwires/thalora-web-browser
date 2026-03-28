//! Comprehensive test suite for Video APIs
//! Tests HTMLVideoElement

use boa_engine::string::JsString;
use boa_engine::{Context, JsValue, Source};

// Helper to initialize context with browser APIs
fn create_test_context() -> Context {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
    context
}

// ============================================================================
// HTMLVideoElement Constructor Tests
// ============================================================================

#[test]
fn test_htmlvideoelement_constructor_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes("typeof HTMLVideoElement"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_htmlvideoelement_constructor_creates_instance() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        video !== null && video !== undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// HTMLVideoElement Media Properties Tests
// ============================================================================

#[test]
fn test_htmlvideoelement_src_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        video.src = 'video.mp4';
        video.src === 'video.mp4';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_volume_default() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        video.volume === 1;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_volume_setter() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        video.volume = 0.7;
        video.volume === 0.7;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_muted_default() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        video.muted === false;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_loop_default() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        video.loop === false;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_autoplay_default() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        video.autoplay === false;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_controls_default() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        video.controls === false;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// HTMLVideoElement Video-Specific Properties Tests
// ============================================================================

#[test]
fn test_htmlvideoelement_poster_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        video.poster = 'poster.jpg';
        video.poster === 'poster.jpg';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_videowidth_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        typeof video.videoWidth === 'number';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_videoheight_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        typeof video.videoHeight === 'number';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// HTMLVideoElement Playback Properties Tests
// ============================================================================

#[test]
fn test_htmlvideoelement_currenttime_default() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        video.currentTime === 0;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_duration_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        typeof video.duration === 'number';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_playbackrate_default() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        video.playbackRate === 1;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_playbackrate_setter() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        video.playbackRate = 2;
        video.playbackRate === 2;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_paused_default() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        video.paused === true;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_ended_default() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        video.ended === false;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// HTMLVideoElement State Constants Tests
// ============================================================================

#[test]
fn test_htmlvideoelement_have_nothing_constant() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        HTMLVideoElement.HAVE_NOTHING === 0;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_have_metadata_constant() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        HTMLVideoElement.HAVE_METADATA === 1;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_have_current_data_constant() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        HTMLVideoElement.HAVE_CURRENT_DATA === 2;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_have_future_data_constant() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        HTMLVideoElement.HAVE_FUTURE_DATA === 3;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_have_enough_data_constant() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        HTMLVideoElement.HAVE_ENOUGH_DATA === 4;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// HTMLVideoElement Network State Constants Tests
// ============================================================================

#[test]
fn test_htmlvideoelement_network_empty_constant() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        HTMLVideoElement.NETWORK_EMPTY === 0;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_network_idle_constant() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        HTMLVideoElement.NETWORK_IDLE === 1;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_network_loading_constant() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        HTMLVideoElement.NETWORK_LOADING === 2;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_network_no_source_constant() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        HTMLVideoElement.NETWORK_NO_SOURCE === 3;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// HTMLVideoElement Methods Tests
// ============================================================================

#[test]
fn test_htmlvideoelement_play_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        typeof video.play === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_play_returns_promise() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        video.play() instanceof Promise;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_pause_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        typeof video.pause === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_load_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        typeof video.load === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_canplaytype_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        typeof video.canPlayType === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_getvideoplaybackquality_method() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        typeof video.getVideoPlaybackQuality === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_htmlvideoelement_getvideoplaybackquality_returns_object() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let video = new HTMLVideoElement();
        let quality = video.getVideoPlaybackQuality();
        typeof quality === 'object' && quality !== null;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}
