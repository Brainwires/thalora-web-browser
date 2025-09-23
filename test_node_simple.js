// Test Node interface directly
try {
    console.log("=== Node Interface Test ===");

    // Test Node constants
    console.log("Node.ELEMENT_NODE:", Node.ELEMENT_NODE);
    console.log("Node.TEXT_NODE:", Node.TEXT_NODE);
    console.log("Node.DOCUMENT_NODE:", Node.DOCUMENT_NODE);

    // Test Node constructor
    console.log("Node constructor exists:", typeof Node === 'function');

    // Create a test Node object directly
    var node = new Node();
    console.log("node created successfully");

    console.log("node.nodeType():", node.nodeType());
    console.log("node.nodeName():", node.nodeName());
    console.log("node.nodeValue():", node.nodeValue());

    // Test methods
    console.log("node.hasChildNodes():", node.hasChildNodes());

    // Test Node properties and methods
    console.log("node.parentNode():", node.parentNode());
    console.log("node.childNodes():", node.childNodes());
    console.log("node.firstChild():", node.firstChild());
    console.log("node.lastChild():", node.lastChild());
    console.log("node.previousSibling():", node.previousSibling());
    console.log("node.nextSibling():", node.nextSibling());
    console.log("node.ownerDocument():", node.ownerDocument());

    console.log("=== Node Interface Test Complete ===");

} catch(e) {
    console.error("Node interface test error:", e.message);
    console.error("Stack:", e.stack);
}