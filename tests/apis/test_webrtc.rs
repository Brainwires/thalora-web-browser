use synaptic::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_rtc_peer_connection_constructor() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test RTCPeerConnection exists
    let result = context.eval(Source::from_bytes("typeof RTCPeerConnection")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");

    // Test RTCPeerConnection can be instantiated
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        typeof pc === "object";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_rtc_peer_connection_states() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test initial connection states
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        pc.connectionState === "new" &&
        pc.iceConnectionState === "new" &&
        pc.iceGatheringState === "new" &&
        pc.signalingState === "stable";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_rtc_peer_connection_create_offer() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test createOffer method exists and returns Promise-like object
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        const offer = pc.createOffer();
        typeof offer.then === "function" && typeof offer.catch === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test createOffer callback receives offer object
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        let offerReceived = false;
        let offerType = null;
        let hasSdp = false;

        pc.createOffer().then(function(offer) {
            offerReceived = true;
            offerType = offer.type;
            hasSdp = typeof offer.sdp === "string" && offer.sdp.length > 0;
        });

        offerReceived && offerType === "offer" && hasSdp;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

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

#[tokio::test]
async fn test_rtc_peer_connection_description_methods() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test setLocalDescription and setRemoteDescription exist
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        typeof pc.setLocalDescription === "function" &&
        typeof pc.setRemoteDescription === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test they return Promise-like objects
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        const local = pc.setLocalDescription();
        const remote = pc.setRemoteDescription();
        typeof local.then === "function" && typeof remote.then === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_rtc_peer_connection_ice_candidates() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test addIceCandidate method exists
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        typeof pc.addIceCandidate === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test addIceCandidate returns Promise-like object
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        const result = pc.addIceCandidate();
        typeof result.then === "function" && typeof result.catch === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_rtc_peer_connection_data_channel() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test createDataChannel method exists
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        typeof pc.createDataChannel === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test createDataChannel returns data channel object
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        const dc = pc.createDataChannel("test");
        dc.label === "test" &&
        dc.readyState === "connecting" &&
        dc.bufferedAmount === 0 &&
        typeof dc.send === "function" &&
        typeof dc.close === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test data channel event handlers
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        const dc = pc.createDataChannel("test");
        dc.onopen === null &&
        dc.onclose === null &&
        dc.onmessage === null &&
        dc.onerror === null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_rtc_peer_connection_event_handlers() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test event handler properties exist and are initially null
    let result = context.eval(Source::from_bytes(r#"
        const pc = new RTCPeerConnection();
        pc.onicecandidate === null &&
        pc.oniceconnectionstatechange === null &&
        pc.onnegotiationneeded === null &&
        pc.ondatachannel === null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_rtc_ice_candidate_constructor() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test RTCIceCandidate exists
    let result = context.eval(Source::from_bytes("typeof RTCIceCandidate")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");

    // Test RTCIceCandidate can be instantiated with init object
    let result = context.eval(Source::from_bytes(r#"
        const candidate = new RTCIceCandidate({
            candidate: "candidate:1 1 UDP 2130706431 192.168.1.1 54400 typ host",
            sdpMid: "data",
            sdpMLineIndex: 0
        });
        candidate.candidate.includes("192.168.1.1") &&
        candidate.sdpMid === "data" &&
        candidate.sdpMLineIndex === 0;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_rtc_session_description_constructor() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test RTCSessionDescription exists
    let result = context.eval(Source::from_bytes("typeof RTCSessionDescription")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");

    // Test RTCSessionDescription can be instantiated with init object
    let result = context.eval(Source::from_bytes(r#"
        const desc = new RTCSessionDescription({
            type: "offer",
            sdp: "v=0\r\no=- 123 456 IN IP4 127.0.0.1\r\n"
        });
        desc.type === "offer" && desc.sdp.includes("v=0");
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_navigator_media_devices_get_user_media() {
    let mut context = Context::default();
    let web_apis = WebApis::new();
    web_apis.setup_all_apis(&mut context).expect("Failed to setup WebAPIs");

    // Test navigator.mediaDevices.getUserMedia exists
    let result = context.eval(Source::from_bytes("typeof navigator.mediaDevices.getUserMedia")).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "function");

    // Test getUserMedia returns Promise-like object
    let result = context.eval(Source::from_bytes(r#"
        const result = navigator.mediaDevices.getUserMedia();
        typeof result.then === "function" && typeof result.catch === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test getUserMedia callback receives MediaStream
    let result = context.eval(Source::from_bytes(r#"
        let streamReceived = false;
        let streamId = null;
        let isActive = false;

        navigator.mediaDevices.getUserMedia().then(function(stream) {
            streamReceived = true;
            streamId = stream.id;
            isActive = stream.active;
        });

        streamReceived && streamId === "mock-stream-id" && isActive === true;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}