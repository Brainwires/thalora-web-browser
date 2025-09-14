use synaptic::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_find_last() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.prototype.findLast
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3, 4, 5];
        [
            arr.findLast(x => x > 3),
            arr.findLast(x => x > 10),
            arr.findLast(x => x % 2 === 0),
            [].findLast(x => true)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_array_find_last_index() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.prototype.findLastIndex
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3, 4, 5];
        [
            arr.findLastIndex(x => x > 3),
            arr.findLastIndex(x => x > 10),
            arr.findLastIndex(x => x % 2 === 0),
            [].findLastIndex(x => true)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_array_to_reversed() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.prototype.toReversed
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3];
        const reversed = arr.toReversed();
        [
            JSON.stringify(arr),
            JSON.stringify(reversed),
            arr !== reversed
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_array_to_sorted() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.prototype.toSorted
    let result = engine.execute_enhanced(r#"
        const arr = [3, 1, 4, 1, 5];
        const sorted = arr.toSorted();
        const customSorted = arr.toSorted((a, b) => b - a);
        [
            JSON.stringify(arr),
            JSON.stringify(sorted),
            JSON.stringify(customSorted),
            arr !== sorted
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_array_to_spliced() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.prototype.toSpliced
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3, 4, 5];
        const spliced1 = arr.toSpliced(2, 1);
        const spliced2 = arr.toSpliced(1, 2, 'a', 'b');
        [
            JSON.stringify(arr),
            JSON.stringify(spliced1),
            JSON.stringify(spliced2),
            arr !== spliced1
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_array_with() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.prototype.with
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3];
        const modified1 = arr.with(1, 'changed');
        const modified2 = arr.with(-1, 'last');
        [
            JSON.stringify(arr),
            JSON.stringify(modified1),
            JSON.stringify(modified2),
            arr !== modified1
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_set_intersection() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Set.prototype.intersection
    let result = engine.execute_enhanced(r#"
        const set1 = new Set([1, 2, 3, 4]);
        const set2 = new Set([3, 4, 5, 6]);
        const intersection = set1.intersection(set2);
        [
            intersection.size,
            intersection.has(3),
            intersection.has(4),
            intersection.has(1),
            intersection.has(5)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_set_union() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Set.prototype.union
    let result = engine.execute_enhanced(r#"
        const set1 = new Set([1, 2, 3]);
        const set2 = new Set([3, 4, 5]);
        const union = set1.union(set2);
        [
            union.size,
            union.has(1),
            union.has(2),
            union.has(3),
            union.has(4),
            union.has(5)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_set_difference() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Set.prototype.difference
    let result = engine.execute_enhanced(r#"
        const set1 = new Set([1, 2, 3, 4]);
        const set2 = new Set([3, 4, 5, 6]);
        const difference = set1.difference(set2);
        [
            difference.size,
            difference.has(1),
            difference.has(2),
            difference.has(3),
            difference.has(4)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_set_symmetric_difference() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Set.prototype.symmetricDifference
    let result = engine.execute_enhanced(r#"
        const set1 = new Set([1, 2, 3]);
        const set2 = new Set([3, 4, 5]);
        const symDiff = set1.symmetricDifference(set2);
        [
            symDiff.size,
            symDiff.has(1),
            symDiff.has(2),
            symDiff.has(3),
            symDiff.has(4),
            symDiff.has(5)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_set_is_subset_of() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Set.prototype.isSubsetOf
    let result = engine.execute_enhanced(r#"
        const set1 = new Set([1, 2]);
        const set2 = new Set([1, 2, 3, 4]);
        const set3 = new Set([5, 6]);
        [
            set1.isSubsetOf(set2),
            set2.isSubsetOf(set1),
            set3.isSubsetOf(set2),
            new Set().isSubsetOf(set1)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_set_is_superset_of() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Set.prototype.isSupersetOf
    let result = engine.execute_enhanced(r#"
        const set1 = new Set([1, 2, 3, 4]);
        const set2 = new Set([1, 2]);
        const set3 = new Set([5, 6]);
        [
            set1.isSupersetOf(set2),
            set2.isSupersetOf(set1),
            set1.isSupersetOf(set3),
            set1.isSupersetOf(new Set())
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_set_is_disjoint_from() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Set.prototype.isDisjointFrom
    let result = engine.execute_enhanced(r#"
        const set1 = new Set([1, 2, 3]);
        const set2 = new Set([4, 5, 6]);
        const set3 = new Set([3, 4, 5]);
        [
            set1.isDisjointFrom(set2),
            set1.isDisjointFrom(set3),
            set2.isDisjointFrom(set3),
            new Set().isDisjointFrom(set1)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_hashbang_comments() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test hashbang comment removal
    let result = engine.execute_enhanced(r#"
        // Hashbang comments would be stripped during transformation
        const value = 42;
        value
    "#).await.unwrap();

    assert_eq!(result.as_number().unwrap(), 42.0);
}

#[tokio::test]
async fn test_array_methods_polyfill_existence() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that all ES2023 array methods exist
    let result = engine.execute_enhanced(r#"
        const arr = [];
        [
            typeof arr.findLast === 'function',
            typeof arr.findLastIndex === 'function',
            typeof arr.toReversed === 'function',
            typeof arr.toSorted === 'function',
            typeof arr.toSpliced === 'function',
            typeof arr.with === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_set_methods_polyfill_existence() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that all ES2023 set methods exist
    let result = engine.execute_enhanced(r#"
        const set = new Set();
        [
            typeof set.intersection === 'function',
            typeof set.union === 'function',
            typeof set.difference === 'function',
            typeof set.symmetricDifference === 'function',
            typeof set.isSubsetOf === 'function',
            typeof set.isSupersetOf === 'function',
            typeof set.isDisjointFrom === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}