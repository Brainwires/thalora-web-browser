// Test native localStorage implementation from Boa
console.log("Testing native localStorage from Boa engine...");

// Test basic functionality
console.log("typeof window.localStorage:", typeof window.localStorage);
console.log("typeof localStorage:", typeof localStorage);

// Test localStorage methods
console.log("localStorage.length before:", localStorage.length);
localStorage.setItem('test', 'value');
console.log("localStorage.length after setItem:", localStorage.length);
console.log("localStorage.getItem('test'):", localStorage.getItem('test'));

localStorage.setItem('number', 42);
console.log("localStorage.getItem('number'):", localStorage.getItem('number'), typeof localStorage.getItem('number'));

localStorage.setItem('object', {a: 1});
console.log("localStorage.getItem('object'):", localStorage.getItem('object'));

// Test key method
console.log("localStorage.key(0):", localStorage.key(0));
console.log("localStorage.key(1):", localStorage.key(1));

// Test removeItem
localStorage.removeItem('test');
console.log("localStorage.length after removeItem:", localStorage.length);
console.log("localStorage.getItem('test') after remove:", localStorage.getItem('test'));

// Test sessionStorage separation
console.log("typeof sessionStorage:", typeof sessionStorage);
sessionStorage.setItem('session', 'data');
console.log("sessionStorage.getItem('session'):", sessionStorage.getItem('session'));
console.log("localStorage.getItem('session'):", localStorage.getItem('session'));

// Test clear
localStorage.clear();
console.log("localStorage.length after clear:", localStorage.length);

console.log("Native localStorage test completed successfully!");