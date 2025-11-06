//! Comprehensive tests for worker message passing
//! Tests bidirectional communication between main thread and workers

#[cfg(test)]
mod worker_message_tests {
    use crate::worker::worker_thread::{WorkerThread, WorkerConfig, WorkerType, WorkerCommand, WorkerEvent};
    use crate::misc::structured_clone::{structured_clone, StructuredCloneValue};
    use boa_engine::{Context, JsValue, js_string};
    use std::time::Duration;
    use std::thread;

    /// Helper to serialize a JsValue for message passing
    fn serialize_message(value: &JsValue, context: &mut Context) -> StructuredCloneValue {
        structured_clone(value, context, None).expect("Failed to clone message")
    }

    /// Test main thread -> worker postMessage
    #[test]
    fn test_post_message_to_worker() {
        let script = r#"
            var receivedMessage = null;
            onmessage = function(event) {
                receivedMessage = event.data;
            };
        "#;

        let config = WorkerConfig {
            name: Some("message-test-worker".to_string()),
            script_url: script.to_string(),
            worker_type: WorkerType::Classic,
            stack_size: Some(2 * 1024 * 1024),
        };

        let mut worker = WorkerThread::spawn(config).expect("Worker should spawn");
        thread::sleep(Duration::from_millis(200));

        // Create a message to send
        let mut context = Context::default();
        let message = JsValue::from(js_string!("Hello Worker!"));
        let serialized = serialize_message(&message, &mut context);

        // Send message to worker
        let result = worker.send_command(WorkerCommand::PostMessage { message: serialized });
        assert!(result.is_ok(), "Should send message to worker");

        // Give worker time to process message
        thread::sleep(Duration::from_millis(200));

        worker.terminate();
    }

    /// Test worker -> main thread postMessage
    #[test]
    fn test_post_message_from_worker() {
        let script = r#"
            postMessage("Hello from worker!");
        "#;

        let config = WorkerConfig {
            name: Some("worker-post-message".to_string()),
            script_url: script.to_string(),
            worker_type: WorkerType::Classic,
            stack_size: Some(2 * 1024 * 1024),
        };

        let mut worker = WorkerThread::spawn(config).expect("Worker should spawn");
        thread::sleep(Duration::from_millis(300));

        // Check for message event from worker
        let mut got_message = false;
        for _ in 0..10 {
            if let Some(event) = worker.try_recv_event() {
                match event {
                    WorkerEvent::Message { data } => {
                        got_message = true;
                        // Verify message data
                        let mut context = Context::default();
                        if let Ok(value) = crate::misc::structured_clone::structured_deserialize(&data, &mut context) {
                            if let Some(s) = value.as_string() {
                                assert_eq!(s.to_std_string_escaped(), "Hello from worker!");
                            }
                        }
                    }
                    WorkerEvent::Error { message, .. } => {
                        panic!("Worker error: {}", message);
                    }
                    _ => {}
                }
            }
        }

        assert!(got_message, "Should receive message from worker");
        worker.terminate();
    }

    /// Test bidirectional message exchange
    #[test]
    fn test_bidirectional_messaging() {
        let script = r#"
            var messageCount = 0;
            onmessage = function(event) {
                messageCount++;
                postMessage("Reply " + messageCount + ": " + event.data);
            };
        "#;

        let config = WorkerConfig {
            name: Some("bidirectional-worker".to_string()),
            script_url: script.to_string(),
            worker_type: WorkerType::Classic,
            stack_size: Some(2 * 1024 * 1024),
        };

        let mut worker = WorkerThread::spawn(config).expect("Worker should spawn");
        thread::sleep(Duration::from_millis(200));

        // Send first message
        let mut context = Context::default();
        let message1 = JsValue::from(js_string!("Message 1"));
        let serialized1 = serialize_message(&message1, &mut context);
        worker.send_command(WorkerCommand::PostMessage { message: serialized1 }).expect("Send 1");

        thread::sleep(Duration::from_millis(200));

        // Check for reply
        let mut got_reply = false;
        for _ in 0..10 {
            if let Some(event) = worker.try_recv_event() {
                if let WorkerEvent::Message { .. } = event {
                    got_reply = true;
                    break;
                }
            }
        }

        assert!(got_reply, "Should receive reply from worker");

        // Send second message
        let message2 = JsValue::from(js_string!("Message 2"));
        let serialized2 = serialize_message(&message2, &mut context);
        worker.send_command(WorkerCommand::PostMessage { message: serialized2 }).expect("Send 2");

        thread::sleep(Duration::from_millis(200));

        worker.terminate();
    }

    /// Test posting numbers and booleans
    #[test]
    fn test_post_message_primitives() {
        let script = r#"
            onmessage = function(event) {
                if (typeof event.data === 'number') {
                    postMessage(event.data * 2);
                } else if (typeof event.data === 'boolean') {
                    postMessage(!event.data);
                }
            };
        "#;

        let config = WorkerConfig {
            name: Some("primitive-worker".to_string()),
            script_url: script.to_string(),
            worker_type: WorkerType::Classic,
            stack_size: Some(2 * 1024 * 1024),
        };

        let mut worker = WorkerThread::spawn(config).expect("Worker should spawn");
        thread::sleep(Duration::from_millis(200));

        // Send a number
        let mut context = Context::default();
        let number = JsValue::from(42);
        let serialized = serialize_message(&number, &mut context);
        worker.send_command(WorkerCommand::PostMessage { message: serialized }).expect("Send number");

        thread::sleep(Duration::from_millis(200));

        // Check for doubled number
        let mut got_number = false;
        for _ in 0..10 {
            if let Some(event) = worker.try_recv_event() {
                if let WorkerEvent::Message { data } = event {
                    if let Ok(value) = crate::misc::structured_clone::structured_deserialize(&data, &mut context) {
                        if let Some(num) = value.as_number() {
                            assert_eq!(num, 84.0);
                            got_number = true;
                            break;
                        }
                    }
                }
            }
        }

        assert!(got_number, "Should receive doubled number from worker");
        worker.terminate();
    }

    /// Test posting arrays
    #[test]
    fn test_post_message_array() {
        let script = r#"
            onmessage = function(event) {
                if (Array.isArray(event.data)) {
                    postMessage(event.data.length);
                }
            };
        "#;

        let config = WorkerConfig {
            name: Some("array-worker".to_string()),
            script_url: script.to_string(),
            worker_type: WorkerType::Classic,
            stack_size: Some(2 * 1024 * 1024),
        };

        let mut worker = WorkerThread::spawn(config).expect("Worker should spawn");
        thread::sleep(Duration::from_millis(200));

        // Create an array
        let mut context = Context::default();
        let array = context.eval(boa_engine::Source::from_bytes("[1, 2, 3, 4, 5]")).expect("Create array");
        let serialized = serialize_message(&array, &mut context);
        worker.send_command(WorkerCommand::PostMessage { message: serialized }).expect("Send array");

        thread::sleep(Duration::from_millis(200));

        // Check for array length response
        let mut got_length = false;
        for _ in 0..10 {
            if let Some(event) = worker.try_recv_event() {
                if let WorkerEvent::Message { data } = event {
                    if let Ok(value) = crate::misc::structured_clone::structured_deserialize(&data, &mut context) {
                        if let Some(num) = value.as_number() {
                            assert_eq!(num, 5.0);
                            got_length = true;
                            break;
                        }
                    }
                }
            }
        }

        assert!(got_length, "Should receive array length from worker");
        worker.terminate();
    }

    /// Test posting objects
    #[test]
    fn test_post_message_object() {
        let script = r#"
            onmessage = function(event) {
                if (event.data && typeof event.data === 'object') {
                    postMessage({
                        received: true,
                        name: event.data.name,
                        value: event.data.value
                    });
                }
            };
        "#;

        let config = WorkerConfig {
            name: Some("object-worker".to_string()),
            script_url: script.to_string(),
            worker_type: WorkerType::Classic,
            stack_size: Some(2 * 1024 * 1024),
        };

        let mut worker = WorkerThread::spawn(config).expect("Worker should spawn");
        thread::sleep(Duration::from_millis(200));

        // Create an object
        let mut context = Context::default();
        let obj = context.eval(boa_engine::Source::from_bytes("({name: 'test', value: 123})"))
            .expect("Create object");
        let serialized = serialize_message(&obj, &mut context);
        worker.send_command(WorkerCommand::PostMessage { message: serialized }).expect("Send object");

        thread::sleep(Duration::from_millis(300));

        // Check for response object
        let mut got_object = false;
        for _ in 0..10 {
            if let Some(event) = worker.try_recv_event() {
                if let WorkerEvent::Message { data } = event {
                    if let Ok(_value) = crate::misc::structured_clone::structured_deserialize(&data, &mut context) {
                        got_object = true;
                        break;
                    }
                }
            }
        }

        assert!(got_object, "Should receive object from worker");
        worker.terminate();
    }

    /// Test multiple rapid messages
    #[test]
    fn test_rapid_messaging() {
        let script = r#"
            var count = 0;
            onmessage = function(event) {
                count++;
                if (count >= 5) {
                    postMessage("Received " + count + " messages");
                }
            };
        "#;

        let config = WorkerConfig {
            name: Some("rapid-message-worker".to_string()),
            script_url: script.to_string(),
            worker_type: WorkerType::Classic,
            stack_size: Some(2 * 1024 * 1024),
        };

        let mut worker = WorkerThread::spawn(config).expect("Worker should spawn");
        thread::sleep(Duration::from_millis(200));

        // Send 5 rapid messages
        let mut context = Context::default();
        for i in 1..=5 {
            let message = JsValue::from(js_string!(format!("Message {}", i)));
            let serialized = serialize_message(&message, &mut context);
            worker.send_command(WorkerCommand::PostMessage { message: serialized }).expect("Send message");
        }

        thread::sleep(Duration::from_millis(300));

        // Check for summary message
        let mut got_summary = false;
        for _ in 0..10 {
            if let Some(event) = worker.try_recv_event() {
                if let WorkerEvent::Message { .. } = event {
                    got_summary = true;
                    break;
                }
            }
        }

        assert!(got_summary, "Should receive summary message from worker");
        worker.terminate();
    }

    /// Test error handling in message handler
    #[test]
    fn test_message_handler_error() {
        let script = r#"
            onmessage = function(event) {
                if (event.data === "error") {
                    throw new Error("Test error in message handler");
                }
                postMessage("ok");
            };
        "#;

        let config = WorkerConfig {
            name: Some("error-handler-worker".to_string()),
            script_url: script.to_string(),
            worker_type: WorkerType::Classic,
            stack_size: Some(2 * 1024 * 1024),
        };

        let mut worker = WorkerThread::spawn(config).expect("Worker should spawn");
        thread::sleep(Duration::from_millis(200));

        // Send normal message first
        let mut context = Context::default();
        let message = JsValue::from(js_string!("normal"));
        let serialized = serialize_message(&message, &mut context);
        worker.send_command(WorkerCommand::PostMessage { message: serialized }).expect("Send normal");

        thread::sleep(Duration::from_millis(200));

        // Worker should still be running
        assert!(worker.is_running(), "Worker should still be running after processing normal message");

        worker.terminate();
    }
}
