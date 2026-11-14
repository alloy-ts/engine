//! Tree-walking evaluator: ScriptEvaluation, declaration instantiation,
//! statement evaluation to Completion Records, and expression evaluation
//! through Reference Records.

use super::completion::{Completion, CompletionRecord, Exception};
use super::environment::{DeclarativeEnvironment, not_defined};
use super::operations::{
    is_less_than, is_loosely_equal, is_strictly_equal, to_boolean, to_number, to_string,
};
use super::parser::redeclaration;
use super::realm::Realm;
use super::script::{
    BinaryOp, DeclarationKind, Expr, ScriptRecord, Statement, UnaryOp, VariableBinding,
    lexically_declared_names, var_declared_names,
};
use super::value::Value;

/// Scripts are evaluated as non-strict (sloppy) code: there are no
/// directives yet, so `"use strict"` is just an expression statement.
const STRICT: bool = false;

/// ScriptEvaluation (§16.1.6): GlobalDeclarationInstantiation, then the
/// statement list. A normal completion with an empty value becomes
/// undefined.
pub fn evaluate_script(realm: &mut Realm, script: &ScriptRecord) -> Completion<Value> {
    let statements = script.statements();
    if let Err(error) = global_declaration_instantiation(realm, statements) {
        let value = realm.alloc_error(error);
        return Completion::Throw(value);
    }
    let mut context = ScriptContext {
        realm,
        scopes: Vec::new(),
    };
    context.statement_list(statements).into()
}

/// GlobalDeclarationInstantiation (§16.1.7). All checks run before any
/// binding is created, so a SyntaxError leaves the Realm unchanged.
fn global_declaration_instantiation(
    realm: &mut Realm,
    script: &[Statement],
) -> Result<(), Exception> {
    let env = &mut realm.global_env;
    let var_names = var_declared_names(script);
    for (_, name) in lexically_declared_names(script) {
        if env.has_lexical_declaration(name) || env.has_restricted_global_property(name) {
            return Err(redeclaration(name));
        }
    }
    if let Some(name) = var_names
        .iter()
        .find(|name| env.has_lexical_declaration(name))
    {
        return Err(redeclaration(name));
    }
    for (kind, name) in lexically_declared_names(script) {
        if kind == DeclarationKind::Const {
            env.declarative_record.create_immutable_binding(name, true);
        } else {
            env.declarative_record.create_mutable_binding(name);
        }
    }
    for name in var_names {
        env.create_global_var_binding(name);
    }
    Ok(())
}

/// BlockDeclarationInstantiation (§14.2.3) for `let` / `const`.
fn block_declaration_instantiation(body: &[Statement]) -> DeclarativeEnvironment {
    let mut env = DeclarativeEnvironment::default();
    for (kind, name) in lexically_declared_names(body) {
        if kind == DeclarationKind::Const {
            env.create_immutable_binding(name, true);
        } else {
            env.create_mutable_binding(name);
        }
    }
    env
}

/// `[[Base]]` of a Reference Record to an identifier.
#[derive(Clone, Copy, Debug)]
enum Base {
    Unresolvable,
    /// The global Environment Record.
    Global,
    /// A block's Declarative Environment Record, by index in the scope stack.
    Declarative(usize),
}

/// A Reference Record (§6.2.5) for an identifier. `[[Strict]]` is always
/// [`STRICT`] and `[[ThisValue]]` is always empty, so neither is stored.
struct Reference<'a> {
    base: Base,
    name: &'a str,
}

/// The running execution context of one ScriptEvaluation.
///
/// `scopes` holds the block Declarative Environment Records: the last one is
/// the LexicalEnvironment, each one's `[[OuterEnv]]` is the one before it,
/// and the first one's is the Realm's global environment. Without closures
/// no environment outlives its block, so a stack is enough.
struct ScriptContext<'r> {
    realm: &'r mut Realm,
    scopes: Vec<DeclarativeEnvironment>,
}

impl ScriptContext<'_> {
    /// StatementList evaluation (§14.2.2): each item's completion is passed
    /// through UpdateEmpty with the list's value so far, so the list's value
    /// is that of the last value-producing statement.
    fn statement_list(&mut self, list: &[Statement]) -> CompletionRecord {
        let mut completion = CompletionRecord::EMPTY;
        for statement in list {
            let value = match completion {
                CompletionRecord::Normal(value) => value,
                CompletionRecord::Break { .. }
                | CompletionRecord::Continue { .. }
                | CompletionRecord::Throw(_) => {
                    unreachable!("abrupt completions return early")
                }
            };
            completion = self.statement(statement).update_empty(value);
            if completion.is_abrupt() {
                break;
            }
        }
        completion
    }

    fn statement(&mut self, statement: &Statement) -> CompletionRecord {
        match statement {
            Statement::Empty => CompletionRecord::EMPTY,
            // ExpressionStatement (§14.5.1).
            Statement::Expression(expr) => match self.expression(expr) {
                Ok(value) => CompletionRecord::Normal(Some(value)),
                Err(error) => CompletionRecord::Throw(self.realm.alloc_error(error)),
            },
            // VariableStatement (§14.3.2.1) and LexicalDeclaration
            // (§14.3.1.2) complete with empty.
            Statement::Declaration(kind, bindings) => match self.declaration(*kind, bindings) {
                Ok(()) => CompletionRecord::EMPTY,
                Err(error) => CompletionRecord::Throw(self.realm.alloc_error(error)),
            },
            // Block (§14.2.2).
            Statement::Block(body) if body.is_empty() => CompletionRecord::EMPTY,
            Statement::Block(body) => self.block(body),
            // IfStatement (§14.6.2), including UpdateEmpty for the selected
            // branch as required by the evaluation algorithm.
            Statement::If {
                condition,
                consequent,
                alternate,
            } => match self.expression(condition) {
                Err(error) => CompletionRecord::Throw(self.realm.alloc_error(error)),
                Ok(value) if to_boolean(self.realm.heap(), value) => self
                    .statement(consequent)
                    .update_empty(Some(Value::Undefined)),
                Ok(_) => alternate.as_deref().map_or_else(
                    || CompletionRecord::Normal(Some(Value::Undefined)),
                    |statement| {
                        self.statement(statement)
                            .update_empty(Some(Value::Undefined))
                    },
                ),
            },
            Statement::While { condition, body } => self.while_statement(condition, body, &[]),
            Statement::DoWhile { body, condition } => self.do_while_statement(body, condition, &[]),
            Statement::Labelled { label, body } => self.labelled_statement(label, body),
            Statement::Break(target) => CompletionRecord::Break {
                value: None,
                target: target.clone(),
            },
            Statement::Continue(target) => CompletionRecord::Continue {
                value: None,
                target: target.clone(),
            },
            Statement::Throw(expression) => match self.expression(expression) {
                Ok(value) => CompletionRecord::Throw(value),
                Err(error) => CompletionRecord::Throw(self.realm.alloc_error(error)),
            },
            Statement::Try {
                body,
                catch,
                finally,
            } => {
                let mut completion = self.block(body);
                if let CompletionRecord::Throw(thrown) = &completion
                    && let Some(catch) = catch
                {
                    let mut catch_environment = DeclarativeEnvironment::default();
                    if let Some(binding) = &catch.binding {
                        catch_environment.create_mutable_binding(binding);
                        catch_environment.initialize_binding(binding, *thrown);
                    }
                    self.scopes.push(catch_environment);
                    completion = self.block(&catch.body);
                    self.scopes.pop();
                }
                if let Some(finally) = finally {
                    let final_completion = self.block(finally);
                    if final_completion.is_abrupt() {
                        return final_completion;
                    }
                }
                completion
            }
        }
    }

    fn block(&mut self, body: &[Statement]) -> CompletionRecord {
        self.scopes.push(block_declaration_instantiation(body));
        let completion = self.statement_list(body);
        self.scopes.pop();
        completion
    }

    fn labelled_statement(&mut self, label: &str, body: &Statement) -> CompletionRecord {
        let mut labels = vec![label];
        let mut item = body;
        while let Statement::Labelled { label, body } = item {
            labels.push(label);
            item = body;
        }
        let result = match item {
            Statement::While { condition, body } => self.while_statement(condition, body, &labels),
            Statement::DoWhile { body, condition } => {
                self.do_while_statement(body, condition, &labels)
            }
            statement => self.statement(statement),
        };
        match result {
            CompletionRecord::Break {
                value,
                target: Some(target),
            } if labels.iter().any(|label| **label == *target) => CompletionRecord::Normal(value),
            other => other,
        }
    }

    fn while_statement(
        &mut self,
        condition: &Expr,
        body: &Statement,
        labels: &[&str],
    ) -> CompletionRecord {
        let mut value = None;
        loop {
            let condition = match self.expression(condition) {
                Ok(value) => value,
                Err(error) => {
                    return CompletionRecord::Throw(self.realm.alloc_error(error));
                }
            };
            if !to_boolean(self.realm.heap(), condition) {
                return CompletionRecord::Normal(value);
            }
            match self.statement(body).update_empty(value) {
                CompletionRecord::Normal(next) => value = next,
                CompletionRecord::Continue {
                    value: next,
                    target,
                } if target
                    .as_deref()
                    .is_none_or(|target| labels.contains(&target)) =>
                {
                    value = next;
                }
                CompletionRecord::Break { value, target }
                    if target
                        .as_deref()
                        .is_none_or(|target| labels.contains(&target)) =>
                {
                    return CompletionRecord::Normal(value);
                }
                abrupt => return abrupt,
            }
        }
    }

    fn do_while_statement(
        &mut self,
        body: &Statement,
        condition: &Expr,
        labels: &[&str],
    ) -> CompletionRecord {
        let mut value = None;
        loop {
            match self.statement(body).update_empty(value) {
                CompletionRecord::Normal(next) => value = next,
                CompletionRecord::Continue {
                    value: next,
                    target,
                } if target
                    .as_deref()
                    .is_none_or(|target| labels.contains(&target)) =>
                {
                    value = next;
                }
                CompletionRecord::Break { value, target }
                    if target
                        .as_deref()
                        .is_none_or(|target| labels.contains(&target)) =>
                {
                    return CompletionRecord::Normal(value);
                }
                abrupt => return abrupt,
            }

            let condition = match self.expression(condition) {
                Ok(value) => value,
                Err(error) => return CompletionRecord::Throw(self.realm.alloc_error(error)),
            };
            if !to_boolean(self.realm.heap(), condition) {
                return CompletionRecord::Normal(value);
            }
        }
    }

    fn declaration(
        &mut self,
        kind: DeclarationKind,
        bindings: &[VariableBinding],
    ) -> Result<(), Exception> {
        for VariableBinding { name, initializer } in bindings {
            let reference = self.resolve_binding(name);
            match (kind, initializer) {
                // `var x;` does nothing at runtime.
                (DeclarationKind::Var, None) => {}
                (DeclarationKind::Var, Some(initializer)) => {
                    let value = self.expression(initializer)?;
                    self.put_value(&reference, value)?;
                }
                // `let x;` initializes to undefined.
                (_, initializer) => {
                    let value = match initializer {
                        Some(initializer) => self.expression(initializer)?,
                        None => Value::Undefined,
                    };
                    self.initialize_referenced_binding(&reference, value);
                }
            }
        }
        Ok(())
    }

    /// ResolveBinding (§9.4.2) / GetIdentifierReference (§9.1.2.1) from the
    /// running LexicalEnvironment outwards.
    fn resolve_binding<'a>(&self, name: &'a str) -> Reference<'a> {
        let base = match self.scopes.iter().rposition(|env| env.has_binding(name)) {
            Some(index) => Base::Declarative(index),
            None if self.realm.global_env.has_binding(name) => Base::Global,
            None => Base::Unresolvable,
        };
        Reference { base, name }
    }

    /// GetValue (§6.2.5.5).
    fn get_value(&self, reference: &Reference) -> Result<Value, Exception> {
        match reference.base {
            Base::Unresolvable => Err(not_defined(reference.name)),
            Base::Global => self
                .realm
                .global_env
                .get_binding_value(reference.name, STRICT),
            Base::Declarative(index) => self.scopes[index].get_binding_value(reference.name),
        }
    }

    /// PutValue (§6.2.5.6). Sloppy code assigning an unresolvable reference
    /// creates a global object property.
    fn put_value(&mut self, reference: &Reference, value: Value) -> Result<(), Exception> {
        let name = reference.name;
        match reference.base {
            Base::Unresolvable if STRICT => Err(not_defined(name)),
            Base::Unresolvable => self
                .realm
                .global_env
                .set_global_object_property(name, value, false),
            Base::Global => self
                .realm
                .global_env
                .set_mutable_binding(name, value, STRICT),
            Base::Declarative(index) => self.scopes[index].set_mutable_binding(name, value, STRICT),
        }
    }

    /// InitializeReferencedBinding (§6.2.5.8) for a `let` / `const` binding,
    /// which declaration instantiation created in the current environment.
    fn initialize_referenced_binding(&mut self, reference: &Reference, value: Value) {
        match reference.base {
            Base::Global => self
                .realm
                .global_env
                .declarative_record
                .initialize_binding(reference.name, value),
            Base::Declarative(index) => {
                self.scopes[index].initialize_binding(reference.name, value)
            }
            Base::Unresolvable => unreachable!("lexical bindings are instantiated before use"),
        }
    }

    fn expression(&mut self, expr: &Expr) -> Result<Value, Exception> {
        Ok(match expr {
            Expr::Number(n) => Value::Number(*n),
            Expr::String(s) => self.realm.alloc_string(s.clone()),
            Expr::Boolean(b) => Value::Boolean(*b),
            Expr::Null => Value::Null,
            Expr::Identifier(name) => self.get_value(&self.resolve_binding(name))?,
            Expr::Unary(op, operand) => {
                if *op == UnaryOp::Typeof
                    && let Expr::Identifier(name) = &**operand
                    && matches!(self.resolve_binding(name).base, Base::Unresolvable)
                {
                    return Ok(self.realm.alloc_string("undefined"));
                }
                let value = self.expression(operand)?;
                let heap = self.realm.heap();
                match op {
                    UnaryOp::Plus => Value::Number(to_number(heap, value)),
                    UnaryOp::Minus => Value::Number(-to_number(heap, value)),
                    UnaryOp::Not => Value::Boolean(!to_boolean(heap, value)),
                    UnaryOp::Typeof => {
                        let name = match value {
                            Value::Undefined => "undefined",
                            Value::Null => "object",
                            Value::Boolean(_) => "boolean",
                            Value::Number(_) => "number",
                            Value::String(_) => "string",
                            Value::Error(_) => "object",
                        };
                        self.realm.alloc_string(name)
                    }
                }
            }
            Expr::Binary(op, left, right) => {
                let left = self.expression(left)?;
                match op {
                    BinaryOp::LogicalAnd if !to_boolean(self.realm.heap(), left) => left,
                    BinaryOp::LogicalOr if to_boolean(self.realm.heap(), left) => left,
                    BinaryOp::LogicalAnd | BinaryOp::LogicalOr => self.expression(right)?,
                    _ => {
                        let right = self.expression(right)?;
                        binary(self.realm, *op, left, right)
                    }
                }
            }
            // AssignmentExpression (§13.15.2): the target reference is
            // resolved before the right-hand side is evaluated.
            Expr::Assign(op, name, value) => {
                let reference = self.resolve_binding(name);
                let value = match op {
                    None => self.expression(value)?,
                    Some(op) => {
                        let left = self.get_value(&reference)?;
                        let right = self.expression(value)?;
                        // ApplyStringOrNumericBinaryOperator.
                        binary(self.realm, *op, left, right)
                    }
                };
                self.put_value(&reference, value)?;
                value
            }
        })
    }
}

fn binary(realm: &mut Realm, op: BinaryOp, x: Value, y: Value) -> Value {
    let heap = realm.heap();
    let number = |f: fn(f64, f64) -> f64| Value::Number(f(to_number(heap, x), to_number(heap, y)));
    match op {
        BinaryOp::LogicalOr | BinaryOp::LogicalAnd => {
            unreachable!("logical operators are evaluated with short-circuiting")
        }
        BinaryOp::Add => {
            if matches!(x, Value::String(_)) || matches!(y, Value::String(_)) {
                let joined = format!("{}{}", to_string(heap, x), to_string(heap, y));
                return realm.alloc_string(joined);
            }
            number(|a, b| a + b)
        }
        BinaryOp::Sub => number(|a, b| a - b),
        BinaryOp::Mul => number(|a, b| a * b),
        BinaryOp::Div => number(|a, b| a / b),
        // Rust's `%` on f64 is the truncating remainder Number::remainder
        // specifies (the sign follows the dividend).
        BinaryOp::Rem => number(|a, b| a % b),
        BinaryOp::Lt => Value::Boolean(is_less_than(heap, x, y) == Some(true)),
        BinaryOp::Gt => Value::Boolean(is_less_than(heap, y, x) == Some(true)),
        BinaryOp::LtEq => Value::Boolean(is_less_than(heap, y, x) == Some(false)),
        BinaryOp::GtEq => Value::Boolean(is_less_than(heap, x, y) == Some(false)),
        BinaryOp::LooseEq => Value::Boolean(is_loosely_equal(heap, x, y)),
        BinaryOp::LooseNe => Value::Boolean(!is_loosely_equal(heap, x, y)),
        BinaryOp::StrictEq => Value::Boolean(is_strictly_equal(heap, x, y)),
        BinaryOp::StrictNe => Value::Boolean(!is_strictly_equal(heap, x, y)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Evaluates `source` in `realm` and renders the result as JavaScript
    /// would display it, prefixing the type so `"1"` and `1` differ.
    fn run_in(realm: &mut Realm, source: &str) -> String {
        match realm.evaluate(source) {
            Completion::Normal(Value::String(s)) => format!("string:{}", realm.heap().string(s)),
            Completion::Normal(Value::Number(n)) if n == 0.0 && n.is_sign_negative() => {
                "number:-0".to_owned()
            }
            Completion::Normal(value) => {
                let kind = match value {
                    Value::Undefined => "undefined",
                    Value::Null => "null",
                    Value::Boolean(_) => "boolean",
                    Value::Number(_) => "number",
                    Value::String(_) => unreachable!(),
                    Value::Error(_) => "object",
                };
                format!("{kind}:{}", to_string(realm.heap(), value))
            }
            Completion::Throw(Value::Error(handle)) => {
                let error = realm.heap().error(handle);
                format!("{}: {}", error.kind.name(), error.message)
            }
            Completion::Throw(value) => {
                format!("throw:{}", to_string(realm.heap(), value))
            }
        }
    }

    fn run(source: &str) -> String {
        run_in(&mut Realm::new(), source)
    }

    fn check(cases: &[(&str, &str)]) {
        for (source, expected) in cases {
            assert_eq!(run(source), *expected, "{source}");
        }
    }

    #[test]
    fn empty_and_multi_statement_scripts() {
        check(&[
            ("", "undefined:undefined"),
            ("  \n // comment\n", "undefined:undefined"),
            (";;;", "undefined:undefined"),
            ("1; 2; 3", "number:3"),
            ("1\n2\n'three'", "string:three"),
            ("1;;", "number:1"),
            ("1\n+2", "number:3"),
        ]);
    }

    #[test]
    fn precedence_and_unary() {
        check(&[
            ("1 + 2 * 3", "number:7"),
            ("(1 + 2) * 3", "number:9"),
            ("10 - 4 - 3", "number:3"),
            ("2 * 3 % 4", "number:2"),
            ("12 / 2 / 3", "number:2"),
            ("-2 * -3", "number:6"),
            ("- -1", "number:1"),
            ("+'42'", "number:42"),
            ("+''", "number:0"),
            ("+' 0x10 '", "number:16"),
            ("+'abc'", "number:NaN"),
            ("+true", "number:1"),
            ("+null", "number:0"),
            ("+undefined", "number:NaN"),
            ("-null", "number:-0"),
            ("!0", "boolean:true"),
            ("!''", "boolean:true"),
            ("!'0'", "boolean:false"),
            ("!NaN", "boolean:true"),
            ("!!-0", "boolean:false"),
            ("!null", "boolean:true"),
            ("!!undefined", "boolean:false"),
            ("1 + 2 < 4 === true", "boolean:true"),
            ("!1 == 0", "boolean:true"),
        ]);
    }

    #[test]
    fn typeof_operator() {
        check(&[
            ("typeof undefined", "string:undefined"),
            ("typeof missing", "string:undefined"),
            ("typeof null", "string:object"),
            ("typeof false", "string:boolean"),
            ("typeof 1", "string:number"),
            ("typeof 'x'", "string:string"),
            ("typeof typeof 1", "string:string"),
            ("typeof 1 + '!'", "string:number!"),
        ]);
        assert_eq!(
            run("{ typeof x; let x = 1; }"),
            "ReferenceError: Cannot access 'x' before initialization"
        );
    }

    #[test]
    fn logical_operators_short_circuit_and_return_values() {
        check(&[
            ("0 || 2", "number:2"),
            ("1 || 2", "number:1"),
            ("0 && 2", "number:0"),
            ("1 && 2", "number:2"),
            ("'' || 'fallback'", "string:fallback"),
            ("'left' && 'right'", "string:right"),
            ("false || true && false", "boolean:false"),
            ("false && missing", "boolean:false"),
            ("true || missing", "boolean:true"),
            ("let x = 0; false && (x = 1); x", "number:0"),
            ("let x = 0; true || (x = 1); x", "number:0"),
            ("let x = 0; true && (x = 1); x", "number:1"),
        ]);
    }

    #[test]
    fn if_else_selects_one_branch_and_propagates_completion() {
        check(&[
            ("if (true) 1; else 2", "number:1"),
            ("if (false) 1; else 2", "number:2"),
            ("if (0) 1; else 2", "number:2"),
            ("if ('x') 1", "number:1"),
            ("if (false) 1", "undefined:undefined"),
            ("1; if (false);", "undefined:undefined"),
            ("if (true) { 1; 2; } else 3", "number:2"),
            ("if (false) 1; else if (true) 2; else 3", "number:2"),
            ("let x = 0; if (true) x = 1; else x = 2; x", "number:1"),
            ("let x = 0; if (false) x = 1; else x = 2; x", "number:2"),
            ("if (false) missing; else 1", "number:1"),
        ]);

        // `var` declarations in either branch are instantiated before the
        // condition runs, while only the selected initializer executes.
        check(&[
            ("x; if (false) var x = 1; x", "undefined:undefined"),
            ("if (true) var x = 1; else var y = 2; x", "number:1"),
            (
                "if (true) var x = 1; else var y = 2; y",
                "undefined:undefined",
            ),
        ]);
    }

    #[test]
    fn while_break_and_continue_follow_completion_semantics() {
        check(&[
            ("while (false) 1", "undefined:undefined"),
            ("let x = 0; while (x < 3) x += 1", "number:3"),
            ("while (true) { 7; break; }", "number:7"),
            (
                "let x = 0; let sum = 0; while (x < 5) { x += 1; if (x === 2) continue; sum += x; if (x === 4) break; } sum",
                "number:8",
            ),
            (
                "let outer = 0; let hits = 0; while (outer < 2) { outer += 1; let inner = 0; while (inner < 3) { inner += 1; if (inner === 2) break; hits += 1; } } hits",
                "number:2",
            ),
            ("x; while (false) var x = 1; x", "undefined:undefined"),
        ]);
    }

    #[test]
    fn do_while_runs_once_and_tests_condition_after_the_body() {
        check(&[
            ("do 1; while (false)", "number:1"),
            ("1; do {} while (false)", "number:1"),
            ("let x = 0; do x += 1; while (false); x", "number:1"),
            ("let x = 0; do x += 1; while (x < 3); x", "number:3"),
            (
                "let x = 0; let checks = 0; do { x += 1; if (x === 1) continue; } while ((checks += 1) < 3); x",
                "number:3",
            ),
            ("do { 7; break; } while (true)", "number:7"),
        ]);
    }

    #[test]
    fn do_while_labels_and_finally_preserve_completion_semantics() {
        check(&[
            (
                "let x = 0; outer: do { x += 1; if (x < 3) continue outer; break outer; } while (true); x",
                "number:3",
            ),
            (
                "let x = 0; do { try { x += 1; continue; } finally { x += 1; } } while (x < 3); x",
                "number:4",
            ),
            (
                "let x = 0; outer: do { try { break outer; } finally { x = 3; } } while (true); x",
                "number:3",
            ),
            (
                "let x = 0; do { try { continue; } finally { x += 1; } } while (x < 2); x",
                "number:2",
            ),
        ]);
    }

    #[test]
    fn labelled_control_flow_uses_completion_targets() {
        check(&[
            ("done: { 7; break done; 9; }", "number:7"),
            (
                "let x = 0; outer: while (x < 4) { x += 1; let y = 0; while (y < 3) { y += 1; if (y === 2) continue outer; } x = 99; } x",
                "number:4",
            ),
            (
                "let outer = 0; let hits = 0; top: while (outer < 3) { outer += 1; while (true) { hits += 1; break top; } } hits",
                "number:1",
            ),
            (
                "let x = 0; first: second: while (x < 3) { x += 1; continue first; } x",
                "number:3",
            ),
        ]);
    }

    #[test]
    fn throw_try_catch_finally_follow_completion_semantics() {
        check(&[
            ("try { throw 1; } catch (e) { e; }", "number:1"),
            ("try { missing; } catch (e) { typeof e; }", "string:object"),
            ("try { throw 1; } catch { 2; }", "number:2"),
            ("try { 1; } finally { 2; }", "number:1"),
            (
                "try { throw 1; } catch (e) { e + 1; } finally { 9; }",
                "number:2",
            ),
            ("try { throw 1; } finally { throw 2; }", "throw:2"),
            (
                "let x = 0; while (true) { try { break; } finally { x = 3; } } x",
                "number:3",
            ),
            (
                "let x = 0; outer: while (x < 2) { try { x += 1; continue outer; } finally { x += 1; } } x",
                "number:2",
            ),
            ("x; try { var x = 1; } finally { var y = 2; } y", "number:2"),
        ]);
        assert_eq!(
            run("try { throw 1; } catch (e) {} e"),
            "ReferenceError: e is not defined"
        );
    }

    #[test]
    fn string_concatenation_and_coercion() {
        check(&[
            ("'a' + 'b'", "string:ab"),
            ("1 + '2'", "string:12"),
            ("'1' + 2", "string:12"),
            ("1 + 2 + '3'", "string:33"),
            ("'1' + 2 + 3", "string:123"),
            ("'' + null + undefined + true", "string:nullundefinedtrue"),
            ("'' + 0.1 * 3", "string:0.30000000000000004"),
            ("'' + -0", "string:0"),
            ("'' + 1e21", "string:1e+21"),
            ("'' + 1 / 3", "string:0.3333333333333333"),
            ("'x' + -Infinity", "string:x-Infinity"),
            ("'' + NaN", "string:NaN"),
            ("'6' * '7'", "number:42"),
            ("'10' - 1", "number:9"),
            ("'a' - 1", "number:NaN"),
            ("true + 1", "number:2"),
            ("null + 1", "number:1"),
            ("undefined + 1", "number:NaN"),
            ("'é' + '😀'", "string:é😀"),
        ]);
    }

    #[test]
    fn number_edge_cases() {
        check(&[
            ("1 / 0", "number:Infinity"),
            ("-1 / 0", "number:-Infinity"),
            ("1 / -0", "number:-Infinity"),
            ("0 / 0", "number:NaN"),
            ("Infinity - Infinity", "number:NaN"),
            ("Infinity * 0", "number:NaN"),
            ("-0", "number:-0"),
            ("0 * -1", "number:-0"),
            ("-0 + 0", "number:0"),
            ("-0 - 0", "number:-0"),
            ("5 % 3", "number:2"),
            ("-5 % 3", "number:-2"),
            ("5 % -3", "number:2"),
            ("-4 % 2", "number:-0"),
            ("5 % 0", "number:NaN"),
            ("5 % Infinity", "number:5"),
            ("Infinity % 2", "number:NaN"),
            ("5.5 % 2", "number:1.5"),
            ("0.1 + 0.2", "number:0.30000000000000004"),
            ("1e308 * 10", "number:Infinity"),
            ("NaN", "number:NaN"),
            ("Infinity", "number:Infinity"),
            ("undefined", "undefined:undefined"),
            ("null", "null:null"),
        ]);
    }

    #[test]
    fn comparisons() {
        check(&[
            ("1 < 2", "boolean:true"),
            ("2 <= 2", "boolean:true"),
            ("3 > 2", "boolean:true"),
            ("2 >= 3", "boolean:false"),
            ("NaN < 1", "boolean:false"),
            ("NaN >= 1", "boolean:false"),
            ("NaN <= NaN", "boolean:false"),
            ("-0 < 0", "boolean:false"),
            ("-0 <= 0", "boolean:true"),
            ("-Infinity < Infinity", "boolean:true"),
            ("'a' < 'b'", "boolean:true"),
            ("'B' < 'a'", "boolean:true"),
            ("'10' < '9'", "boolean:true"),
            ("'10' < 9", "boolean:false"),
            ("'abc' < 'abcd'", "boolean:true"),
            ("'' < 'a'", "boolean:true"),
            ("'a' <= 'a'", "boolean:true"),
            ("'a' >= 'b'", "boolean:false"),
            // U+FF61 sorts after U+1F600 by UTF-16 code units (0xFF61 > 0xD83D)
            // although it is the smaller code point.
            ("'\\uFF61' > '😀'", "boolean:true"),
            ("null < 1", "boolean:true"),
            ("null >= 0", "boolean:true"),
            ("undefined < 1", "boolean:false"),
            ("undefined >= 0", "boolean:false"),
            ("true > false", "boolean:true"),
            ("'x' < 1", "boolean:false"),
            ("'x' >= 1", "boolean:false"),
        ]);
    }

    #[test]
    fn strict_and_loose_equality() {
        check(&[
            ("1 === 1", "boolean:true"),
            ("1 === '1'", "boolean:false"),
            ("1 == '1'", "boolean:true"),
            ("'a' === 'a'", "boolean:true"),
            ("'a' == 'b'", "boolean:false"),
            ("0 == ''", "boolean:true"),
            ("'' == '0'", "boolean:false"),
            ("0 == '0'", "boolean:true"),
            ("' \\n' == 0", "boolean:true"),
            ("'0x10' == 16", "boolean:true"),
            ("null == undefined", "boolean:true"),
            ("null === undefined", "boolean:false"),
            ("null == 0", "boolean:false"),
            ("undefined == 0", "boolean:false"),
            ("null == false", "boolean:false"),
            ("undefined == ''", "boolean:false"),
            ("null == null", "boolean:true"),
            ("undefined === undefined", "boolean:true"),
            ("true == 1", "boolean:true"),
            ("true === 1", "boolean:false"),
            ("false == '0'", "boolean:true"),
            ("false == ''", "boolean:true"),
            ("true == '1'", "boolean:true"),
            ("true == 'true'", "boolean:false"),
            ("true == true", "boolean:true"),
            ("NaN == NaN", "boolean:false"),
            ("NaN === NaN", "boolean:false"),
            ("NaN != NaN", "boolean:true"),
            ("NaN !== NaN", "boolean:true"),
            ("0 === -0", "boolean:true"),
            ("0 == -0", "boolean:true"),
            ("1 !== 1", "boolean:false"),
            ("1 != '2'", "boolean:true"),
            ("'NaN' == NaN", "boolean:false"),
        ]);
    }

    #[test]
    fn errors_are_throw_completions() {
        check(&[
            ("missing", "ReferenceError: missing is not defined"),
            ("1; missing; 2", "ReferenceError: missing is not defined"),
            ("a + b", "ReferenceError: a is not defined"),
            ("globalThis", "ReferenceError: globalThis is not defined"),
            ("1 +", "SyntaxError: Unexpected end of input"),
            ("1 + 2)", "SyntaxError: Unexpected token ')'"),
            ("(1 + 2", "SyntaxError: Unexpected end of input"),
            ("'abc", "SyntaxError: Unterminated string literal"),
            ("1; 2 +", "SyntaxError: Unexpected end of input"),
            ("x => 1", "SyntaxError: Unexpected token '=>'"),
        ]);
    }

    #[test]
    fn syntax_errors_are_reported_before_evaluation() {
        let mut realm = Realm::new();
        // A later syntax error means no statement runs (early errors).
        assert_eq!(
            run_in(&mut realm, "missing; 1 +"),
            "SyntaxError: Unexpected end of input"
        );
    }

    #[test]
    fn identifiers_resolve_from_realm_globals() {
        let mut realm = Realm::new();
        realm.set_global("x", Value::Number(20.0)).unwrap();
        let name = realm.alloc_string("Ada");
        realm.set_global("name", name).unwrap();
        assert_eq!(run_in(&mut realm, "x * 2 + 2"), "number:42");
        assert_eq!(run_in(&mut realm, "'hi ' + name"), "string:hi Ada");
        assert_eq!(run("x"), "ReferenceError: x is not defined");
    }

    /// Runs each script in order in one Realm.
    fn check_in(cases: &[(&str, &str)]) -> Realm {
        let mut realm = Realm::new();
        for (source, expected) in cases {
            assert_eq!(run_in(&mut realm, source), *expected, "{source}");
        }
        realm
    }

    const TDZ_X: &str = "ReferenceError: Cannot access 'x' before initialization";
    const CONST: &str = "TypeError: Assignment to constant variable.";

    #[test]
    fn statement_list_values_follow_update_empty() {
        check(&[
            ("1; let x = 2;", "number:1"),
            ("1; {}", "number:1"),
            ("1; var a;", "number:1"),
            ("1; var a = 2", "number:1"),
            ("1;;;;;", "number:1"),
            ("1; {} {}", "number:1"),
            ("1; { let y = 5; }", "number:1"),
            ("1; { 2; let q; }", "number:2"),
            ("1; { 2 }", "number:2"),
            ("{ 1 } 2", "number:2"),
            ("1; { { } var b; }", "number:1"),
            ("let z = 1", "undefined:undefined"),
            ("const c = 1; var v; {}", "undefined:undefined"),
            ("var c = 3; c", "number:3"),
            ("x = 5", "number:5"),
        ]);
    }

    #[test]
    fn lexical_bindings_have_a_temporal_dead_zone() {
        check(&[
            ("x; let x = 1", TDZ_X),
            ("x = 2; let x", TDZ_X),
            ("x += 1; let x = 1", TDZ_X),
            ("let x = x", TDZ_X),
            ("const x = x + 1", TDZ_X),
            ("{ x; let x; }", TDZ_X),
            ("let x = 1; { x; let x = 2; }", TDZ_X),
            ("let x; x", "undefined:undefined"),
        ]);
        // A global binding whose initializer threw stays uninitialized.
        let realm = check_in(&[
            ("let x = missing", "ReferenceError: missing is not defined"),
            ("x", TDZ_X),
            ("x = 1", TDZ_X),
            (
                "let x = 2",
                "SyntaxError: Identifier 'x' has already been declared",
            ),
        ]);
        assert!(realm.get_global("x").is_none());
    }

    #[test]
    fn const_bindings_reject_assignment() {
        check(&[
            ("const c = 1; c = 2", CONST),
            ("const c = 1; c += 1", CONST),
            ("{ const d = 1; d = 2 }", CONST),
            ("const c = 1; { c = 2 }", CONST),
            ("const c = 1; c", "number:1"),
            ("const a = 1, b = a + 1; b", "number:2"),
            (
                "const c",
                "SyntaxError: Missing initializer in const declaration",
            ),
            (
                "const a = 1, b",
                "SyntaxError: Missing initializer in const declaration",
            ),
        ]);
        // The failed assignment leaves the value unchanged.
        check_in(&[
            ("const k = 1", "undefined:undefined"),
            ("k = 2", CONST),
            ("k", "number:1"),
        ]);
    }

    #[test]
    fn redeclarations_are_early_errors_within_a_script() {
        let redeclared =
            |name: &str| format!("SyntaxError: Identifier '{name}' has already been declared");
        for (source, name) in [
            ("let a; let a", "a"),
            ("let a, a", "a"),
            ("const a = 1; let a", "a"),
            ("let a; var a", "a"),
            ("var a; let a", "a"),
            ("{ var a; } let a", "a"),
            ("let a; { var a; }", "a"),
            ("{ let b; const b = 1 }", "b"),
            ("{ let b; { var b; } }", "b"),
            ("{ var b; let b; }", "b"),
        ] {
            assert_eq!(run(source), redeclared(name), "{source}");
        }
        // Early errors run before anything is evaluated or declared.
        check_in(&[
            ("x = 1; let y; let y", &redeclared("y")),
            ("x", "ReferenceError: x is not defined"),
        ]);
        check(&[
            ("var a = 1; var a = 2; a", "number:2"),
            ("var a = 1, a = 3; a", "number:3"),
            ("let a = 1; { let a = 2; } a", "number:1"),
            ("{ let a = 1; } { let a = 2; a }", "number:2"),
            ("{ var v = 1; } { var v = 2; } v", "number:2"),
            (
                "let let = 1",
                "SyntaxError: let is disallowed as a lexically bound name",
            ),
            (
                "const let = 1",
                "SyntaxError: let is disallowed as a lexically bound name",
            ),
        ]);
    }

    #[test]
    fn redeclarations_across_scripts_follow_global_declaration_instantiation() {
        let redeclared =
            |name: &str| format!("SyntaxError: Identifier '{name}' has already been declared");
        let realm = check_in(&[
            ("let a = 1; const b = 2; var c = 3", "undefined:undefined"),
            ("let a = 2", &redeclared("a")),
            ("var a", &redeclared("a")),
            ("{ var a; }", &redeclared("a")),
            ("const b = 3", &redeclared("b")),
            ("let c", &redeclared("c")),
            ("const c = 1", &redeclared("c")),
            ("var c = 4; c", "number:4"),
            ("a + b", "number:3"),
            // A failed GlobalDeclarationInstantiation declares nothing.
            ("let fresh = 1; let a = 2", &redeclared("a")),
            ("fresh", "ReferenceError: fresh is not defined"),
            // Non-configurable built-ins are restricted global properties.
            ("let undefined", &redeclared("undefined")),
            ("const NaN = 1", &redeclared("NaN")),
            ("var undefined; undefined", "undefined:undefined"),
            // Implicit globals are configurable, so `let` may shadow them.
            ("implicit = 1", "number:1"),
            ("let implicit = 2; implicit", "number:2"),
            ("implicit = 3; implicit", "number:3"),
            // Blocks may shadow global lexical bindings.
            ("{ let a = 10; a }", "number:10"),
            ("a", "number:1"),
        ]);
        let env = &realm.global_env;
        // let/const live in the declarative record, var on the global object.
        assert!(env.has_lexical_declaration("a") && env.has_lexical_declaration("b"));
        assert!(!env.has_lexical_declaration("c") && env.has_restricted_global_property("c"));
        assert!(!env.has_restricted_global_property("a"));
    }

    #[test]
    fn blocks_scope_lexical_declarations() {
        check(&[
            ("let x = 1; { let x = 2; x }", "number:2"),
            ("let x = 1; { let x = 2; } x", "number:1"),
            ("let x = 1; { x = 3; } x", "number:3"),
            ("{ let x = 1; { let x = 2; } x }", "number:1"),
            ("{ const k = 1; { const k = 2; k } }", "number:2"),
            (
                "{ let inner = 1; } inner",
                "ReferenceError: inner is not defined",
            ),
            ("{ var hoisted = 1; } hoisted", "number:1"),
            ("hoisted; { var hoisted = 1; }", "undefined:undefined"),
            ("{ let a = 1; { a += 1; { a *= 10; } } a }", "number:20"),
            ("{", "SyntaxError: Unexpected end of input"),
            ("}", "SyntaxError: Unexpected token '}'"),
            ("{ 1 2 }", "SyntaxError: Unexpected number"),
        ]);
        // The LexicalEnvironment is restored when a block throws.
        check_in(&[
            (
                "{ let s = 1; missing }",
                "ReferenceError: missing is not defined",
            ),
            ("s", "ReferenceError: s is not defined"),
        ]);
    }

    #[test]
    fn assignment_and_compound_assignment() {
        check(&[
            ("var s = '1'; s += 2; s", "string:12"),
            ("var n = '6'; n *= '7'", "number:42"),
            ("var u; u += 1", "number:NaN"),
            ("var b = true; b -= 1", "number:0"),
            ("var z = null; z += 'a'", "string:nulla"),
            ("let x = 10; x %= 4", "number:2"),
            ("let x = 1; x /= 0", "number:Infinity"),
            ("let x = -1; x *= 0", "number:-0"),
            ("let x = 'a'; x -= 1", "number:NaN"),
            // The left value is read before the right side runs.
            ("let a = 1; a += a += 1", "number:3"),
            ("let a = 1; a += (a = 10); a", "number:11"),
            ("a = b = 3; a + b", "number:6"),
            ("(a) = 1; a", "number:1"),
            ("let x\nx = 1\nx", "number:1"),
            ("missing += 1", "ReferenceError: missing is not defined"),
            ("1 = 2", "SyntaxError: Invalid left-hand side in assignment"),
            (
                "a + 1 = 2",
                "SyntaxError: Invalid left-hand side in assignment",
            ),
            (
                "x = 1 = 2",
                "SyntaxError: Invalid left-hand side in assignment",
            ),
            (
                "(a = 1) = 2",
                "SyntaxError: Invalid left-hand side in assignment",
            ),
            (
                "-a = 1",
                "SyntaxError: Invalid left-hand side in assignment",
            ),
            // `let` is only a keyword before a binding identifier.
            ("var let = 1; let", "number:1"),
            ("let = 2; let", "number:2"),
        ]);
    }

    #[test]
    fn sloppy_assignment_creates_globals_and_ignores_read_only_ones() {
        let mut realm = check_in(&[
            ("created = 1", "number:1"),
            ("created + 1", "number:2"),
            ("undefined = 1", "number:1"),
            ("undefined", "undefined:undefined"),
            ("Infinity += 1; Infinity", "number:Infinity"),
            ("var NaN = 1; NaN", "number:NaN"),
        ]);
        assert!(matches!(
            realm.get_global("created"),
            Some(Value::Number(1.0))
        ));
        // Implicit globals are ordinary configurable properties.
        assert!(!realm.global_env.has_restricted_global_property("created"));
        realm.set_global("created", Value::Number(5.0)).unwrap();
        assert_eq!(run_in(&mut realm, "created"), "number:5");
    }

    #[test]
    fn declaration_and_assignment_nesting_is_bounded() {
        let too_deep = format!(
            "RangeError: Expression nesting exceeds the limit of {}",
            super::super::parser::MAX_DEPTH
        );
        check(&[
            (&"{".repeat(100_000), &too_deep),
            (&format!("{}1", "a=".repeat(100_000)), &too_deep),
        ]);
        let depth = super::super::parser::MAX_DEPTH - 1;
        let blocks = format!("{}1{}", "{".repeat(depth), "}".repeat(depth));
        assert_eq!(run(&blocks), "number:1");
        assert_eq!(run(&format!("{}1", "a=".repeat(depth))), "number:1");
    }
}
