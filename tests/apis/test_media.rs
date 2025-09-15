use synaptic::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_audio_context_constructor() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test AudioContext exists
    let result = context.eval(Source::from_bytes("typeof AudioContext")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");

    // Test AudioContext can be instantiated
    let result = context.eval(Source::from_bytes(r#"
        const ctx = new AudioContext();
        ctx.state === "running" &&
        ctx.sampleRate === 44100 &&
        ctx.currentTime === 0 &&
        typeof ctx.destination === "object";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_audio_context_create_oscillator() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test createOscillator method
    let result = context.eval(Source::from_bytes(r#"
        const ctx = new AudioContext();
        const osc = ctx.createOscillator();
        osc.type === "sine" &&
        typeof osc.frequency === "object" &&
        typeof osc.start === "function" &&
        typeof osc.stop === "function" &&
        typeof osc.connect === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_audio_context_create_gain() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test createGain method
    let result = context.eval(Source::from_bytes(r#"
        const ctx = new AudioContext();
        const gain = ctx.createGain();
        gain.gain.value === 1.0 &&
        typeof gain.connect === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_audio_context_decode_audio_data() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test decodeAudioData method returns Promise-like object
    let result = context.eval(Source::from_bytes(r#"
        const ctx = new AudioContext();
        const result = ctx.decodeAudioData();
        typeof result.then === "function" && typeof result.catch === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test decodeAudioData callback receives AudioBuffer
    let result = context.eval(Source::from_bytes(r#"
        const ctx = new AudioContext();
        let bufferReceived = false;
        let bufferLength = 0;
        let sampleRate = 0;
        let channels = 0;

        ctx.decodeAudioData().then(function(buffer) {
            bufferReceived = true;
            bufferLength = buffer.length;
            sampleRate = buffer.sampleRate;
            channels = buffer.numberOfChannels;
        });

        bufferReceived && bufferLength === 44100 && sampleRate === 44100 && channels === 2;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_audio_constructor() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test Audio constructor exists
    let result = context.eval(Source::from_bytes("typeof Audio")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");

    // Test Audio can be instantiated with src
    let result = context.eval(Source::from_bytes(r#"
        const audio = new Audio("test.mp3");
        audio.src === "test.mp3" &&
        audio.currentTime === 0 &&
        audio.duration === 0 &&
        audio.paused === true &&
        audio.volume === 1.0 &&
        audio.muted === false;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test Audio without src
    let result = context.eval(Source::from_bytes(r#"
        const audio = new Audio();
        audio.src === "";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_audio_play_pause_methods() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test audio play and pause methods
    let result = context.eval(Source::from_bytes(r#"
        const audio = new Audio();
        typeof audio.play === "function" &&
        typeof audio.pause === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test play method returns Promise-like object
    let result = context.eval(Source::from_bytes(r#"
        const audio = new Audio();
        const playResult = audio.play();
        typeof playResult.then === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_media_recorder_constructor() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test MediaRecorder exists
    let result = context.eval(Source::from_bytes("typeof MediaRecorder")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");

    // Test MediaRecorder can be instantiated
    let result = context.eval(Source::from_bytes(r#"
        const recorder = new MediaRecorder();
        recorder.state === "inactive" &&
        recorder.mimeType === "video/webm" &&
        typeof recorder.start === "function" &&
        typeof recorder.stop === "function" &&
        typeof recorder.pause === "function" &&
        typeof recorder.resume === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_media_recorder_event_handlers() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test MediaRecorder event handlers
    let result = context.eval(Source::from_bytes(r#"
        const recorder = new MediaRecorder();
        recorder.ondataavailable === null &&
        recorder.onstop === null &&
        recorder.onstart === null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_media_recorder_is_type_supported() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test MediaRecorder.isTypeSupported static method
    let result = context.eval(Source::from_bytes(r#"
        typeof MediaRecorder.isTypeSupported === "function" &&
        MediaRecorder.isTypeSupported("video/webm") === true &&
        MediaRecorder.isTypeSupported("video/mp4") === true;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_navigator_media_devices_get_display_media() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test navigator.mediaDevices.getDisplayMedia exists
    let result = context.eval(Source::from_bytes("typeof navigator.mediaDevices.getDisplayMedia")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");

    // Test getDisplayMedia returns Promise-like object and works
    let result = context.eval(Source::from_bytes(r#"
        let streamReceived = false;
        let streamId = null;
        let isActive = false;

        const result = navigator.mediaDevices.getDisplayMedia();
        result.then(function(stream) {
            streamReceived = true;
            streamId = stream.id;
            isActive = stream.active;
        });

        typeof result.then === "function" &&
        streamReceived &&
        streamId === "screen-capture-stream" &&
        isActive === true;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_speech_synthesis_api() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test speechSynthesis global exists
    let result = context.eval(Source::from_bytes("typeof speechSynthesis")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "object");

    // Test speechSynthesis properties and methods
    let result = context.eval(Source::from_bytes(r#"
        speechSynthesis.speaking === false &&
        speechSynthesis.pending === false &&
        speechSynthesis.paused === false &&
        typeof speechSynthesis.speak === "function" &&
        typeof speechSynthesis.cancel === "function" &&
        typeof speechSynthesis.getVoices === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test getVoices returns array-like object
    let result = context.eval(Source::from_bytes(r#"
        const voices = speechSynthesis.getVoices();
        typeof voices === "object" && voices.length === 0;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_speech_synthesis_utterance() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test SpeechSynthesisUtterance constructor exists
    let result = context.eval(Source::from_bytes("typeof SpeechSynthesisUtterance")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");

    // Test SpeechSynthesisUtterance can be instantiated with text
    let result = context.eval(Source::from_bytes(r#"
        const utterance = new SpeechSynthesisUtterance("Hello, world!");
        utterance.text === "Hello, world!" &&
        utterance.lang === "en-US" &&
        utterance.volume === 1.0 &&
        utterance.rate === 1.0 &&
        utterance.pitch === 1.0;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test SpeechSynthesisUtterance without text
    let result = context.eval(Source::from_bytes(r#"
        const utterance = new SpeechSynthesisUtterance();
        utterance.text === "";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_speech_recognition_api() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test SpeechRecognition exists (both standard and webkit prefixed)
    let result = context.eval(Source::from_bytes(r#"
        typeof SpeechRecognition === "function" &&
        typeof webkitSpeechRecognition === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test SpeechRecognition can be instantiated
    let result = context.eval(Source::from_bytes(r#"
        const recognition = new SpeechRecognition();
        recognition.continuous === false &&
        recognition.interimResults === false &&
        recognition.lang === "en-US" &&
        recognition.maxAlternatives === 1 &&
        typeof recognition.start === "function" &&
        typeof recognition.stop === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test webkit prefixed version works the same
    let result = context.eval(Source::from_bytes(r#"
        const recognition = new webkitSpeechRecognition();
        recognition.continuous === false &&
        recognition.lang === "en-US";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}