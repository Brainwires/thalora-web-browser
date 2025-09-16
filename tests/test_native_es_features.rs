use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_es6_array_methods_native() {
    println!("🧪 Testing ES6 Array methods are natively available...");

    let browser = HeadlessWebBrowser::new();

    let array_methods_test = browser.lock().unwrap().execute_javascript(
        "typeof Array.prototype.find === 'function'"
    ).await;

    match array_methods_test {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("ES6 Array methods test: {}", value_str);
            assert!(value_str.contains("true"), "ES6 Array methods should work natively, got: {}", value_str);
            println!("✅ ES6 Array methods working natively");
        },
        Err(e) => panic!("Failed to test ES6 Array methods: {:?}", e),
    }
}

#[tokio::test]
async fn test_es6_object_methods_native() {
    println!("🧪 Testing ES6 Object methods are natively available...");

    let browser = HeadlessWebBrowser::new();

    let object_methods_test = browser.lock().unwrap().execute_javascript(
        "typeof Object.keys === 'function'"
    ).await;

    match object_methods_test {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("ES6 Object methods test: {}", value_str);
            assert!(value_str.contains("true"), "ES6 Object methods should work natively, got: {}", value_str);
            println!("✅ ES6 Object methods working natively");
        },
        Err(e) => panic!("Failed to test ES6 Object methods: {:?}", e),
    }
}

#[tokio::test]
async fn test_es6_string_methods_native() {
    println!("🧪 Testing ES6 String methods are natively available...");

    let browser = HeadlessWebBrowser::new();

    let string_methods_test = browser.lock().unwrap().execute_javascript(
        "typeof String.prototype.includes === 'function'"
    ).await;

    match string_methods_test {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("ES6 String methods test: {}", value_str);
            assert!(value_str.contains("true"), "ES6 String methods should work natively, got: {}", value_str);
            println!("✅ ES6 String methods working natively");
        },
        Err(e) => panic!("Failed to test ES6 String methods: {:?}", e),
    }
}

#[tokio::test]
async fn test_es2019_array_flat_native() {
    println!("🧪 Testing ES2019 Array.flat is natively available...");

    let browser = HeadlessWebBrowser::new();

    let flat_test = browser.lock().unwrap().execute_javascript(
        "typeof Array.prototype.flat === 'function'"
    ).await;

    match flat_test {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("ES2019 Array.flat test: {}", value_str);
            assert!(value_str.contains("true"), "ES2019 Array.flat should work natively, got: {}", value_str);
            println!("✅ ES2019 Array.flat working natively");
        },
        Err(e) => panic!("Failed to test ES2019 Array.flat: {:?}", e),
    }
}

#[tokio::test]
async fn test_promise_features_native() {
    println!("🧪 Testing Promise features are natively available...");

    let browser = HeadlessWebBrowser::new();

    let promise_test = browser.lock().unwrap().execute_javascript(
        "typeof Promise.allSettled === 'function'"
    ).await;

    match promise_test {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Promise features test: {}", value_str);
            assert!(value_str.contains("true"), "Promise features should be available natively, got: {}", value_str);
            println!("✅ Promise features working natively");
        },
        Err(e) => panic!("Failed to test Promise features: {:?}", e),
    }
}

#[tokio::test]
async fn test_number_methods_native() {
    println!("🧪 Testing Number methods are natively available...");

    let browser = HeadlessWebBrowser::new();

    let number_test = browser.lock().unwrap().execute_javascript(
        "typeof Number.isNaN === 'function'"
    ).await;

    match number_test {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Number methods test: {}", value_str);
            assert!(value_str.contains("true"), "Number methods should be available natively, got: {}", value_str);
            println!("✅ Number methods working natively");
        },
        Err(e) => panic!("Failed to test Number methods: {:?}", e),
    }
}

#[tokio::test]
async fn test_string_padding_methods_native() {
    println!("🧪 Testing String padding methods are natively available...");

    let browser = HeadlessWebBrowser::new();

    let padding_test = browser.lock().unwrap().execute_javascript(
        "typeof String.prototype.padStart === 'function'"
    ).await;

    match padding_test {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("String padding methods test: {}", value_str);
            assert!(value_str.contains("true"), "String padding methods should be available natively, got: {}", value_str);
            println!("✅ String padding methods working natively");
        },
        Err(e) => panic!("Failed to test String padding methods: {:?}", e),
    }
}