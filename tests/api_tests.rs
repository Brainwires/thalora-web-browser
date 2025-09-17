// API Integration Tests
// This file runs all web API tests

use thalora::{WebSocketManager, WebSocketJsApi};
use thalora::apis::WebApis;
use thalora::apis::websocket::{ConnectionState, MessageType, WebSocketMessage};
use boa_engine::{Context, Source};

mod utils;
use utils::{create_isolated_test_server, get_server_url};

mod websocket_connection {
    use super::*;
    include!("apis/websocket/connection.rs");
}

mod websocket_manager_creation {
    use super::*;
    include!("apis/websocket/manager_creation.rs");
}

mod websocket_messaging {
    use super::*;
    include!("apis/websocket/messaging.rs");
}

mod websocket_ping_pong {
    use super::*;
    include!("apis/websocket/ping_pong.rs");
}

mod websocket_multiple_connections {
    use super::*;
    include!("apis/websocket/multiple_connections.rs");
}

mod websocket_realtime_events_simulation {
    use super::*;
    include!("apis/websocket/realtime_events_simulation.rs");
}

mod websocket_message_handlers {
    use super::*;
    include!("apis/websocket/message_handlers.rs");
}

mod websocket_js_api {
    use super::*;
    include!("apis/websocket/js_api.rs");
}

mod websocket_connection_error_handling {
    use super::*;
    include!("apis/websocket/connection_error_handling.rs");
}

mod geolocation_navigator_exists {
    use super::*;
    include!("apis/geolocation/navigator_exists.rs");
}

mod geolocation_get_current_position {
    use super::*;
    include!("apis/geolocation/get_current_position.rs");
}

mod geolocation_position_object {
    use super::*;
    include!("apis/geolocation/position_object.rs");
}

mod geolocation_watch_position {
    use super::*;
    include!("apis/geolocation/watch_position.rs");
}

mod geolocation_clear_watch {
    use super::*;
    include!("apis/geolocation/clear_watch.rs");
}

mod geolocation_error_handling {
    use super::*;
    include!("apis/geolocation/error_handling.rs");
}

mod geolocation_coordinates_accuracy {
    use super::*;
    include!("apis/geolocation/coordinates_accuracy.rs");
}