use anyhow::Result;

/// Advanced syntax transformer for ES2022 features
pub struct SyntaxTransformer;

impl SyntaxTransformer {
    pub fn new() -> Self {
        Self
    }

    /// Transform ES2025+ syntax to ES5-compatible code
    pub fn transform_latest(&self, code: &str) -> Result<String> {
        let mut transformed = code.to_string();

        // ES2022+ transformations
        transformed = self.transform_nullish_coalescing(&transformed);
        transformed = self.transform_optional_chaining(&transformed);
        transformed = self.transform_logical_assignment(&transformed);
        transformed = self.transform_numeric_separators(&transformed);
        transformed = self.transform_private_fields(&transformed);
        transformed = self.transform_class_fields(&transformed);
        transformed = self.transform_top_level_await(&transformed);
        transformed = self.transform_bigint_literals(&transformed);
        transformed = self.transform_array_at(&transformed);

        // ES2023+ transformations
        transformed = self.transform_hashbang_comments(&transformed);
        transformed = self.transform_array_with(&transformed);
        transformed = self.transform_array_group_methods(&transformed);

        // ES2024+ transformations
        transformed = self.transform_regexp_v_flag(&transformed);
        transformed = self.transform_using_declarations(&transformed);
        transformed = self.transform_import_assertions(&transformed);

        // ES2025+ experimental transformations
        transformed = self.transform_pattern_matching(&transformed);
        transformed = self.transform_pipeline_operator(&transformed);
        transformed = self.transform_partial_application(&transformed);
        transformed = self.transform_records_tuples(&transformed);
        transformed = self.transform_decorators(&transformed);

        Ok(transformed)
    }

    /// Transform ES2022 syntax to ES5-compatible code (legacy method)
    pub fn transform_es2022(&self, code: &str) -> Result<String> {
        self.transform_latest(code)
    }

    fn transform_nullish_coalescing(&self, code: &str) -> String {
        // Transform a ?? b to (a !== null && a !== undefined) ? a : b
        regex::Regex::new(r"(\w+)\s*\?\?\s*([^;]+)")
            .unwrap()
            .replace_all(code, |caps: &regex::Captures| {
                format!("(({}) !== null && ({}) !== undefined) ? ({}) : ({})",
                       &caps[1], &caps[1], &caps[1], &caps[2])
            })
            .to_string()
    }

    fn transform_optional_chaining(&self, code: &str) -> String {
        // Transform a?.b to (a !== null && a !== undefined) ? a.b : undefined
        regex::Regex::new(r"(\w+)\?\\.(\w+)")
            .unwrap()
            .replace_all(code, |caps: &regex::Captures| {
                format!("(({}) !== null && ({}) !== undefined) ? ({}).{} : undefined",
                       &caps[1], &caps[1], &caps[1], &caps[2])
            })
            .to_string()
    }

    fn transform_logical_assignment(&self, code: &str) -> String {
        let mut result = code.to_string();

        // Transform ||= operator
        result = regex::Regex::new(r"(\w+)\s*\|\|=\s*([^;]+)")
            .unwrap()
            .replace_all(&result, "$1 = $1 || ($2)")
            .to_string();

        // Transform &&= operator
        result = regex::Regex::new(r"(\w+)\s*&&=\s*([^;]+)")
            .unwrap()
            .replace_all(&result, "$1 = $1 && ($2)")
            .to_string();

        // Transform ??= operator
        result = regex::Regex::new(r"(\w+)\s*\?\?=\s*([^;]+)")
            .unwrap()
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{} = ({} !== null && {} !== undefined) ? {} : ({})",
                       &caps[1], &caps[1], &caps[1], &caps[1], &caps[2])
            })
            .to_string();

        result
    }

    fn transform_numeric_separators(&self, code: &str) -> String {
        // Remove numeric separators: 1_000_000 -> 1000000
        regex::Regex::new(r"\b(\d+)(_\d+)+\b")
            .unwrap()
            .replace_all(code, |caps: &regex::Captures| {
                caps[0].replace("_", "")
            })
            .to_string()
    }

    fn transform_private_fields(&self, code: &str) -> String {
        let mut result = code.to_string();

        // Transform private field declarations: #privateField = value;
        result = regex::Regex::new(r"#(\w+)\s*=\s*([^;]+);")
            .unwrap()
            .replace_all(&result, "this._private_$1 = $2;")
            .to_string();

        // Transform private field access: this.#privateField
        result = regex::Regex::new(r"this\.#(\w+)")
            .unwrap()
            .replace_all(&result, "this._private_$1")
            .to_string();

        result
    }

    fn transform_class_fields(&self, code: &str) -> String {
        // Transform class field declarations
        regex::Regex::new(r"class\s+(\w+)\s*\{[^}]*(\w+)\s*=\s*([^;]+);")
            .unwrap()
            .replace_all(code, |caps: &regex::Captures| {
                format!("class {} {{ constructor() {{ this.{} = {}; }} }}",
                       &caps[1], &caps[2], &caps[3])
            })
            .to_string()
    }

    fn transform_top_level_await(&self, code: &str) -> String {
        // Wrap top-level await in async IIFE
        if regex::Regex::new(r"^\s*await\s+").unwrap().is_match(code) {
            format!("(async function() {{ {} }})()", code)
        } else {
            code.to_string()
        }
    }

    fn transform_bigint_literals(&self, code: &str) -> String {
        // Transform BigInt literals: 123n -> BigInt('123')
        regex::Regex::new(r"\b(\d+)n\b")
            .unwrap()
            .replace_all(code, "BigInt('$1')")
            .to_string()
    }

    fn transform_array_at(&self, code: &str) -> String {
        // Transform arr.at(index) to arr[index >= 0 ? index : arr.length + index]
        regex::Regex::new(r"(\w+)\.at\(([^)]+)\)")
            .unwrap()
            .replace_all(code, |caps: &regex::Captures| {
                format!("(function(arr, idx) {{ var i = Math.floor(idx) || 0; return i >= 0 ? arr[i] : arr[arr.length + i]; }})({}, {})",
                       &caps[1], &caps[2])
            })
            .to_string()
    }

    /// Transform modern syntax patterns
    pub fn transform_modern_syntax(&self, code: &str) -> Result<String> {
        let mut transformed = code.to_string();

        // Transform arrow functions to regular functions
        transformed = self.transform_arrow_functions(&transformed);

        // Transform template literals
        transformed = self.transform_template_literals(&transformed);

        // Transform destructuring assignment
        transformed = self.transform_destructuring(&transformed);

        // Transform spread operator
        transformed = self.transform_spread_operator(&transformed);

        // Transform for...of loops
        transformed = self.transform_for_of_loops(&transformed);

        // Transform const/let to var
        transformed = regex::Regex::new(r"\b(const|let)\b")
            .unwrap()
            .replace_all(&transformed, "var")
            .to_string();

        Ok(transformed)
    }

    fn transform_arrow_functions(&self, code: &str) -> String {
        let mut result = code.to_string();

        // Transform arrow functions with blocks: (params) => { body }
        result = regex::Regex::new(r"(\w+)\s*=\s*\(([^)]*)\)\s*=>\s*\{([^}]*)\}")
            .unwrap()
            .replace_all(&result, "var $1 = function($2) {$3}")
            .to_string();

        // Transform arrow functions with expressions: (params) => expression
        result = regex::Regex::new(r"(\w+)\s*=\s*\(([^)]*)\)\s*=>\s*([^;{]+)")
            .unwrap()
            .replace_all(&result, "var $1 = function($2) { return $3; }")
            .to_string();

        result
    }

    fn transform_template_literals(&self, code: &str) -> String {
        regex::Regex::new(r"`([^`]*)`")
            .unwrap()
            .replace_all(code, |caps: &regex::Captures| {
                let content = &caps[1];
                let with_vars = regex::Regex::new(r"\$\{([^}]+)\}")
                    .unwrap()
                    .replace_all(content, "' + ($1) + '");
                format!("'{}'", with_vars.replace("' + '' + '", ""))
            })
            .to_string()
    }

    fn transform_destructuring(&self, code: &str) -> String {
        // Basic destructuring: const {a, b} = obj; -> var a = obj.a; var b = obj.b;
        regex::Regex::new(r"var\s*\{\s*(\w+)\s*,\s*(\w+)\s*\}\s*=\s*([^;]+);")
            .unwrap()
            .replace_all(code, "var $1 = ($3).$1; var $2 = ($3).$2;")
            .to_string()
    }

    fn transform_spread_operator(&self, code: &str) -> String {
        // Transform spread in function calls: func(...args) -> func.apply(null, args)
        regex::Regex::new(r"(\w+)\(\.\.\.(\w+)\)")
            .unwrap()
            .replace_all(code, "$1.apply(null, $2)")
            .to_string()
    }

    fn transform_for_of_loops(&self, code: &str) -> String {
        // Transform for...of loops to regular for loops
        regex::Regex::new(r"for\s*\(\s*var\s+(\w+)\s+of\s+([^)]+)\)")
            .unwrap()
            .replace_all(code, "for (var __i = 0; __i < ($2).length; __i++) { var $1 = ($2)[__i];")
            .to_string()
    }

    // ES2023+ transformations
    fn transform_hashbang_comments(&self, code: &str) -> String {
        // Remove hashbang comments: #!/usr/bin/env node
        regex::Regex::new(r"^#![^\r\n]*")
            .unwrap()
            .replace_all(code, "")
            .to_string()
    }

    fn transform_array_with(&self, code: &str) -> String {
        // Transform arr.with(index, value) method calls are handled by polyfill
        code.to_string()
    }

    fn transform_array_group_methods(&self, code: &str) -> String {
        // Array grouping methods are handled by polyfill
        code.to_string()
    }

    // ES2024+ transformations
    fn transform_regexp_v_flag(&self, code: &str) -> String {
        // Transform RegExp v flag: /pattern/v -> /pattern/u
        regex::Regex::new(r"/([^/]+)/([gimsuvy]*)v([gimsuvy]*)")
            .unwrap()
            .replace_all(code, "/$1/$2u$3")
            .to_string()
    }

    fn transform_using_declarations(&self, code: &str) -> String {
        // Transform using declarations: using resource = getResource();
        regex::Regex::new(r"using\s+(\w+)\s*=\s*([^;]+);")
            .unwrap()
            .replace_all(code, "var $1 = using($2);")
            .to_string()
    }

    fn transform_import_assertions(&self, code: &str) -> String {
        // Transform import assertions: import data from './data.json' with { type: 'json' }
        regex::Regex::new(r#"import\s+(.+?)\s+from\s+(["'][^"']+["'])\s+with\s+\{[^}]*\}"#)
            .unwrap()
            .replace_all(code, "var $1 = require($2)")
            .to_string()
    }

    // ES2025+ experimental transformations
    fn transform_pattern_matching(&self, code: &str) -> String {
        // Transform basic match expressions: match (value) { ... }
        regex::Regex::new(r"match\s*\(([^)]+)\)\s*\{")
            .unwrap()
            .replace_all(code, "switch ($1) {")
            .to_string()
    }

    fn transform_pipeline_operator(&self, code: &str) -> String {
        // Transform pipeline operator: value |> func -> func(value)
        regex::Regex::new(r"([^|]+)\s*\|>\s*([^|;]+)")
            .unwrap()
            .replace_all(code, "$2($1)")
            .to_string()
    }

    fn transform_partial_application(&self, code: &str) -> String {
        // Transform partial application: fn(?, b, ?) -> fn.partial(undefined, b, undefined)
        regex::Regex::new(r"(\w+)\(([^)]*\?[^)]*)\)")
            .unwrap()
            .replace_all(code, |caps: &regex::Captures| {
                let fn_name = &caps[1];
                let args = caps[2].replace("?", "undefined");
                format!("{}.partial({})", fn_name, args)
            })
            .to_string()
    }

    fn transform_records_tuples(&self, code: &str) -> String {
        // Transform record literals: #{ a: 1, b: 2 } -> Record({ a: 1, b: 2 })
        let mut result = regex::Regex::new(r"#\{([^}]+)\}")
            .unwrap()
            .replace_all(code, "Record({$1})")
            .to_string();

        // Transform tuple literals: #[1, 2, 3] -> Tuple(1, 2, 3)
        result = regex::Regex::new(r"#\[([^\]]+)\]")
            .unwrap()
            .replace_all(&result, "Tuple($1)")
            .to_string();

        result
    }

    fn transform_decorators(&self, code: &str) -> String {
        // Transform decorators: @decorator -> (decorator)
        regex::Regex::new(r"@(\w+)")
            .unwrap()
            .replace_all(code, "/*@$1*/")
            .to_string()
    }
}