// Debug RegExp.escape availability
console.log("Testing RegExp global:", typeof RegExp);
console.log("Testing RegExp.escape:", typeof RegExp.escape);
console.log("RegExp object:", RegExp);
if (typeof RegExp !== 'undefined') {
    console.log("RegExp properties:", Object.getOwnPropertyNames(RegExp));
}
typeof RegExp !== 'undefined' && typeof RegExp.escape;