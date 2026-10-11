//! Workers run on the shared event loop (ThaloraJobExecutor): promises,
//! async functions, timers and fetch behave as they do in pages.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

use crate::misc::structured_clone::StructuredCloneValue;
use crate::worker::worker_thread::{WorkerConfig, WorkerEvent, WorkerThread, WorkerType};

fn spawn(script: &str) -> WorkerThread {
    WorkerThread::spawn(WorkerConfig {
        name: Some("event-loop-test".to_string()),
        script_url: script.to_string(),
        worker_type: WorkerType::Classic,
        stack_size: Some(4 * 1024 * 1024),
        origin: "thalora://test".to_string(),
    })
    .expect("worker spawns")
}

/// String messages posted by the worker until `count` arrive or `timeout`.
fn messages(worker: &WorkerThread, count: usize, timeout: Duration) -> Vec<String> {
    let deadline = Instant::now() + timeout;
    let mut out = Vec::new();
    while out.len() < count && Instant::now() < deadline {
        match worker.recv_event_timeout(Duration::from_millis(50)) {
            Some(WorkerEvent::Message {
                data: StructuredCloneValue::String(s),
            }) => out.push(s),
            Some(WorkerEvent::Error { message, .. }) => out.push(format!("error: {message}")),
            _ => {}
        }
    }
    out
}

#[test]
fn microtasks_async_functions_and_timers_run_in_order() {
    let mut worker = spawn(
        r#"
        setTimeout(() => postMessage('timer'), 20);
        Promise.resolve().then(() => postMessage('microtask'));
        (async () => { await null; await null; postMessage('async'); })();
        postMessage('sync');
        "#,
    );
    let got = messages(&worker, 4, Duration::from_secs(5));
    assert_eq!(got, ["sync", "microtask", "async", "timer"]);
    worker.terminate();
}

#[test]
fn intervals_repeat_and_clear() {
    let mut worker = spawn(
        r#"
        let n = 0;
        const id = setInterval(() => {
            n += 1;
            if (n === 3) { clearInterval(id); postMessage('ticks:' + n); }
        }, 5);
        "#,
    );
    assert_eq!(messages(&worker, 1, Duration::from_secs(5)), ["ticks:3"]);
    worker.terminate();
}

#[test]
fn fetch_settles_inside_a_worker() {
    // SAFETY: tests that touch THALORA_ALLOW_LOOPBACK only ever set it to "1"
    unsafe { std::env::set_var("THALORA_ALLOW_LOOPBACK", "1") };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let body = r#"{"message":"from server"}"#;
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
        }
    });

    let mut worker = spawn(&format!(
        r#"
        fetch('http://127.0.0.1:{port}/data')
            .then(r => r.json())
            .then(d => postMessage('fetched:' + d.message))
            .catch(e => postMessage('failed:' + e));
        "#
    ));
    assert_eq!(
        messages(&worker, 1, Duration::from_secs(10)),
        ["fetched:from server"]
    );
    worker.terminate();
}
