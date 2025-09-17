use std::process::Command;

fn main() {
    println!("🧪 Testing native WebSocket implementation in Boa through Thalora browser...");

    // Test 1: WebSocket constructor exists
    let output = Command::new("./target/release/thalora")
        .arg("--eval")
        .arg("typeof WebSocket")
        .output();

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            println!("WebSocket type check:");
            println!("stdout: {}", stdout);
            println!("stderr: {}", stderr);

            if stdout.contains("function") {
                println!("✅ WebSocket constructor exists");
            } else {
                println!("❌ CRITICAL: WebSocket constructor not found!");
                return;
            }
        },
        Err(e) => {
            println!("❌ CRITICAL: Failed to run thalora: {:?}", e);
            return;
        }
    }

    // Test 2: WebSocket constants
    let output = Command::new("./target/release/thalora")
        .arg("--eval")
        .arg("WebSocket.CONNECTING")
        .output();

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            println!("WebSocket.CONNECTING: {}", stdout);

            if stdout.trim() == "0" {
                println!("✅ WebSocket.CONNECTING = 0");
            } else {
                println!("❌ CRITICAL: WebSocket.CONNECTING not 0!");
                return;
            }
        },
        Err(e) => {
            println!("❌ CRITICAL: Failed to check WebSocket.CONNECTING: {:?}", e);
            return;
        }
    }

    // Test 3: Create WebSocket instance
    let js_code = r#"
        try {
            var ws = new WebSocket("wss://echo.websocket.org");
            typeof ws;
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let output = Command::new("./target/release/thalora")
        .arg("--eval")
        .arg(js_code)
        .output();

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            println!("WebSocket instance creation: {}", stdout);

            if stdout.contains("error") {
                println!("❌ CRITICAL: WebSocket instance creation failed: {}", stdout);
                return;
            } else if stdout.contains("object") {
                println!("✅ WebSocket instance created successfully");
            } else {
                println!("❌ CRITICAL: WebSocket instance not an object: {}", stdout);
                return;
            }
        },
        Err(e) => {
            println!("❌ CRITICAL: Failed to create WebSocket instance: {:?}", e);
            return;
        }
    }

    println!("🎉 All WebSocket tests PASSED - Native implementation working!");
}