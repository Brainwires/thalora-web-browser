/*
 * Fingerprint surface dump.
 *
 * Walks the global object to a bounded depth and describes what a page
 * script can observe about the JavaScript environment: global property
 * names (in enumeration order) and descriptors, function name/length and
 * whether Function.prototype.toString looks native, prototype chains,
 * interface prototypes, namespace objects, navigator/screen scalar values
 * and error-stack formats.
 *
 * Self-contained and ES5-style so it runs unchanged in any engine. Defines
 * one global, `__fingerprintDump()`, which returns canonical JSON (object
 * keys sorted, so equal surfaces give byte-identical output) including a
 * SHA-256 `digest` of everything else. dump.html calls it and prints the
 * result into <pre id="out">. `cargo xtask fingerprint-dump` drives it
 * inside Thalora through `__fingerprintDump.begin()/step(ms)/finish()`, which
 * splits the walk across evaluations (Thalora caps each one at 5 s).
 *
 * Property descriptor encoding (one compact string per property):
 *   data:     "v|<wec>|<typeof>"                 (non-function, non-object)
 *             "v|<wec>|object|<ctor>>...>Object"  (object: prototype chain)
 *             "v|<wec>|function|<name>|<length>|<N or X>"
 *   accessor: "a|<-ec>|get:<N|X|->|set:<N|X|->"
 * <wec> are the writable/enumerable/configurable flags ('-' when false).
 * N = Function.prototype.toString output ends in "{ [native code] }",
 * X = it does not, '-' = no getter/setter.
 */
(function (global) {
  'use strict';

  var MAX_MEMBERS = 2000; // per object, keeps the dump bounded
  var NATIVE_RE = /\{\s*\[native code\]\s*\}$/;
  var fnToString = Function.prototype.toString;
  var hasOwn = Object.prototype.hasOwnProperty;
  var objToString = Object.prototype.toString;
  var SELF = '__fingerprintDump';
  // Globals that alias the global object itself; never recursed into
  var SELF_REFS = { window: 1, self: 1, top: 1, parent: 1, frames: 1, globalThis: 1 };

  function attempt(fn, fallback) {
    try {
      return fn();
    } catch (e) {
      return fallback;
    }
  }

  /** Value of `fn()`, with undefined and exceptions spelled out (JSON-safe). */
  function read(fn) {
    var v = attempt(fn, '<throws>');
    return v === undefined ? '<undefined>' : v;
  }

  function isNative(fn) {
    return attempt(function () {
      return NATIVE_RE.test(fnToString.call(fn));
    }, false);
  }

  function keyName(key) {
    return typeof key === 'symbol' ? '@@' + attempt(function () {
      return key.description !== undefined ? key.description : String(key).slice(7, -1);
    }, '?') : key;
  }

  function ownKeys(obj) {
    return attempt(function () {
      if (typeof Reflect !== 'undefined' && Reflect.ownKeys) return Reflect.ownKeys(obj);
      return Object.getOwnPropertyNames(obj);
    }, []);
  }

  function protoName(p) {
    return attempt(function () {
      if (hasOwn.call(p, 'constructor') && typeof p.constructor === 'function') {
        var n = p.constructor.name;
        if (n) return n;
      }
      return objToString.call(p).slice(8, -1);
    }, '?');
  }

  /** Constructor names along the prototype chain of `obj` (excluding obj). */
  function chain(obj) {
    var names = [];
    var p = attempt(function () { return Object.getPrototypeOf(obj); }, null);
    while (p && names.length < 30) {
      names.push(protoName(p));
      p = attempt(function () { return Object.getPrototypeOf(p); }, null);
    }
    return names;
  }

  function flags(d) {
    return (d.writable ? 'w' : '-') + (d.enumerable ? 'e' : '-') + (d.configurable ? 'c' : '-');
  }

  function fnMark(fn) {
    return typeof fn === 'function' ? (isNative(fn) ? 'N' : 'X') : '-';
  }

  function describeValue(v) {
    var t = typeof v;
    if (t === 'function') {
      var name = attempt(function () { return String(v.name); }, '?');
      var length = attempt(function () { return String(v.length); }, '?');
      return 'function|' + name.replace(/\|/g, '/') + '|' + length + '|' + fnMark(v);
    }
    if (t === 'object' && v !== null) return 'object|' + chain(v).join('>');
    if (v === null) return 'null';
    return t;
  }

  function describe(obj, key) {
    var d = attempt(function () { return Object.getOwnPropertyDescriptor(obj, key); }, undefined);
    if (!d) return 'missing';
    if (hasOwn.call(d, 'value') || hasOwn.call(d, 'writable')) {
      return 'v|' + flags(d) + '|' + describeValue(d.value);
    }
    return 'a|' + flags(d) + '|get:' + fnMark(d.get) + '|set:' + fnMark(d.set);
  }

  function members(obj) {
    var out = {};
    var keys = ownKeys(obj);
    for (var i = 0; i < keys.length && i < MAX_MEMBERS; i++) {
      out[keyName(keys[i])] = describe(obj, keys[i]);
    }
    return out;
  }

  function dataValue(obj, key) {
    var d = attempt(function () { return Object.getOwnPropertyDescriptor(obj, key); }, undefined);
    return d && hasOwn.call(d, 'value') ? d.value : undefined;
  }

  function isScalar(v) {
    var t = typeof v;
    return v === null || t === 'string' || t === 'number' || t === 'boolean' || t === 'undefined';
  }

  /** Scalar (and array-of-scalar) values readable through `obj`, keyed by name. */
  function scalars(obj) {
    var out = {};
    if (!obj) return out;
    var seen = {};
    var names = [];
    attempt(function () {
      for (var k in obj) names.push(k);
    }, null);
    // Non-enumerable accessors on the prototype chain (engine-dependent)
    var p = attempt(function () { return Object.getPrototypeOf(obj); }, null);
    while (p && p !== Object.prototype) {
      names = names.concat(attempt(function () { return Object.getOwnPropertyNames(p); }, []));
      p = attempt(function () { return Object.getPrototypeOf(p); }, null);
    }
    for (var i = 0; i < names.length; i++) {
      var k = names[i];
      if (seen[k] || k === 'constructor') continue;
      seen[k] = 1;
      var v = attempt(function () { return obj[k]; }, '<throws>');
      if (isScalar(v)) {
        out[k] = v === undefined ? '<undefined>' : v;
      } else if (Array.isArray(v) && v.every(isScalar)) {
        out[k] = v.slice();
      }
    }
    return out;
  }

  // URLs vary with how the page is served, and frames below the probe depend
  // on the caller (inline script vs. a devtools/MCP evaluation), so keep the
  // message plus the innermost three frames only.
  function normalizeStack(s) {
    return String(s)
      .replace(/(?:https?|file|blob):\/\/[^\s)]*/g, '<url>')
      .replace(/\r\n/g, '\n')
      .split('\n')
      .slice(0, 4)
      .join('\n');
  }

  function errorSample(make) {
    var e = attempt(make, null);
    if (!e) return null;
    return {
      ctor: protoName(attempt(function () { return Object.getPrototypeOf(e); }, null) || {}),
      message: attempt(function () { return String(e.message); }, '?'),
      ownKeys: attempt(function () { return Object.getOwnPropertyNames(e).sort(); }, []),
      stack: attempt(function () { return e.stack === undefined ? '<undefined>' : normalizeStack(e.stack); }, '<throws>'),
      toString: attempt(function () { return String(e); }, '?'),
    };
  }

  function errors() {
    function outerProbe() { return innerProbe(); }
    function innerProbe() { return new Error('probe'); }
    return {
      constructed: errorSample(outerProbe),
      typeError: errorSample(function () {
        try {
          var n = null;
          return n.missingProperty;
        } catch (e) {
          return e;
        }
      }),
      referenceError: errorSample(function () {
        try {
          return eval('undefinedFingerprintProbe');
        } catch (e) {
          return e;
        }
      }),
      stackTraceLimit: attempt(function () { return Error.stackTraceLimit === undefined ? '<undefined>' : Error.stackTraceLimit; }, '<throws>'),
      captureStackTrace: typeof Error.captureStackTrace,
      errorPrototypeStack: describe(Error.prototype, 'stack'),
    };
  }

  function toStringSamples() {
    var getter = attempt(function () {
      return Object.getOwnPropertyDescriptor(Object.getPrototypeOf(global.navigator), 'userAgent').get;
    }, null);
    return {
      fnToString: attempt(function () { return fnToString.call(fnToString); }, '<throws>'),
      boundFunction: attempt(function () { return fnToString.call(function () {}.bind(null)); }, '<throws>'),
      navigatorUserAgentGetter: getter ? attempt(function () { return fnToString.call(getter); }, '<throws>') : '<none>',
      fnToStringOfToString: attempt(function () { return fnToString.call(Function.prototype.toString.toString); }, '<throws>'),
      objectToStringWindow: attempt(function () { return objToString.call(global); }, '<throws>'),
    };
  }

  function environment() {
    return {
      devicePixelRatio: read(function () { return global.devicePixelRatio; }),
      innerWidth: read(function () { return global.innerWidth; }),
      innerHeight: read(function () { return global.innerHeight; }),
      outerWidth: read(function () { return global.outerWidth; }),
      outerHeight: read(function () { return global.outerHeight; }),
      timezoneOffset: read(function () { return new Date(0).getTimezoneOffset(); }),
      timeZone: read(function () { return Intl.DateTimeFormat().resolvedOptions().timeZone; }),
      locale: read(function () { return Intl.DateTimeFormat().resolvedOptions().locale; }),
      dateString: read(function () { return new Date(0).toString(); }),
    };
  }

  function chains() {
    var doc = attempt(function () { return global.document; }, null);
    var targets = {
      window: global,
      document: doc,
      documentElement: doc && attempt(function () { return doc.documentElement; }, null),
      body: doc && attempt(function () { return doc.body; }, null),
      navigator: attempt(function () { return global.navigator; }, null),
      screen: attempt(function () { return global.screen; }, null),
      location: attempt(function () { return global.location; }, null),
      history: attempt(function () { return global.history; }, null),
      performance: attempt(function () { return global.performance; }, null),
      error: attempt(function () { return new Error('x'); }, null),
    };
    var out = {};
    for (var k in targets) {
      out[k] = targets[k] ? chain(targets[k]) : null;
    }
    return out;
  }

  /** Facts about `window` itself, which a page may see as distinct from the global. */
  function windowFacts() {
    var w = attempt(function () { return global.window; }, undefined);
    var names = w ? attempt(function () { return Object.getOwnPropertyNames(w); }, []) : [];
    names = names.filter(function (n) { return n !== SELF; });
    var same = w === global;
    return {
      isGlobalThis: same,
      ownPropertyCount: names.length,
      // Only listed when they differ from `globals.order`
      ownPropertyNames: same ? null : names,
    };
  }

  /**
   * Incremental walker: `step(budgetMs)` processes globals until the time
   * budget runs out, so engines with a per-evaluation time limit can run the
   * dump across several evaluations.
   */
  function Walker() {
    var names = attempt(function () { return Object.getOwnPropertyNames(global); }, []);
    this.names = names.filter(function (n) { return n !== SELF; });
    this.index = 0;
    this.props = {};
    this.interfaces = {};
    this.namespaces = {};
  }

  Walker.prototype.visit = function (name) {
    this.props[name] = describe(global, name);
    var v = dataValue(global, name);
    if (typeof v === 'function') {
      var proto = dataValue(v, 'prototype');
      if (/^[A-Z]/.test(name) && proto && typeof proto === 'object') {
        this.interfaces[name] = {
          inherits: attempt(function () { return Object.getPrototypeOf(v).name || ''; }, '?'),
          prototype: members(proto),
          prototypeChain: chain(proto),
          static: members(v),
        };
      }
    } else if (v && typeof v === 'object' && !SELF_REFS[name] && v !== global) {
      this.namespaces[name] = members(v);
    }
  };

  /** Visit globals for up to `budgetMs`; returns how many remain. */
  Walker.prototype.step = function (budgetMs) {
    var start = Date.now();
    while (this.index < this.names.length) {
      this.visit(this.names[this.index++]);
      if (Date.now() - start >= budgetMs) break;
    }
    return this.names.length - this.index;
  };

  Walker.prototype.body = function () {
    var symbols = attempt(function () {
      return Object.getOwnPropertySymbols(global).map(keyName);
    }, []);
    return {
      format: 'thalora-fingerprint-dump/1',
      globals: {
        count: this.names.length,
        order: this.names,
        props: this.props,
        symbols: symbols,
      },
      window: windowFacts(),
      interfaces: this.interfaces,
      namespaces: this.namespaces,
      values: {
        navigator: scalars(attempt(function () { return global.navigator; }, null)),
        screen: scalars(attempt(function () { return global.screen; }, null)),
        environment: environment(),
      },
      chains: chains(),
      errors: errors(),
      toStringSamples: toStringSamples(),
    };
  };

  // ---- canonical JSON (sorted keys) --------------------------------------

  function canonical(v) {
    if (v === null || typeof v !== 'object') {
      return JSON.stringify(v === undefined ? null : v);
    }
    if (Array.isArray(v)) {
      return '[' + v.map(canonical).join(',') + ']';
    }
    var keys = Object.keys(v).sort();
    var parts = [];
    for (var i = 0; i < keys.length; i++) {
      parts.push(JSON.stringify(keys[i]) + ':' + canonical(v[keys[i]]));
    }
    return '{' + parts.join(',') + '}';
  }

  // ---- SHA-256 (no dependency on TextEncoder or crypto.subtle) -----------

  function utf8(s) {
    var out = [];
    for (var i = 0; i < s.length; i++) {
      var c = s.charCodeAt(i);
      if (c >= 0xd800 && c < 0xdc00 && i + 1 < s.length) {
        var lo = s.charCodeAt(i + 1);
        if (lo >= 0xdc00 && lo < 0xe000) {
          c = 0x10000 + ((c - 0xd800) << 10) + (lo - 0xdc00);
          i++;
        }
      }
      if (c < 0x80) out.push(c);
      else if (c < 0x800) out.push(0xc0 | (c >> 6), 0x80 | (c & 63));
      else if (c < 0x10000) out.push(0xe0 | (c >> 12), 0x80 | ((c >> 6) & 63), 0x80 | (c & 63));
      else out.push(0xf0 | (c >> 18), 0x80 | ((c >> 12) & 63), 0x80 | ((c >> 6) & 63), 0x80 | (c & 63));
    }
    return out;
  }

  var K = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
  ];

  function sha256(s) {
    var bytes = utf8(s);
    var bitLen = bytes.length * 8;
    bytes.push(0x80);
    while (bytes.length % 64 !== 56) bytes.push(0);
    var hi = Math.floor(bitLen / 0x100000000);
    var lo = bitLen >>> 0;
    bytes.push((hi >>> 24) & 255, (hi >>> 16) & 255, (hi >>> 8) & 255, hi & 255);
    bytes.push((lo >>> 24) & 255, (lo >>> 16) & 255, (lo >>> 8) & 255, lo & 255);
    var h = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    var w = new Array(64);
    function rotr(x, n) { return (x >>> n) | (x << (32 - n)); }
    for (var off = 0; off < bytes.length; off += 64) {
      for (var t = 0; t < 16; t++) {
        var j = off + t * 4;
        w[t] = ((bytes[j] << 24) | (bytes[j + 1] << 16) | (bytes[j + 2] << 8) | bytes[j + 3]) | 0;
      }
      for (t = 16; t < 64; t++) {
        var s0 = rotr(w[t - 15], 7) ^ rotr(w[t - 15], 18) ^ (w[t - 15] >>> 3);
        var s1 = rotr(w[t - 2], 17) ^ rotr(w[t - 2], 19) ^ (w[t - 2] >>> 10);
        w[t] = (w[t - 16] + s0 + w[t - 7] + s1) | 0;
      }
      var a = h[0], b = h[1], c = h[2], d = h[3], e = h[4], f = h[5], g = h[6], k = h[7];
      for (t = 0; t < 64; t++) {
        var S1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
        var ch = (e & f) ^ (~e & g);
        var t1 = (k + S1 + ch + K[t] + w[t]) | 0;
        var S0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
        var maj = (a & b) ^ (a & c) ^ (b & c);
        var t2 = (S0 + maj) | 0;
        k = g; g = f; f = e; e = (d + t1) | 0;
        d = c; c = b; b = a; a = (t1 + t2) | 0;
      }
      h[0] = (h[0] + a) | 0; h[1] = (h[1] + b) | 0; h[2] = (h[2] + c) | 0; h[3] = (h[3] + d) | 0;
      h[4] = (h[4] + e) | 0; h[5] = (h[5] + f) | 0; h[6] = (h[6] + g) | 0; h[7] = (h[7] + k) | 0;
    }
    var hex = '';
    for (var x = 0; x < 8; x++) hex += ('00000000' + (h[x] >>> 0).toString(16)).slice(-8);
    return hex;
  }

  // One-shot: the whole dump as canonical JSON with its digest
  var api = function () {
    var walker = new Walker();
    walker.step(Infinity);
    var body = walker.body();
    body.digest = 'sha256:' + sha256(canonical(body));
    return canonical(body);
  };
  // Incremental: begin() -> step(ms) until it returns 0 -> finish(). finish()
  // returns plain JSON without `digest`; the caller canonicalizes and hashes
  // it (cargo xtask fingerprint-dump does, with the same canonical form).
  var state = null;
  api.begin = function () {
    state = new Walker();
    return state.names.length;
  };
  api.step = function (budgetMs) {
    return state.step(budgetMs || 1000);
  };
  api.finish = function () {
    return JSON.stringify(state.body());
  };
  global[SELF] = api;
})(typeof globalThis !== 'undefined' ? globalThis : this);
