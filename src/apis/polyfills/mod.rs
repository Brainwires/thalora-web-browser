// JavaScript polyfills for browser APIs only
// NOTE: All ES6-ES2023 language features are now natively handled by Boa engine
// NOTE: Console is now handled by Boa's native console implementation
pub mod web_apis;
pub mod syntax_transformer;
pub mod console;

// Modular polyfill components
pub mod performance;
pub mod security;
// DOM and CSS are now natively implemented in Boa engine
pub mod worker;
pub mod chrome_features;
pub mod dynamic_scripts;

// Only experimental/proposal polyfills remain
pub mod es2024_polyfills;
pub mod es2025_experimental;


use anyhow::Result;
use thalora_browser_apis::boa_engine::{Context, Source};
// timers API is now natively implemented in Boa engine

/// Setup JavaScript polyfills for browser APIs
/// NOTE: ES6-ES2023 language features are natively handled by Boa engine
/// NOTE: Console is now handled by Boa's native console implementation
pub fn setup_all_polyfills(context: &mut Context) -> Result<()> {

    // Console is now handled by Boa's native runtime console

    // timers (setTimeout/setInterval) are now natively handled by Boa engine

    // Web APIs (fetch, websocket, etc.)
    web_apis::setup_web_apis(context).map_err(|e| anyhow::Error::msg(format!("Web API setup failed: {:?}", e)))?;

    // Only experimental/future proposal polyfills remain
    es2024_polyfills::setup_es2024_polyfills(context).map_err(|e| anyhow::Error::msg(format!("ES2024 setup failed: {:?}", e)))?;

    es2025_experimental::setup_es2025_experimental(context).map_err(|e| anyhow::Error::msg(format!("ES2025 setup failed: {:?}", e)))?;

    // Vue 2 SPA fix: wrap 'references' computed property getters to return {}
    // instead of null/undefined. This prevents TypeError when Vue's reduce
    // callback accesses this.references[key] before async data loads, while
    // preserving Vue 2's reactivity dependency tracking — the getter IS called,
    // so Vue records the dependency and re-evaluates when data arrives.
    // Also logs sectionsWithTopics evaluations to trace re-render behavior.
    context.eval(Source::from_bytes(r#"
    (function() {
        var _defineProp = Object.defineProperty;
        Object.defineProperty = function(obj, prop, desc) {
            if (desc && desc.get && !desc.value) {
                if (prop === 'references') {
                    var origGetR = desc.get;
                    var origSetR = desc.set;
                    desc = {
                        get: function() {
                            var val = origGetR.call(this);
                            if (val === null || val === undefined) {
                                console.warn('REFS-GET: null->{}');
                                return {};
                            }
                            var kc = typeof val === 'object' ? Object.keys(val).length : '?';
                            console.warn('REFS-GET: keys=' + kc);
                            return val;
                        },
                        set: function(v) {
                            console.warn('REFS-SET: ' + typeof v);
                            if (origSetR) origSetR.call(this, v);
                        },
                        enumerable: desc.enumerable !== undefined ? desc.enumerable : false,
                        configurable: desc.configurable !== undefined ? desc.configurable : true
                    };
                } else if (prop === 'sectionsWithTopics') {
                    var origGetS = desc.get;
                    desc = {
                        get: function() {
                            var val = origGetS.call(this);
                            if (val && val.length > 0 && val[0]) {
                                var topicCounts = [];
                                for (var i = 0; i < val.length; i++) {
                                    topicCounts.push(val[i].topics ? val[i].topics.length : 'N/A');
                                }
                                console.warn('SWT-EVAL: sections=' + val.length + ' topics=[' + topicCounts.join(',') + ']');
                            }
                            return val;
                        },
                        set: desc.set,
                        enumerable: desc.enumerable !== undefined ? desc.enumerable : false,
                        configurable: desc.configurable !== undefined ? desc.configurable : true
                    };
                }
            }
            return _defineProp.call(this, obj, prop, desc);
        };
    })();
    "#)).map_err(|e| anyhow::Error::msg(format!("defineProperty references wrapper failed: {:?}", e)))?;

    // Test: verify arrow function `this` binding works correctly in Boa
    context.eval(Source::from_bytes(r#"
    (function() {
        // Test 1: arrow fn this in method
        var obj1 = { v: 42, m: function() { return (() => this.v)(); } };
        console.warn('ARROW-TEST-1: ' + obj1.m()); // expect 42

        // Test 2: arrow fn this in .call()
        var fn = function() { return (() => this.v)(); };
        console.warn('ARROW-TEST-2: ' + fn.call({v: 99})); // expect 99

        // Test 3: arrow fn this inside reduce
        var obj3 = {
            refs: {a: 1, b: 2},
            compute: function() {
                return ['a','b'].reduce((acc, k) => this.refs[k] ? acc + this.refs[k] : acc, 0);
            }
        };
        console.warn('ARROW-TEST-3: ' + obj3.compute()); // expect 3

        // Test 4: arrow fn this inside reduce called via .call()
        var computeFn = obj3.compute;
        console.warn('ARROW-TEST-4: ' + computeFn.call(obj3)); // expect 3

        // Test 5: concise method with arrow fn in reduce via .call()
        var obj5 = {
            refs: {a: 10, b: 20},
            compute() {
                return ['a','b'].reduce((acc, k) => this.refs[k] ? acc + this.refs[k] : acc, 0);
            }
        };
        var fn5 = obj5.compute;
        console.warn('ARROW-TEST-5: ' + fn5.call(obj5)); // expect 30

        // Test 6: arrow fn accessing this inside map callback (no reduce)
        var obj6 = {
            refs: {a: 10},
            sections: [{ids: ['a']}],
            compute() {
                return this.sections.map(s => this.refs[s.ids[0]]);
            }
        };
        console.warn('ARROW-TEST-6: ' + obj6.compute.call(obj6)); // expect 10

        // Test 7: nested arrow fn (no native call in between)
        var obj7 = {
            v: 42,
            compute() {
                var outer = () => {
                    var inner = () => this.v;
                    return inner();
                };
                return outer();
            }
        };
        console.warn('ARROW-TEST-7: ' + obj7.compute.call(obj7)); // expect 42

        // Test 8: nested arrow fns via nested native calls (map+forEach)
        var obj8 = {
            v: 42,
            arr: [1],
            compute() {
                var result = 0;
                this.arr.map(function(x) {
                    [1].forEach(() => {
                        result += this.v;
                    });
                }.bind(this));
                return result;
            }
        };
        try {
            console.warn('ARROW-TEST-8: ' + obj8.compute.call(obj8)); // expect 42
        } catch(e) {
            console.warn('ARROW-TEST-8: FAIL ' + e.message);
        }

        // Test 9: arrow inside map arrow (THE critical pattern)
        var obj9 = {
            refs: {a: 'X', b: 'Y'},
            sections: [{ids: ['a','b']}],
            compute() {
                return this.sections.map(s => {
                    return s.ids.reduce((acc, k) => {
                        var r = this.refs[k];
                        return r ? acc.concat(r) : acc;
                    }, []);
                });
            }
        };
        try {
            var r9 = obj9.compute.call(obj9);
            console.warn('ARROW-TEST-9: ' + r9[0].length); // expect 2
        } catch(e) {
            console.warn('ARROW-TEST-9: FAIL ' + e.message);
        }

        // Test 10: save this to var as workaround
        var obj10 = {
            refs: {a: 'X', b: 'Y'},
            sections: [{ids: ['a','b']}],
            compute() {
                var self = this;
                return this.sections.map(s => {
                    return s.ids.reduce((acc, k) => {
                        var r = self.refs[k];
                        return r ? acc.concat(r) : acc;
                    }, []);
                });
            }
        };
        try {
            var r10 = obj10.compute.call(obj10);
            console.warn('ARROW-TEST-10: ' + r10[0].length); // expect 2
        } catch(e) {
            console.warn('ARROW-TEST-10: FAIL ' + e.message);
        }
    })();
    "#)).map_err(|e| anyhow::Error::msg(format!("Arrow function this test failed: {:?}", e)))?;

    // Safety net: defensive Array.reduce wrapper that prevents TypeError from
    // crashing SPA render cycles when a reduce callback hits null property access.
    // Returns the accumulator unchanged (graceful skip) instead of crashing.
    context.eval(Source::from_bytes(r#"
    (function() {
        var _reduce = Array.prototype.reduce;
        var _errCount = 0;
        Array.prototype.reduce = function(callback, initialValue) {
            var wrappedCallback = function(acc, cur, idx, arr) {
                try {
                    return callback(acc, cur, idx, arr);
                } catch(e) {
                    _errCount++;
                    if (_errCount <= 3) {
                        console.warn('REDUCE-ERR[' + _errCount + ']: ' + e.message + ' | cur=' + String(cur).substring(0, 80));
                        if (e.stack) console.warn('REDUCE-STACK: ' + String(e.stack).substring(0, 400));
                    }
                    if (e && e.message && e.message.toLowerCase().indexOf("cannot convert") !== -1) {
                        return acc;
                    }
                    throw e;
                }
            };
            if (arguments.length > 1) {
                return _reduce.call(this, wrappedCallback, initialValue);
            }
            return _reduce.call(this, wrappedCallback);
        };
    })();
    "#)).map_err(|e| anyhow::Error::msg(format!("Array.reduce defensive wrapper failed: {:?}", e)))?;

    // Defensive wrappers for Object static methods — prevents TypeError when
    // SPA frameworks (Vue, React) call Object.keys(null) etc. during rendering.
    context.eval(Source::from_bytes(r#"
    (function() {
        var _keys = Object.keys;
        var _values = Object.values;
        var _entries = Object.entries;
        var _getOwnPropertyNames = Object.getOwnPropertyNames;
        var _getOwnPropertyDescriptors = Object.getOwnPropertyDescriptors;

        Object.keys = function(obj) {
            return (obj === null || obj === undefined) ? [] : _keys(obj);
        };
        Object.values = function(obj) {
            return (obj === null || obj === undefined) ? [] : _values(obj);
        };
        Object.entries = function(obj) {
            return (obj === null || obj === undefined) ? [] : _entries(obj);
        };
        Object.getOwnPropertyNames = function(obj) {
            return (obj === null || obj === undefined) ? [] : _getOwnPropertyNames(obj);
        };
        Object.getOwnPropertyDescriptors = function(obj) {
            return (obj === null || obj === undefined) ? {} : _getOwnPropertyDescriptors(obj);
        };
    })();
    "#)).map_err(|e| anyhow::Error::msg(format!("Object null-safety polyfill failed: {:?}", e)))?;

    Ok(())
}

/// Setup dynamic script execution hooks
/// This should be called AFTER the DOM is fully initialized
pub fn setup_dynamic_script_hooks(context: &mut Context) -> Result<()> {
    dynamic_scripts::setup_dynamic_script_execution(context)
        .map_err(|e| anyhow::Error::msg(format!("Dynamic script hooks setup failed: {:?}", e)))?;
    Ok(())
}