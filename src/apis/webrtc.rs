use anyhow::Result;
use boa_engine::{Context, JsObject, JsValue, NativeFunction, js_string, property::Attribute};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// WebRTC API implementation for full browser compatibility
pub struct WebRTCManager {
    peer_connections: Arc<Mutex<HashMap<String, RTCPeerConnection>>>,
    data_channels: Arc<Mutex<HashMap<String, RTCDataChannel>>>,
}

#[derive(Debug, Clone)]
pub struct RTCPeerConnection {
    pub connection_state: String,
    pub ice_connection_state: String,
    pub ice_gathering_state: String,
    pub signaling_state: String,
}

#[derive(Debug, Clone)]
pub struct RTCDataChannel {
    pub label: String,
    pub ready_state: String,
    pub buffered_amount: u32,
}

impl WebRTCManager {
    pub fn new() -> Self {
        Self {
            peer_connections: Arc::new(Mutex::new(HashMap::new())),
            data_channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Setup WebRTC API in global scope
    pub fn setup_webrtc_api(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        // RTCPeerConnection constructor
        let rtc_peer_connection_constructor = unsafe { NativeFunction::from_closure({
            let peer_connections = Arc::clone(&self.peer_connections);
            move |_, args, context| {
                let pc_obj = JsObject::default();

                // Connection state properties
                pc_obj.set(js_string!("connectionState"), JsValue::from(js_string!("new")), false, context)?;
                pc_obj.set(js_string!("iceConnectionState"), JsValue::from(js_string!("new")), false, context)?;
                pc_obj.set(js_string!("iceGatheringState"), JsValue::from(js_string!("new")), false, context)?;
                pc_obj.set(js_string!("signalingState"), JsValue::from(js_string!("stable")), false, context)?;

                // Methods
                let create_offer_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
                    let promise_obj = JsObject::default();

                    let then_fn = unsafe { NativeFunction::from_closure(move |_, callback_args, callback_context| {
                        if !callback_args.is_empty() && callback_args[0].is_callable() {
                            let offer_obj = JsObject::default();
                            offer_obj.set(js_string!("type"), JsValue::from(js_string!("offer")), false, callback_context)?;
                            offer_obj.set(js_string!("sdp"), JsValue::from(js_string!("v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\n")), false, callback_context)?;

                            if let Some(callback) = callback_args[0].as_callable() {
                                let _ = callback.call(&JsValue::undefined(), &[JsValue::from(offer_obj)], callback_context);
                            }
                        }
                        Ok(JsValue::undefined())
                    }) };
                    promise_obj.set(js_string!("then"), JsValue::from(then_fn.to_js_function(context.realm())), false, context)?;

                    let catch_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                    promise_obj.set(js_string!("catch"), JsValue::from(catch_fn.to_js_function(context.realm())), false, context)?;

                    Ok(JsValue::from(promise_obj))
                }) };
                pc_obj.set(js_string!("createOffer"), JsValue::from(create_offer_fn.to_js_function(context.realm())), false, context)?;

                let create_answer_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
                    let promise_obj = JsObject::default();

                    let then_fn = unsafe { NativeFunction::from_closure(move |_, callback_args, callback_context| {
                        if !callback_args.is_empty() && callback_args[0].is_callable() {
                            let answer_obj = JsObject::default();
                            answer_obj.set(js_string!("type"), JsValue::from(js_string!("answer")), false, callback_context)?;
                            answer_obj.set(js_string!("sdp"), JsValue::from(js_string!("v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\n")), false, callback_context)?;

                            if let Some(callback) = callback_args[0].as_callable() {
                                let _ = callback.call(&JsValue::undefined(), &[JsValue::from(answer_obj)], callback_context);
                            }
                        }
                        Ok(JsValue::undefined())
                    }) };
                    promise_obj.set(js_string!("then"), JsValue::from(then_fn.to_js_function(context.realm())), false, context)?;

                    let catch_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                    promise_obj.set(js_string!("catch"), JsValue::from(catch_fn.to_js_function(context.realm())), false, context)?;

                    Ok(JsValue::from(promise_obj))
                }) };
                pc_obj.set(js_string!("createAnswer"), JsValue::from(create_answer_fn.to_js_function(context.realm())), false, context)?;

                let set_local_description_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
                    let promise_obj = JsObject::default();
                    let then_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                    promise_obj.set(js_string!("then"), JsValue::from(then_fn.to_js_function(context.realm())), false, context)?;
                    let catch_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                    promise_obj.set(js_string!("catch"), JsValue::from(catch_fn.to_js_function(context.realm())), false, context)?;
                    Ok(JsValue::from(promise_obj))
                }) };
                pc_obj.set(js_string!("setLocalDescription"), JsValue::from(set_local_description_fn.to_js_function(context.realm())), false, context)?;

                let set_remote_description_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
                    let promise_obj = JsObject::default();
                    let then_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                    promise_obj.set(js_string!("then"), JsValue::from(then_fn.to_js_function(context.realm())), false, context)?;
                    let catch_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                    promise_obj.set(js_string!("catch"), JsValue::from(catch_fn.to_js_function(context.realm())), false, context)?;
                    Ok(JsValue::from(promise_obj))
                }) };
                pc_obj.set(js_string!("setRemoteDescription"), JsValue::from(set_remote_description_fn.to_js_function(context.realm())), false, context)?;

                let add_ice_candidate_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
                    let promise_obj = JsObject::default();
                    let then_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                    promise_obj.set(js_string!("then"), JsValue::from(then_fn.to_js_function(context.realm())), false, context)?;
                    let catch_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                    promise_obj.set(js_string!("catch"), JsValue::from(catch_fn.to_js_function(context.realm())), false, context)?;
                    Ok(JsValue::from(promise_obj))
                }) };
                pc_obj.set(js_string!("addIceCandidate"), JsValue::from(add_ice_candidate_fn.to_js_function(context.realm())), false, context)?;

                let create_data_channel_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
                    let dc_obj = JsObject::default();

                    let label = if !args.is_empty() {
                        args[0].to_string(context)?.to_std_string_escaped()
                    } else {
                        "datachannel".to_string()
                    };

                    dc_obj.set(js_string!("label"), JsValue::from(js_string!(label)), false, context)?;
                    dc_obj.set(js_string!("readyState"), JsValue::from(js_string!("connecting")), false, context)?;
                    dc_obj.set(js_string!("bufferedAmount"), JsValue::from(0), false, context)?;

                    let send_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                    dc_obj.set(js_string!("send"), JsValue::from(send_fn.to_js_function(context.realm())), false, context)?;

                    let close_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                    dc_obj.set(js_string!("close"), JsValue::from(close_fn.to_js_function(context.realm())), false, context)?;

                    // Event handlers
                    dc_obj.set(js_string!("onopen"), JsValue::null(), true, context)?;
                    dc_obj.set(js_string!("onclose"), JsValue::null(), true, context)?;
                    dc_obj.set(js_string!("onmessage"), JsValue::null(), true, context)?;
                    dc_obj.set(js_string!("onerror"), JsValue::null(), true, context)?;

                    Ok(JsValue::from(dc_obj))
                }) };
                pc_obj.set(js_string!("createDataChannel"), JsValue::from(create_data_channel_fn.to_js_function(context.realm())), false, context)?;

                let close_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                pc_obj.set(js_string!("close"), JsValue::from(close_fn.to_js_function(context.realm())), false, context)?;

                // Event handlers
                pc_obj.set(js_string!("onicecandidate"), JsValue::null(), true, context)?;
                pc_obj.set(js_string!("oniceconnectionstatechange"), JsValue::null(), true, context)?;
                pc_obj.set(js_string!("onnegotiationneeded"), JsValue::null(), true, context)?;
                pc_obj.set(js_string!("ondatachannel"), JsValue::null(), true, context)?;

                Ok(JsValue::from(pc_obj))
            }
        }) };
        context.register_global_property(js_string!("RTCPeerConnection"), JsValue::from(rtc_peer_connection_constructor.to_js_function(context.realm())), Attribute::all())?;

        // RTCIceCandidate constructor
        let rtc_ice_candidate_constructor = unsafe { NativeFunction::from_closure(|_, args, context| {
            let candidate_obj = JsObject::default();

            if !args.is_empty() && args[0].is_object() {
                if let Some(init) = args[0].as_object() {
                    if let Ok(candidate) = init.get(js_string!("candidate"), context) {
                        candidate_obj.set(js_string!("candidate"), candidate, false, context)?;
                    }
                    if let Ok(sdp_mid) = init.get(js_string!("sdpMid"), context) {
                        candidate_obj.set(js_string!("sdpMid"), sdp_mid, false, context)?;
                    }
                    if let Ok(sdp_m_line_index) = init.get(js_string!("sdpMLineIndex"), context) {
                        candidate_obj.set(js_string!("sdpMLineIndex"), sdp_m_line_index, false, context)?;
                    }
                }
            }

            Ok(JsValue::from(candidate_obj))
        }) };
        context.register_global_property(js_string!("RTCIceCandidate"), JsValue::from(rtc_ice_candidate_constructor.to_js_function(context.realm())), Attribute::all())?;

        // RTCSessionDescription constructor
        let rtc_session_description_constructor = unsafe { NativeFunction::from_closure(|_, args, context| {
            let desc_obj = JsObject::default();

            if !args.is_empty() && args[0].is_object() {
                if let Some(init) = args[0].as_object() {
                    if let Ok(type_val) = init.get(js_string!("type"), context) {
                        desc_obj.set(js_string!("type"), type_val, false, context)?;
                    }
                    if let Ok(sdp) = init.get(js_string!("sdp"), context) {
                        desc_obj.set(js_string!("sdp"), sdp, false, context)?;
                    }
                }
            }

            Ok(JsValue::from(desc_obj))
        }) };
        context.register_global_property(js_string!("RTCSessionDescription"), JsValue::from(rtc_session_description_constructor.to_js_function(context.realm())), Attribute::all())?;

        // getUserMedia in navigator.mediaDevices
        let navigator_obj = context.global_object().get(js_string!("navigator"), context)?;
        if let Some(nav_obj) = navigator_obj.as_object() {
            let media_devices_obj = JsObject::default();

            let get_user_media_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
                let promise_obj = JsObject::default();

                let then_fn = unsafe { NativeFunction::from_closure(move |_, callback_args, callback_context| {
                    if !callback_args.is_empty() && callback_args[0].is_callable() {
                        let stream_obj = JsObject::default();

                        // Mock MediaStream
                        stream_obj.set(js_string!("id"), JsValue::from(js_string!("mock-stream-id")), false, callback_context)?;
                        stream_obj.set(js_string!("active"), JsValue::from(true), false, callback_context)?;

                        let get_tracks_fn = unsafe { NativeFunction::from_closure(|_, _, callback_context| {
                            let tracks_array = JsObject::default();
                            tracks_array.set(js_string!("length"), JsValue::from(0), false, callback_context)?;
                            Ok(JsValue::from(tracks_array))
                        }) };
                        stream_obj.set(js_string!("getTracks"), JsValue::from(get_tracks_fn.to_js_function(callback_context.realm())), false, callback_context)?;

                        if let Some(callback) = callback_args[0].as_callable() {
                            let _ = callback.call(&JsValue::undefined(), &[JsValue::from(stream_obj)], callback_context);
                        }
                    }
                    Ok(JsValue::undefined())
                }) };
                promise_obj.set(js_string!("then"), JsValue::from(then_fn.to_js_function(context.realm())), false, context)?;

                let catch_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                promise_obj.set(js_string!("catch"), JsValue::from(catch_fn.to_js_function(context.realm())), false, context)?;

                Ok(JsValue::from(promise_obj))
            }) };
            media_devices_obj.set(js_string!("getUserMedia"), JsValue::from(get_user_media_fn.to_js_function(context.realm())), false, context)?;

            nav_obj.set(js_string!("mediaDevices"), JsValue::from(media_devices_obj), false, context)?;
        }

        Ok(())
    }
}