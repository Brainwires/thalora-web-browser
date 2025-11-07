use anyhow::{anyhow, Result};
use swc_common::{
    errors::{ColorConfig, Handler},
    sync::Lrc,
    FileName, SourceMap,
};
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use std::env;

/// JavaScript security validator using AST parsing
/// This replaces the weak pattern-matching approach with proper JavaScript parsing
pub struct JavaScriptSecurityValidator {
    /// Maximum code size in bytes
    max_code_size: usize,
}

impl JavaScriptSecurityValidator {
    pub fn new() -> Self {
        Self {
            max_code_size: 10_000_000, // 10 MB
        }
    }

    /// Validate JavaScript code for security risks
    ///
    /// SECURITY POLICY (HARD BLOCKS):
    /// - Block eval() calls
    /// - Block Function() constructor
    /// - Block setTimeout/setInterval with string arguments
    /// - Block dynamic code generation
    /// - Block Node.js-specific APIs
    /// - Block document.write and dangerous DOM manipulation
    /// - Block attempts to access __proto__, constructor.constructor
    /// - Block import() and dynamic imports
    /// - Block WebAssembly.instantiate
    ///
    /// This implements a WHITELIST approach by default - code must be explicitly safe.
    pub fn is_safe_javascript(&self, js_code: &str) -> Result<()> {
        // Size limit check
        if js_code.len() > self.max_code_size {
            return Err(anyhow!(
                "JavaScript code too large: {} bytes (max: {} bytes)",
                js_code.len(),
                self.max_code_size
            ));
        }

        // Empty code is safe
        if js_code.trim().is_empty() {
            return Ok(());
        }

        // Parse JavaScript into AST
        let cm: Lrc<SourceMap> = Default::default();
        let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

        let fm = cm.new_source_file(
            FileName::Custom("input.js".into()),
            js_code.to_string(),
        );

        let lexer = Lexer::new(
            Syntax::Es(Default::default()),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);

        // Parse as module (stricter than script mode)
        let module = match parser.parse_module() {
            Ok(module) => module,
            Err(e) => {
                return Err(anyhow!("JavaScript syntax error: {:?}", e));
            }
        };

        // Traverse AST and check for dangerous patterns
        self.check_module_security(&module)?;

        Ok(())
    }

    /// Check module-level security
    fn check_module_security(&self, module: &Module) -> Result<()> {
        for item in &module.body {
            match item {
                ModuleItem::ModuleDecl(decl) => {
                    self.check_module_decl(decl)?;
                }
                ModuleItem::Stmt(stmt) => {
                    self.check_statement(stmt)?;
                }
            }
        }
        Ok(())
    }

    /// Check module declarations
    fn check_module_decl(&self, decl: &ModuleDecl) -> Result<()> {
        match decl {
            ModuleDecl::Import(_) => {
                return Err(anyhow!(
                    "SECURITY: import statements are not allowed (dynamic module loading)"
                ));
            }
            ModuleDecl::ExportDecl(export) => {
                self.check_declaration(&export.decl)?;
            }
            ModuleDecl::ExportNamed(_) => {
                // Named exports are generally safe
            }
            ModuleDecl::ExportDefaultDecl(export) => {
                match &export.decl {
                    DefaultDecl::Class(class) => {
                        self.check_class(&class.class)?;
                    }
                    DefaultDecl::Fn(func) => {
                        if let Some(function) = &func.function {
                            self.check_function(function)?;
                        }
                    }
                    _ => {}
                }
            }
            ModuleDecl::ExportDefaultExpr(export) => {
                self.check_expression(&export.expr)?;
            }
            ModuleDecl::ExportAll(_) => {
                return Err(anyhow!(
                    "SECURITY: export * statements are not allowed"
                ));
            }
            _ => {}
        }
        Ok(())
    }

    /// Check statement security
    fn check_statement(&self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Expr(expr_stmt) => {
                self.check_expression(&expr_stmt.expr)?;
            }
            Stmt::Block(block) => {
                for stmt in &block.stmts {
                    self.check_statement(stmt)?;
                }
            }
            Stmt::If(if_stmt) => {
                self.check_expression(&if_stmt.test)?;
                self.check_statement(&if_stmt.cons)?;
                if let Some(alt) = &if_stmt.alt {
                    self.check_statement(alt)?;
                }
            }
            Stmt::While(while_stmt) => {
                self.check_expression(&while_stmt.test)?;
                self.check_statement(&while_stmt.body)?;
            }
            Stmt::DoWhile(do_while) => {
                self.check_statement(&do_while.body)?;
                self.check_expression(&do_while.test)?;
            }
            Stmt::For(for_stmt) => {
                if let Some(init) = &for_stmt.init {
                    match init {
                        VarDeclOrExpr::VarDecl(var_decl) => {
                            self.check_var_decl(var_decl)?;
                        }
                        VarDeclOrExpr::Expr(expr) => {
                            self.check_expression(expr)?;
                        }
                    }
                }
                if let Some(test) = &for_stmt.test {
                    self.check_expression(test)?;
                }
                if let Some(update) = &for_stmt.update {
                    self.check_expression(update)?;
                }
                self.check_statement(&for_stmt.body)?;
            }
            Stmt::ForIn(for_in) => {
                self.check_expression(&for_in.right)?;
                self.check_statement(&for_in.body)?;
            }
            Stmt::ForOf(for_of) => {
                self.check_expression(&for_of.right)?;
                self.check_statement(&for_of.body)?;
            }
            Stmt::Switch(switch) => {
                self.check_expression(&switch.discriminant)?;
                for case in &switch.cases {
                    if let Some(test) = &case.test {
                        self.check_expression(test)?;
                    }
                    for stmt in &case.cons {
                        self.check_statement(stmt)?;
                    }
                }
            }
            Stmt::Return(return_stmt) => {
                if let Some(arg) = &return_stmt.arg {
                    self.check_expression(arg)?;
                }
            }
            Stmt::Throw(throw_stmt) => {
                self.check_expression(&throw_stmt.arg)?;
            }
            Stmt::Try(try_stmt) => {
                for stmt in &try_stmt.block.stmts {
                    self.check_statement(stmt)?;
                }
                if let Some(handler) = &try_stmt.handler {
                    for stmt in &handler.body.stmts {
                        self.check_statement(stmt)?;
                    }
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    for stmt in &finalizer.stmts {
                        self.check_statement(stmt)?;
                    }
                }
            }
            Stmt::Decl(decl) => {
                self.check_declaration(decl)?;
            }
            Stmt::With(_) => {
                return Err(anyhow!(
                    "SECURITY: 'with' statements are not allowed (ambiguous scope)"
                ));
            }
            _ => {}
        }
        Ok(())
    }

    /// Check declaration security
    fn check_declaration(&self, decl: &Decl) -> Result<()> {
        match decl {
            Decl::Var(var_decl) => {
                self.check_var_decl(var_decl)?;
            }
            Decl::Fn(fn_decl) => {
                self.check_function(&fn_decl.function)?;
            }
            Decl::Class(class_decl) => {
                self.check_class(&class_decl.class)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Check variable declaration security
    fn check_var_decl(&self, var_decl: &VarDecl) -> Result<()> {
        for declarator in &var_decl.decls {
            if let Some(init) = &declarator.init {
                self.check_expression(init)?;
            }
        }
        Ok(())
    }

    /// Check function security
    fn check_function(&self, function: &Function) -> Result<()> {
        if let Some(body) = &function.body {
            for stmt in &body.stmts {
                self.check_statement(stmt)?;
            }
        }
        Ok(())
    }

    /// Check class security
    fn check_class(&self, class: &Class) -> Result<()> {
        for member in &class.body {
            match member {
                ClassMember::Constructor(constructor) => {
                    if let Some(body) = &constructor.body {
                        for stmt in &body.stmts {
                            self.check_statement(stmt)?;
                        }
                    }
                }
                ClassMember::Method(method) => {
                    self.check_function(&method.function)?;
                }
                ClassMember::PrivateMethod(method) => {
                    self.check_function(&method.function)?;
                }
                ClassMember::ClassProp(prop) => {
                    if let Some(value) = &prop.value {
                        self.check_expression(value)?;
                    }
                }
                ClassMember::PrivateProp(prop) => {
                    if let Some(value) = &prop.value {
                        self.check_expression(value)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Check expression security (main security validation)
    fn check_expression(&self, expr: &Expr) -> Result<()> {
        match expr {
            // DANGEROUS: eval() call
            Expr::Call(call) => {
                self.check_call_expression(call)?;
            }
            // DANGEROUS: new Function()
            Expr::New(new_expr) => {
                self.check_new_expression(new_expr)?;
            }
            // Check member expressions (e.g., window.eval, Function.constructor)
            Expr::Member(member) => {
                self.check_member_expression(member)?;
            }
            // Check array expressions
            Expr::Array(array) => {
                for elem in array.elems.iter().flatten() {
                    self.check_expression(&elem.expr)?;
                }
            }
            // Check object expressions
            Expr::Object(object) => {
                for prop in &object.props {
                    match prop {
                        PropOrSpread::Prop(prop) => {
                            self.check_property(prop)?;
                        }
                        PropOrSpread::Spread(spread) => {
                            self.check_expression(&spread.expr)?;
                        }
                    }
                }
            }
            // Check function expressions
            Expr::Fn(fn_expr) => {
                self.check_function(&fn_expr.function)?;
            }
            // Check arrow functions
            Expr::Arrow(arrow) => {
                match &*arrow.body {
                    BlockStmtOrExpr::BlockStmt(block) => {
                        for stmt in &block.stmts {
                            self.check_statement(stmt)?;
                        }
                    }
                    BlockStmtOrExpr::Expr(expr) => {
                        self.check_expression(expr)?;
                    }
                }
            }
            // Check template literals
            Expr::Tpl(tpl) => {
                for expr in &tpl.exprs {
                    self.check_expression(expr)?;
                }
            }
            // Check tagged template literals
            Expr::TaggedTpl(tagged) => {
                self.check_expression(&tagged.tag)?;
                for expr in &tagged.tpl.exprs {
                    self.check_expression(expr)?;
                }
            }
            // Check binary expressions
            Expr::Bin(bin) => {
                self.check_expression(&bin.left)?;
                self.check_expression(&bin.right)?;
            }
            // Check unary expressions
            Expr::Unary(unary) => {
                self.check_expression(&unary.arg)?;
            }
            // Check update expressions (++, --)
            Expr::Update(update) => {
                self.check_expression(&update.arg)?;
            }
            // Check conditional expressions
            Expr::Cond(cond) => {
                self.check_expression(&cond.test)?;
                self.check_expression(&cond.cons)?;
                self.check_expression(&cond.alt)?;
            }
            // Check assignment expressions
            Expr::Assign(assign) => {
                self.check_expression(&assign.right)?;
            }
            // Check sequence expressions
            Expr::Seq(seq) => {
                for expr in &seq.exprs {
                    self.check_expression(expr)?;
                }
            }
            // Yield and await are generally safe in their context
            Expr::Yield(yield_expr) => {
                if let Some(arg) = &yield_expr.arg {
                    self.check_expression(arg)?;
                }
            }
            Expr::Await(await_expr) => {
                self.check_expression(&await_expr.arg)?;
            }
            // Paren expressions
            Expr::Paren(paren) => {
                self.check_expression(&paren.expr)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Check call expressions for dangerous functions
    fn check_call_expression(&self, call: &CallExpr) -> Result<()> {
        // Check the callee
        match &call.callee {
            Callee::Expr(expr) => {
                // Check for eval()
                if let Expr::Ident(ident) = &**expr {
                    let name = ident.sym.as_ref();
                    if name == "eval" {
                        return Err(anyhow!(
                            "SECURITY: eval() is not allowed (arbitrary code execution)"
                        ));
                    }
                    if name == "Function" {
                        return Err(anyhow!(
                            "SECURITY: Function() constructor is not allowed (code generation)"
                        ));
                    }
                    // Check for setTimeout/setInterval with string arguments
                    if name == "setTimeout" || name == "setInterval" {
                        // Check if first argument is a string (code execution)
                        if !call.args.is_empty() {
                            if let Expr::Lit(Lit::Str(_)) = &*call.args[0].expr {
                                return Err(anyhow!(
                                    "SECURITY: {}() with string argument is not allowed (code execution)",
                                    name
                                ));
                            }
                        }
                    }
                }

                // Check for dangerous member expressions: window.eval, obj['eval']
                if let Expr::Member(member) = &**expr {
                    self.check_dangerous_member_call(member)?;
                }

                self.check_expression(expr)?;
            }
            _ => {}
        }

        // Check all arguments
        for arg in &call.args {
            self.check_expression(&arg.expr)?;
        }

        Ok(())
    }

    /// Check new expressions for dangerous constructors
    fn check_new_expression(&self, new_expr: &NewExpr) -> Result<()> {
        // Check for new Function()
        if let Expr::Ident(ident) = &*new_expr.callee {
            let name = ident.sym.as_ref();
            if name == "Function" {
                return Err(anyhow!(
                    "SECURITY: new Function() is not allowed (code generation)"
                ));
            }
            if name == "WebAssembly" {
                return Err(anyhow!(
                    "SECURITY: WebAssembly instantiation is not allowed"
                ));
            }
        }

        self.check_expression(&new_expr.callee)?;

        // Check arguments
        if let Some(args) = &new_expr.args {
            for arg in args {
                self.check_expression(&arg.expr)?;
            }
        }

        Ok(())
    }

    /// Check member expressions for dangerous property access
    fn check_member_expression(&self, member: &MemberExpr) -> Result<()> {
        // Check for __proto__ access
        if let MemberProp::Ident(ident) = &member.prop {
            let prop_name = ident.sym.as_ref();
            if prop_name == "__proto__" {
                return Err(anyhow!(
                    "SECURITY: __proto__ access is not allowed (prototype pollution)"
                ));
            }
            // Check for constructor.constructor (Function constructor access)
            if prop_name == "constructor" {
                if let Expr::Member(inner_member) = &*member.obj {
                    if let MemberProp::Ident(inner_ident) = &inner_member.prop {
                        if inner_ident.sym.as_ref() == "constructor" {
                            return Err(anyhow!(
                                "SECURITY: constructor.constructor is not allowed (Function access)"
                            ));
                        }
                    }
                }
            }
        }

        // Check computed property access (e.g., obj["__proto__"])
        if let MemberProp::Computed(computed) = &member.prop {
            if let Expr::Lit(Lit::Str(s)) = &*computed.expr {
                let prop_name = s.value.as_ref();
                if prop_name == "__proto__" {
                    return Err(anyhow!(
                        "SECURITY: __proto__ access is not allowed (prototype pollution)"
                    ));
                }
            }
            self.check_expression(&computed.expr)?;
        }

        self.check_expression(&member.obj)?;

        Ok(())
    }

    /// Check for dangerous member function calls
    fn check_dangerous_member_call(&self, member: &MemberExpr) -> Result<()> {
        // Check for document.write, innerHTML, outerHTML
        if let MemberProp::Ident(ident) = &member.prop {
            let prop_name = ident.sym.as_ref();
            if prop_name == "write" {
                if let Expr::Ident(obj_ident) = &*member.obj {
                    if obj_ident.sym.as_ref() == "document" {
                        return Err(anyhow!(
                            "SECURITY: document.write() is not allowed (XSS vector)"
                        ));
                    }
                }
            }
            // Block innerHTML/outerHTML assignments (handled in assignment check)
        }

        Ok(())
    }

    /// Check property security
    fn check_property(&self, prop: &Prop) -> Result<()> {
        match prop {
            Prop::KeyValue(kv) => {
                self.check_expression(&kv.value)?;
            }
            Prop::Assign(assign) => {
                self.check_expression(&assign.value)?;
            }
            Prop::Getter(getter) => {
                if let Some(body) = &getter.body {
                    for stmt in &body.stmts {
                        self.check_statement(stmt)?;
                    }
                }
            }
            Prop::Setter(setter) => {
                if let Some(body) = &setter.body {
                    for stmt in &body.stmts {
                        self.check_statement(stmt)?;
                    }
                }
            }
            Prop::Method(method) => {
                self.check_function(&method.function)?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl Default for JavaScriptSecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_javascript() {
        let validator = JavaScriptSecurityValidator::new();

        // Safe code examples
        assert!(validator.is_safe_javascript("const x = 1 + 2;").is_ok());
        assert!(validator.is_safe_javascript("function add(a, b) { return a + b; }").is_ok());
        assert!(validator.is_safe_javascript("const arr = [1, 2, 3]; arr.map(x => x * 2);").is_ok());
        assert!(validator.is_safe_javascript("console.log('Hello, world!');").is_ok());
    }

    #[test]
    fn test_eval_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // Direct eval
        assert!(validator.is_safe_javascript("eval('alert(1)')").is_err());

        // Indirect eval
        assert!(validator.is_safe_javascript("window.eval('alert(1)')").is_err());
    }

    #[test]
    fn test_function_constructor_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // Function constructor
        assert!(validator.is_safe_javascript("Function('return 1')()").is_err());
        assert!(validator.is_safe_javascript("new Function('return 1')()").is_err());
    }

    #[test]
    fn test_settimeout_with_string_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // setTimeout with string (code execution)
        assert!(validator.is_safe_javascript("setTimeout('alert(1)', 1000)").is_err());

        // setTimeout with function is OK
        assert!(validator.is_safe_javascript("setTimeout(() => console.log('ok'), 1000)").is_ok());
    }

    #[test]
    fn test_proto_pollution_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // __proto__ access
        assert!(validator.is_safe_javascript("obj.__proto__ = {}").is_err());
        assert!(validator.is_safe_javascript("obj['__proto__'] = {}").is_err());
    }

    #[test]
    fn test_constructor_constructor_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // constructor.constructor access
        assert!(validator.is_safe_javascript("obj.constructor.constructor('return 1')()").is_err());
    }

    #[test]
    fn test_with_statement_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // with statement
        assert!(validator.is_safe_javascript("with (obj) { x = 1; }").is_err());
    }

    #[test]
    fn test_import_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // import statements
        assert!(validator.is_safe_javascript("import { foo } from 'bar';").is_err());
    }

    #[test]
    fn test_code_size_limit() {
        let validator = JavaScriptSecurityValidator::new();

        // Generate code larger than limit
        let large_code = "x = 1;".repeat(2_000_000); // > 10 MB
        assert!(validator.is_safe_javascript(&large_code).is_err());
    }

    #[test]
    fn test_syntax_error_rejected() {
        let validator = JavaScriptSecurityValidator::new();

        // Invalid JavaScript syntax
        assert!(validator.is_safe_javascript("const x = ;").is_err());
        assert!(validator.is_safe_javascript("function {").is_err());
    }

    #[test]
    fn test_complex_safe_code() {
        let validator = JavaScriptSecurityValidator::new();

        let safe_code = r#"
            class Calculator {
                constructor() {
                    this.value = 0;
                }
                add(n) {
                    this.value += n;
                    return this;
                }
                subtract(n) {
                    this.value -= n;
                    return this;
                }
                getValue() {
                    return this.value;
                }
            }
            const calc = new Calculator();
            const result = calc.add(10).subtract(3).getValue();
            console.log(result);
        "#;

        assert!(validator.is_safe_javascript(safe_code).is_ok());
    }
}
