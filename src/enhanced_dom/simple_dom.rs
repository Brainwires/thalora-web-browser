use anyhow::Result;
use boa_engine::{Context, Source};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct SimpleDom {
    storage: Arc<Mutex<HashMap<String, String>>>,
    session_storage: Arc<Mutex<HashMap<String, String>>>,
}

impl SimpleDom {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            session_storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn setup_enhanced_dom_globals(&self, context: &mut Context) -> Result<()> {
        // Setup enhanced DOM with localStorage/sessionStorage using simple JavaScript
        context.eval(Source::from_bytes(r#"
            // Enhanced localStorage implementation
            var localStorage = {
                data: {},
                getItem: function(key) {
                    return this.data[key] || null;
                },
                setItem: function(key, value) {
                    this.data[key] = String(value);
                },
                removeItem: function(key) {
                    delete this.data[key];
                },
                clear: function() {
                    this.data = {};
                },
                get length() {
                    return Object.keys(this.data).length;
                },
                key: function(index) {
                    var keys = Object.keys(this.data);
                    return keys[index] || null;
                }
            };

            // Enhanced sessionStorage implementation
            var sessionStorage = {
                data: {},
                getItem: function(key) {
                    return this.data[key] || null;
                },
                setItem: function(key, value) {
                    this.data[key] = String(value);
                },
                removeItem: function(key) {
                    delete this.data[key];
                },
                clear: function() {
                    this.data = {};
                },
                get length() {
                    return Object.keys(this.data).length;
                },
                key: function(index) {
                    var keys = Object.keys(this.data);
                    return keys[index] || null;
                }
            };

            // Enhanced document object with better DOM API
            if (typeof document === 'undefined') {
                var document = {};
            }
            
            // Extend existing document with enhanced methods
            document.getElementById = document.getElementById || function(id) {
                return {
                    id: id,
                    tagName: 'DIV',
                    nodeName: 'DIV',
                    innerHTML: '',
                    textContent: '',
                    className: '',
                    style: {},
                    classList: {
                        add: function(className) { console.log('Adding class:', className); },
                        remove: function(className) { console.log('Removing class:', className); },
                        toggle: function(className) { console.log('Toggling class:', className); return true; },
                        contains: function(className) { console.log('Checking class:', className); return false; }
                    },
                    getAttribute: function(name) { return this[name] || null; },
                    setAttribute: function(name, value) { this[name] = value; },
                    appendChild: function(child) { console.log('Appending child'); return child; },
                    removeChild: function(child) { console.log('Removing child'); return child; },
                    addEventListener: function(type, listener, options) { 
                        console.log('Event listener added:', type); 
                    },
                    removeEventListener: function(type, listener, options) { 
                        console.log('Event listener removed:', type); 
                    },
                    querySelector: function(selector) {
                        return document.createElement('div');
                    },
                    querySelectorAll: function(selector) {
                        return [document.createElement('div')];
                    },
                    click: function() { console.log('Element clicked'); }
                };
            };

            document.querySelector = document.querySelector || function(selector) {
                return document.getElementById('element_' + selector.replace(/[#.\s]/g, '_'));
            };

            document.querySelectorAll = document.querySelectorAll || function(selector) {
                return [
                    document.getElementById('el1_' + selector),
                    document.getElementById('el2_' + selector)
                ];
            };

            document.createElement = document.createElement || function(tagName) {
                return {
                    tagName: tagName.toUpperCase(),
                    nodeName: tagName.toUpperCase(),
                    innerHTML: '',
                    textContent: '',
                    className: '',
                    id: '',
                    style: {},
                    classList: {
                        add: function(className) { 
                            var classes = (this.parentElement || {className: ''}).className.split(' ');
                            if (classes.indexOf(className) === -1) {
                                classes.push(className);
                                (this.parentElement || {className: ''}).className = classes.join(' ').trim();
                            }
                        },
                        remove: function(className) {
                            var classes = (this.parentElement || {className: ''}).className.split(' ');
                            var index = classes.indexOf(className);
                            if (index > -1) {
                                classes.splice(index, 1);
                                (this.parentElement || {className: ''}).className = classes.join(' ').trim();
                            }
                        },
                        toggle: function(className) {
                            var classes = (this.parentElement || {className: ''}).className.split(' ');
                            var index = classes.indexOf(className);
                            if (index > -1) {
                                classes.splice(index, 1);
                                (this.parentElement || {className: ''}).className = classes.join(' ').trim();
                                return false;
                            } else {
                                classes.push(className);
                                (this.parentElement || {className: ''}).className = classes.join(' ').trim();
                                return true;
                            }
                        },
                        contains: function(className) {
                            var classes = (this.parentElement || {className: ''}).className.split(' ');
                            return classes.indexOf(className) > -1;
                        }
                    },
                    getAttribute: function(name) { return this[name] || null; },
                    setAttribute: function(name, value) { this[name] = value; },
                    appendChild: function(child) { 
                        child.parentElement = this;
                        return child; 
                    },
                    removeChild: function(child) { 
                        child.parentElement = null;
                        return child; 
                    },
                    addEventListener: function(type, listener, options) { 
                        console.log('Event listener added to', tagName, ':', type); 
                    },
                    removeEventListener: function(type, listener, options) { 
                        console.log('Event listener removed from', tagName, ':', type); 
                    },
                    querySelector: function(selector) {
                        return document.createElement('div');
                    },
                    querySelectorAll: function(selector) {
                        return [document.createElement('div')];
                    },
                    click: function() { 
                        console.log(tagName + ' element clicked'); 
                        // Simulate click event
                        if (this.onclick) {
                            this.onclick();
                        }
                    }
                };
            };

            // Enhance document with additional properties
            document.readyState = document.readyState || 'complete';
            document.title = document.title || '';
            document.URL = document.URL || 'about:blank';
            document.documentElement = document.documentElement || document.createElement('html');
            
            // Create body and head if they don't exist
            if (!document.body) {
                document.body = document.createElement('body');
                document.body.id = 'body';
            }
            if (!document.head) {
                document.head = document.createElement('head');
                document.head.id = 'head';
            }

            // Additional DOM methods
            document.addEventListener = document.addEventListener || function(type, listener, options) {
                console.log('Document event listener added:', type);
            };

            document.removeEventListener = document.removeEventListener || function(type, listener, options) {
                console.log('Document event listener removed:', type);
            };

            document.dispatchEvent = document.dispatchEvent || function(event) {
                console.log('Document event dispatched:', event.type || 'unknown');
                return true;
            };

            // Enhanced console for better debugging
            if (typeof console === 'undefined') {
                var console = {};
            }
            console.debug = console.debug || console.log || function() {};
            console.info = console.info || console.log || function() {};
            console.warn = console.warn || console.log || function() {};
            console.error = console.error || console.log || function() {};
            console.trace = console.trace || console.log || function() {};

        "#)).map_err(|e| anyhow::anyhow!("Failed to setup enhanced DOM: {}", e))?;

        Ok(())
    }

    pub fn get_storage_data(&self) -> HashMap<String, String> {
        self.storage.lock().unwrap().clone()
    }

    pub fn get_session_storage_data(&self) -> HashMap<String, String> {
        self.session_storage.lock().unwrap().clone()
    }

    pub fn clear_storage(&self) {
        self.storage.lock().unwrap().clear();
        self.session_storage.lock().unwrap().clear();
    }
}