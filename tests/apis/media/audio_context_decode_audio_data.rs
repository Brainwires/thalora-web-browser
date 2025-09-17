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
