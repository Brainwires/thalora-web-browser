// Simple canvas test to check current implementation
console.log("=== Canvas Feature Test ===");

// Test 1: Canvas element creation
try {
    const canvas = document.createElement('canvas');
    console.log("✅ Canvas element creation:", !!canvas);
    console.log("Canvas type:", typeof canvas);
    console.log("Canvas constructor:", canvas.constructor.name);
} catch (e) {
    console.log("❌ Canvas element creation failed:", e.message);
}

// Test 2: Canvas context
try {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    console.log("✅ Canvas 2D context:", !!ctx);
    console.log("Context type:", typeof ctx);
    if (ctx) {
        console.log("Context constructor:", ctx.constructor.name);

        // Test basic methods
        console.log("fillRect method:", typeof ctx.fillRect);
        console.log("strokeRect method:", typeof ctx.strokeRect);
        console.log("fillText method:", typeof ctx.fillText);
        console.log("measureText method:", typeof ctx.measureText);
    }
} catch (e) {
    console.log("❌ Canvas 2D context failed:", e.message);
}

// Test 3: Canvas properties
try {
    const canvas = document.createElement('canvas');
    console.log("Canvas width:", canvas.width);
    console.log("Canvas height:", canvas.height);
    canvas.width = 800;
    canvas.height = 600;
    console.log("After setting - width:", canvas.width, "height:", canvas.height);
} catch (e) {
    console.log("❌ Canvas properties failed:", e.message);
}

// Test 4: toDataURL
try {
    const canvas = document.createElement('canvas');
    const dataUrl = canvas.toDataURL();
    console.log("✅ toDataURL:", !!dataUrl);
    console.log("Data URL type:", typeof dataUrl);
    console.log("Data URL starts with 'data:':", dataUrl && dataUrl.startsWith('data:'));
} catch (e) {
    console.log("❌ toDataURL failed:", e.message);
}

console.log("=== End Canvas Test ===");