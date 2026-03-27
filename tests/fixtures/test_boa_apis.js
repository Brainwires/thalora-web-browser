// Simple test script to verify Boa APIs work
console.log('Testing Boa APIs...');

// Test fetch API
try {
    console.log('fetch exists:', typeof fetch);
    console.log('Request exists:', typeof Request);
    console.log('Response exists:', typeof Response);
    console.log('Headers exists:', typeof Headers);
} catch (e) {
    console.log('Fetch API error:', e.message);
}

// Test WebSocket API
try {
    console.log('WebSocket exists:', typeof WebSocket);
    console.log('WebSocket.CONNECTING:', WebSocket.CONNECTING);
    console.log('WebSocket.OPEN:', WebSocket.OPEN);
} catch (e) {
    console.log('WebSocket API error:', e.message);
}

// Test Events API
try {
    console.log('Event exists:', typeof Event);
    console.log('EventTarget exists:', typeof EventTarget);
    console.log('CustomEvent exists:', typeof CustomEvent);
} catch (e) {
    console.log('Events API error:', e.message);
}

// Test Storage API
try {
    console.log('localStorage exists:', typeof window.localStorage);
    console.log('sessionStorage exists:', typeof window.sessionStorage);
} catch (e) {
    console.log('Storage API error:', e.message);
}

// Test Navigator API
try {
    console.log('navigator exists:', typeof window.navigator);
    console.log('navigator.userAgent exists:', typeof window.navigator.userAgent);
    console.log('navigator.locks exists:', typeof window.navigator.locks);
} catch (e) {
    console.log('Navigator API error:', e.message);
}

// Test Timers API
try {
    console.log('setTimeout exists:', typeof setTimeout);
    console.log('setInterval exists:', typeof setInterval);
    console.log('clearTimeout exists:', typeof clearTimeout);
    console.log('clearInterval exists:', typeof clearInterval);
} catch (e) {
    console.log('Timers API error:', e.message);
}

console.log('API testing complete.');