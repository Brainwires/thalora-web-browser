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
