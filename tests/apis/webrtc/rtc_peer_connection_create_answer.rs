#[tokio::test]
async fn test_rtc_peer_connection_create_answer() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");
    // Test createAnswer method exists and returns Promise-like object
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        const answer = pc.createAnswer();
        typeof answer.then === "function" && typeof answer.catch === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
    // Test createAnswer callback receives answer object
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        let answerReceived = false;
        let answerType = null;
        let hasSdp = false;
        pc.createAnswer().then(function(answer) {
            answerReceived = true;
            answerType = answer.type;
            hasSdp = typeof answer.sdp === "string" && answer.sdp.length > 0;
        });
        answerReceived && answerType === "answer" && hasSdp;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}
