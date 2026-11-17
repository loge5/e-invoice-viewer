//! A minimal Schematron runner for the official rule artefacts
//! (`queryBinding="xslt2"`), as decided in ADR 0002.
//!
//! - Rule contexts (XSLT match patterns) become XPath expressions: every
//!   top-level union branch that does not start with `/` gets a `//` prefix.
//! - Within a pattern, a node is checked only by the first rule whose context
//!   matches it.
//! - An assert is violated if its test is false. A test that raises a dynamic
//!   error counts as violated too.
//! - The engine evaluates both operands of `and` and `or`, where Saxon (used
//!   by the official validator) stops after the first one that decides. So
//!   if a test raises an error, it is evaluated again with its top-level
//!   `and`/`or` chains rewritten to `if`s, which keeps errors in skipped
//!   operands from firing the rule. Without an error both forms give the same
//!   result; the rewritten form is compiled only when needed, because the
//!   engine compiles `if`s slowly. Function bodies are always rewritten.
//! - The engine raises a type error for a predicate that selects more than
//!   one node (`a[b]` with two `b`). Predicates that are plain location paths
//!   are rewritten to `a[boolean(b)]`, which is the same in XPath.
//! - The engine fails on `normalize-space()` and `string-length()` without
//!   argument; they are rewritten to the equivalent `f(string(.))`.
//! - Tests that start with `/` and use no rule variable do not depend on the
//!   context node and are evaluated once per document.
//! - Variables (`let`) become XPath `let` bindings in front of each
//!   expression that uses them: global ones (schema or pattern level) are
//!   evaluated on the document node, rule ones on the context node.
//! - `xsl:function`s become inline functions bound to a variable; calls are
//!   rewritten to dynamic calls.
//! - `include` is resolved from the files passed to [`RuleSet::compile`].
//! - `<name/>` and `<value-of select="…"/>` in messages are filled in.

use std::cell::OnceCell;
use std::collections::{HashMap, HashSet};
use std::panic::{AssertUnwindSafe, catch_unwind};

use xee_xpath::context::StaticContextBuilder;
use xee_xpath::query::SequenceQuery;
use xee_xpath::{DocumentHandle, Documents, Item, Queries, Query};

use super::{Finding, Severity, business_terms};

const SCHEMATRON_NS: &str = "http://purl.oclc.org/dsdl/schematron";
const XSL_NS: &str = "http://www.w3.org/1999/XSL/Transform";

/// A compiled rule set.
pub(super) struct RuleSet {
    /// Namespace prefixes declared by the rule set, used for locations.
    prefixes: Vec<(String, String)>,
    patterns: Vec<Vec<Rule>>,
    /// `name(.)`, for `<name/>` in messages.
    name_query: SequenceQuery,
    /// IDs of asserts that could not be compiled (none for the official
    /// artefacts; see the unit tests). They are skipped.
    #[cfg_attr(not(test), allow(dead_code, reason = "checked by unit tests"))]
    uncompiled: Vec<String>,
}

struct Rule {
    context: Option<SequenceQuery>,
    asserts: Vec<Assert>,
}

struct Assert {
    id: String,
    severity: Severity,
    message: Vec<MessagePart>,
    test: SequenceQuery,
    /// The test with short-circuiting `and`/`or`, if different; compiled on
    /// first use, for when `test` raises an error.
    short_circuited: Option<(String, OnceCell<Option<SequenceQuery>>)>,
    /// The test does not depend on the context node.
    absolute: bool,
}

enum MessagePart {
    Text(String),
    /// `<name/>`: the name of the context node.
    Name,
    /// `<value-of select="…"/>`, compiled to a string expression.
    ValueOf(SequenceQuery),
}

/// A variable or function binding that expressions may refer to.
struct Binding {
    /// The XPath binding, e.g. `$x := root(.) ! (…)`.
    text: String,
    /// Variables the binding refers to.
    references: Vec<String>,
}

/// The variables and functions of a rule set, and of the rule being
/// compiled.
struct Scope<'a> {
    globals: &'a HashMap<String, Binding>,
    rule_lets: Vec<(String, Binding)>,
}

impl Scope<'_> {
    /// Prefixes `expression` with the bindings it needs, in dependency order.
    /// Returns the expression and whether it uses a rule variable.
    fn wrap(&self, expression: &str) -> (String, bool) {
        let mut ordered: Vec<&str> = Vec::new();
        let mut visited: HashSet<&str> = HashSet::new();
        let mut uses_rule_let = false;
        for name in variable_references(expression) {
            self.visit(&name, &mut ordered, &mut visited, &mut uses_rule_let);
        }
        if ordered.is_empty() {
            return (expression.to_string(), uses_rule_let);
        }
        (
            format!("let {} return ({expression})", ordered.join(", ")),
            uses_rule_let,
        )
    }

    fn visit<'s>(
        &'s self,
        name: &str,
        ordered: &mut Vec<&'s str>,
        visited: &mut HashSet<&'s str>,
        uses_rule_let: &mut bool,
    ) {
        let rule_let = self
            .rule_lets
            .iter()
            .rev()
            .find(|(rule_name, _)| rule_name == name);
        let (key, binding) = match rule_let {
            Some((key, binding)) => {
                *uses_rule_let = true;
                (key.as_str(), binding)
            }
            None => match self.globals.get_key_value(name) {
                Some((key, binding)) => (key.as_str(), binding),
                None => return,
            },
        };
        if !visited.insert(key) {
            return;
        }
        for reference in &binding.references {
            self.visit(reference, ordered, visited, uses_rule_let);
        }
        ordered.push(&binding.text);
    }
}

impl RuleSet {
    /// Compiles a Schematron file; `includes` maps the `href` of each
    /// `include` to the included file. Panics only if a bundled file is not
    /// well-formed XML, which the unit tests rule out.
    pub(super) fn compile(schematron: &str, includes: &[(&str, &str)]) -> Self {
        let doc = roxmltree::Document::parse(schematron).expect("bundled rules are well-formed");
        let included: Vec<(&str, roxmltree::Document)> = includes
            .iter()
            .map(|(href, text)| {
                (
                    *href,
                    roxmltree::Document::parse(text).expect("bundled rules are well-formed"),
                )
            })
            .collect();
        let root = doc.root_element();

        let prefixes: Vec<(String, String)> = children(root, SCHEMATRON_NS, "ns")
            .filter_map(|ns| {
                Some((
                    ns.attribute("prefix")?.to_string(),
                    ns.attribute("uri")?.to_string(),
                ))
            })
            .collect();
        let mut builder = StaticContextBuilder::default();
        for (prefix, uri) in &prefixes {
            builder.add_namespace(prefix, uri);
        }
        let queries = Queries::new(builder);

        // Patterns in document order, with includes resolved.
        let patterns: Vec<roxmltree::Node> = root
            .children()
            .filter(roxmltree::Node::is_element)
            .filter_map(|node| match schematron_name(node) {
                Some("pattern") => Some(node),
                Some("include") => {
                    let href = node.attribute("href")?;
                    let (_, document) = included.iter().find(|(name, _)| *name == href)?;
                    Some(document.root_element())
                }
                _ => None,
            })
            .filter(|node| schematron_name(*node) == Some("pattern"))
            .collect();

        let functions: Vec<(String, String)> = children(root, XSL_NS, "function")
            .filter_map(|function| {
                let name = function.attribute("name")?;
                Some((name.to_string(), function_variable(name)))
            })
            .collect();
        let rewrite = |expression: &str| {
            rewrite_predicates(&rewrite_context_functions(&rewrite_calls(
                expression, &functions,
            )))
        };

        // Global variables: schema and pattern level, evaluated on the
        // document node; functions as inline functions.
        let mut globals: HashMap<String, Binding> = HashMap::new();
        let global_lets = children(root, SCHEMATRON_NS, "let").chain(
            patterns
                .iter()
                .flat_map(|pattern| children(*pattern, SCHEMATRON_NS, "let")),
        );
        for node in global_lets {
            let (Some(name), Some(value)) = (node.attribute("name"), node.attribute("value"))
            else {
                continue;
            };
            let value = rewrite(value);
            globals.insert(
                name.to_string(),
                Binding {
                    text: format!("${name} := root(.) ! ({value})"),
                    references: variable_references(&value),
                },
            );
        }
        for function in children(root, XSL_NS, "function") {
            let Some(name) = function.attribute("name") else {
                continue;
            };
            let text = inline_function(function, &rewrite);
            let variable = function_variable(name);
            globals.insert(
                variable.clone(),
                Binding {
                    text: format!("${variable} := {text}"),
                    references: variable_references(&text),
                },
            );
        }

        let name_query = compile(&queries, "name(.)").expect("name(.) compiles");
        let mut uncompiled = Vec::new();
        let patterns = patterns
            .iter()
            .map(|pattern| {
                children(*pattern, SCHEMATRON_NS, "rule")
                    .map(|rule| compile_rule(rule, &queries, &globals, &rewrite, &mut uncompiled))
                    .collect()
            })
            .collect();
        Self {
            prefixes,
            patterns,
            name_query,
            uncompiled,
        }
    }

    #[cfg(test)]
    pub(super) fn uncompiled(&self) -> Vec<&str> {
        self.uncompiled.iter().map(String::as_str).collect()
    }

    #[cfg(test)]
    pub(super) fn assert_count(&self) -> usize {
        self.patterns
            .iter()
            .flatten()
            .map(|rule| rule.asserts.len())
            .sum()
    }

    /// Evaluates all rules against a loaded invoice and returns the
    /// violations.
    pub(super) fn evaluate(
        &self,
        documents: &mut Documents,
        document: DocumentHandle,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();
        let mut absolute_results: HashMap<*const Assert, bool> = HashMap::new();
        for rules in &self.patterns {
            let mut checked: HashSet<xot::Node> = HashSet::new();
            for rule in rules {
                let Some(context) = &rule.context else {
                    continue;
                };
                let Some(nodes) = guarded(|| context.execute(documents, document).ok())
                    .flatten()
                    .map(|sequence| sequence.iter().collect::<Vec<Item>>())
                else {
                    continue;
                };
                for item in nodes {
                    let Ok(node) = item.to_node() else { continue };
                    if !checked.insert(node) {
                        continue;
                    }
                    for assert in &rule.asserts {
                        let key: *const Assert = assert;
                        let holds = match absolute_results.get(&key) {
                            Some(holds) => *holds,
                            None => {
                                let holds = self.holds(assert, documents, &item);
                                if assert.absolute {
                                    absolute_results.insert(key, holds);
                                }
                                holds
                            }
                        };
                        if !holds {
                            let message = self.message(&assert.message, documents, &item);
                            findings.push(Finding {
                                rule_id: assert.id.clone(),
                                severity: assert.severity,
                                business_terms: business_terms(&message),
                                message,
                                location: location(documents.xot(), node, &self.prefixes),
                            });
                        }
                    }
                }
            }
        }
        findings
    }

    /// Whether an assert holds for a context item. A test that raises an
    /// error is evaluated again short-circuited; if that fails too, or the
    /// engine panics, the assert does not hold.
    fn holds(&self, assert: &Assert, documents: &mut Documents, item: &Item) -> bool {
        if let Some(holds) = test_result(&assert.test, documents, item) {
            return holds;
        }
        let Some((expression, query)) = &assert.short_circuited else {
            return false;
        };
        let query = query.get_or_init(|| {
            let mut builder = StaticContextBuilder::default();
            for (prefix, uri) in &self.prefixes {
                builder.add_namespace(prefix, uri);
            }
            compile(&Queries::new(builder), expression)
        });
        query
            .as_ref()
            .and_then(|query| test_result(query, documents, item))
            .unwrap_or(false)
    }

    /// The message of a violated assert, placeholders filled in for the
    /// context item, whitespace normalised.
    fn message(&self, parts: &[MessagePart], documents: &mut Documents, item: &Item) -> String {
        let text: String = parts
            .iter()
            .map(|part| match part {
                MessagePart::Text(text) => text.clone(),
                MessagePart::Name => string_value(&self.name_query, documents, item),
                MessagePart::ValueOf(query) => string_value(query, documents, item),
            })
            .collect();
        normalize_space(&text)
    }
}

fn compile_rule(
    rule: roxmltree::Node,
    queries: &Queries,
    globals: &HashMap<String, Binding>,
    rewrite: &dyn Fn(&str) -> String,
    uncompiled: &mut Vec<String>,
) -> Rule {
    let mut scope = Scope {
        globals,
        rule_lets: Vec::new(),
    };
    let context = rule.attribute("context").and_then(|context| {
        compile(
            queries,
            &scope.wrap(&rewrite(&context_expression(context))).0,
        )
    });

    let mut asserts = Vec::new();
    for node in rule.children().filter(roxmltree::Node::is_element) {
        match schematron_name(node) {
            Some("let") => {
                if let (Some(name), Some(value)) = (node.attribute("name"), node.attribute("value"))
                {
                    let value = rewrite(value);
                    scope.rule_lets.push((
                        name.to_string(),
                        Binding {
                            text: format!("${name} := ({value})"),
                            references: variable_references(&value),
                        },
                    ));
                }
            }
            Some("assert") => {
                let id = node.attribute("id").unwrap_or_default().to_string();
                let source = rewrite(node.attribute("test").unwrap_or_default());
                let (test, uses_rule_let) = scope.wrap(&source);
                let short_circuited = short_circuit(&source);
                let short_circuited = (short_circuited != source.trim())
                    .then(|| (scope.wrap(&short_circuited).0, OnceCell::new()));
                let message = message_parts(node, queries, &scope, rewrite);
                match (compile(queries, &test), message) {
                    (Some(test), Some(message)) if context.is_some() => asserts.push(Assert {
                        severity: severity(node.attribute("flag")),
                        absolute: source.trim_start().starts_with('/') && !uses_rule_let,
                        message,
                        id,
                        test,
                        short_circuited,
                    }),
                    _ => uncompiled.push(id),
                }
            }
            _ => {}
        }
    }
    Rule { context, asserts }
}

/// The parts of an assert message; `None` if a `value-of` does not compile.
fn message_parts(
    assert: roxmltree::Node,
    queries: &Queries,
    scope: &Scope,
    rewrite: &dyn Fn(&str) -> String,
) -> Option<Vec<MessagePart>> {
    assert
        .children()
        .map(|node| {
            if node.is_text() {
                return Some(MessagePart::Text(
                    node.text().unwrap_or_default().to_string(),
                ));
            }
            match schematron_name(node) {
                Some("name") => Some(match node.attribute("path") {
                    Some(path) => MessagePart::ValueOf(compile(
                        queries,
                        &scope.wrap(&rewrite(&format!("name({path})"))).0,
                    )?),
                    None => MessagePart::Name,
                }),
                Some("value-of") => {
                    let select = rewrite(node.attribute("select").unwrap_or_default());
                    let expression = format!("string-join(({select}) ! string(.), ' ')");
                    Some(MessagePart::ValueOf(compile(
                        queries,
                        &scope.wrap(&expression).0,
                    )?))
                }
                _ => Some(MessagePart::Text(text_content(node))),
            }
        })
        .collect()
}

/// An `xsl:function` as an XPath inline function expression.
fn inline_function(function: roxmltree::Node, rewrite: &dyn Fn(&str) -> String) -> String {
    let params: Vec<String> = children(function, XSL_NS, "param")
        .filter_map(|param| {
            let name = param.attribute("name")?;
            Some(match param.attribute("as") {
                Some(kind) => format!("${name} as {kind}"),
                None => format!("${name}"),
            })
        })
        .collect();
    let variables: Vec<String> = children(function, XSL_NS, "variable")
        .filter_map(|variable| {
            Some(format!(
                "${} := ({})",
                variable.attribute("name")?,
                rewrite(variable.attribute("select")?)
            ))
        })
        .collect();
    let body = children(function, XSL_NS, "sequence")
        .find_map(|sequence| sequence.attribute("select"))
        .map(|select| short_circuit(&rewrite(select)))
        .unwrap_or_else(|| "()".to_string());
    let body = if variables.is_empty() {
        body
    } else {
        format!("let {} return ({body})", variables.join(", "))
    };
    let returns = function
        .attribute("as")
        .map(|kind| format!(" as {kind}"))
        .unwrap_or_default();
    format!("function({}){returns} {{ {body} }}", params.join(", "))
}

/// The variable a function is bound to, e.g. `fn_xr_checkIBAN`.
fn function_variable(name: &str) -> String {
    format!("fn_{}", name.replace(':', "_"))
}

/// Rewrites calls of the rule set's functions to dynamic calls of their
/// variables, outside string literals.
fn rewrite_calls(expression: &str, functions: &[(String, String)]) -> String {
    if functions.is_empty() {
        return expression.to_string();
    }
    let mut out = String::with_capacity(expression.len());
    let mut quote = None;
    let mut index = 0;
    while index < expression.len() {
        let rest = &expression[index..];
        let c = rest.chars().next().expect("not at end");
        match quote {
            Some(open) if c == open => quote = None,
            Some(_) => {}
            None if c == '\'' || c == '"' => quote = Some(c),
            None => {
                let preceded_by_name = expression[..index]
                    .chars()
                    .next_back()
                    .is_some_and(|p| is_name_char(p) || p == '$');
                let call = functions.iter().find(|(name, _)| {
                    rest.starts_with(name.as_str())
                        && rest[name.len()..].trim_start().starts_with('(')
                });
                if let (false, Some((name, variable))) = (preceded_by_name, call) {
                    out.push('$');
                    out.push_str(variable);
                    index += name.len();
                    continue;
                }
            }
        }
        out.push(c);
        index += c.len_utf8();
    }
    out
}

/// Names of the variables an expression refers to (outside string
/// literals), in order of appearance.
fn variable_references(expression: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut quote = None;
    let mut chars = expression.char_indices().peekable();
    while let Some((index, c)) = chars.next() {
        match quote {
            Some(open) if c == open => quote = None,
            Some(_) => {}
            None if c == '\'' || c == '"' => quote = Some(c),
            None if c == '$' => {
                let start = index + 1;
                let mut end = start;
                while let Some(&(i, n)) = chars.peek() {
                    if is_name_char(n) || n == ':' {
                        end = i + n.len_utf8();
                        chars.next();
                    } else {
                        break;
                    }
                }
                let name = &expression[start..end];
                if !name.is_empty() && !names.iter().any(|known| known == name) {
                    names.push(name.to_string());
                }
            }
            None => {}
        }
    }
    names
}

fn is_name_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '-' | '_' | '.')
}

/// Splits an expression at top-level occurrences of the keyword `word`
/// (outside literals and brackets). Returns `None` if one of `blockers`
/// occurs at top level.
fn split_top_level<'e>(expression: &'e str, word: &str, blockers: &[&str]) -> Option<Vec<&'e str>> {
    let mut parts = Vec::new();
    let (mut depth, mut quote, mut start) = (0i32, None, 0);
    let mut chars = expression.char_indices().peekable();
    while let Some((index, c)) = chars.next() {
        match (quote, c) {
            (Some(open), _) if c == open => quote = None,
            (Some(_), _) => {}
            (None, '\'' | '"') => quote = Some(c),
            (None, '(' | '[' | '{') => depth += 1,
            (None, ')' | ']' | '}') => depth -= 1,
            (None, _) if depth == 0 && c.is_alphabetic() => {
                let preceded = expression[..index]
                    .chars()
                    .next_back()
                    .is_some_and(|p| is_name_char(p) || p == '$' || p == ':');
                let mut end = index;
                while let Some(&(i, n)) = chars.peek() {
                    if is_name_char(n) {
                        end = i;
                        chars.next();
                    } else {
                        break;
                    }
                }
                let token = &expression[index..=end];
                let followed_by_colon = expression[end + 1..].starts_with(':');
                if preceded || followed_by_colon {
                    continue;
                }
                if blockers.contains(&token) {
                    return None;
                }
                if token == word {
                    parts.push(&expression[start..index]);
                    start = end + 1;
                }
            }
            _ => {}
        }
    }
    parts.push(&expression[start..]);
    Some(parts)
}

/// Longest `and`/`or` chain that is rewritten. Longer chains in the
/// official rules are lists of value comparisons (code lists) that cannot
/// raise errors, and nesting them deeply is slow to compile.
const MAX_CHAIN: usize = 4;

/// Keywords that end an `and`/`or` chain at top level.
const CHAIN_BLOCKERS: [&str; 9] = [
    "if",
    "then",
    "else",
    "return",
    "satisfies",
    "some",
    "every",
    "for",
    "let",
];

/// Rewrites top-level `a or b` and `a and b` chains to nested `if`s, so that
/// later operands are evaluated only if the earlier ones do not decide, as
/// in Saxon. Operands that are fully parenthesised are rewritten too.
/// Chains longer than [`MAX_CHAIN`] are left as they are.
fn short_circuit(expression: &str) -> String {
    let trimmed = expression.trim();
    if let Some(inner) = parenthesised(trimmed) {
        return format!("({})", short_circuit(inner));
    }
    if let Some(parts) = split_top_level(trimmed, "or", &CHAIN_BLOCKERS)
        && parts.len() > 1
    {
        return if parts.len() <= MAX_CHAIN {
            chain(&parts, true)
        } else {
            trimmed.to_string()
        };
    }
    let mut blockers = CHAIN_BLOCKERS.to_vec();
    blockers.push("or");
    match split_top_level(trimmed, "and", &blockers) {
        Some(parts) if parts.len() > 1 && parts.len() <= MAX_CHAIN => chain(&parts, false),
        _ => trimmed.to_string(),
    }
}

/// `a or b or c` as `if (a) then true() else …`, `a and b and c` as
/// `if (a) then (…) else false()`.
fn chain(parts: &[&str], or: bool) -> String {
    match parts {
        [last] => format!("boolean({})", short_circuit(last)),
        [first, rest @ ..] if or => format!(
            "if ({}) then true() else ({})",
            short_circuit(first),
            chain(rest, or)
        ),
        [first, rest @ ..] => format!(
            "if ({}) then ({}) else false()",
            short_circuit(first),
            chain(rest, or)
        ),
        [] => "true()".to_string(),
    }
}

/// The inside of `( … )` if the parentheses enclose the whole expression.
fn parenthesised(expression: &str) -> Option<&str> {
    let inner = expression.strip_prefix('(')?.strip_suffix(')')?;
    let (mut depth, mut quote) = (0i32, None);
    for c in inner.chars() {
        match (quote, c) {
            (Some(open), _) if c == open => quote = None,
            (Some(_), _) => {}
            (None, '\'' | '"') => quote = Some(c),
            (None, '(' | '[' | '{') => depth += 1,
            (None, ')' | ']' | '}') => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
            }
            _ => {}
        }
    }
    (depth == 0).then_some(inner)
}

/// Functions the engine fails on when called without argument.
const CONTEXT_FUNCTIONS: [&str; 2] = ["normalize-space", "string-length"];

/// Rewrites `normalize-space()` and `string-length()` to
/// `f(string(.))`, outside string literals.
fn rewrite_context_functions(expression: &str) -> String {
    let mut out = String::with_capacity(expression.len());
    let mut quote = None;
    let mut index = 0;
    while index < expression.len() {
        let rest = &expression[index..];
        let c = rest.chars().next().expect("not at end");
        match quote {
            Some(open) if c == open => quote = None,
            Some(_) => {}
            None if c == '\'' || c == '"' => quote = Some(c),
            None => {
                let preceded_by_name = expression[..index]
                    .chars()
                    .next_back()
                    .is_some_and(|p| is_name_char(p) || p == ':' || p == '$');
                let call = CONTEXT_FUNCTIONS.iter().find_map(|name| {
                    let after = rest.strip_prefix(name)?.trim_start().strip_prefix('(')?;
                    let after = after.trim_start().strip_prefix(')')?;
                    Some((name, rest.len() - after.len()))
                });
                if let (false, Some((name, length))) = (preceded_by_name, call) {
                    out.push_str(name);
                    out.push_str("(string(.))");
                    index += length;
                    continue;
                }
            }
        }
        out.push(c);
        index += c.len_utf8();
    }
    out
}

/// Rewrites predicates that are plain location paths, `[b/c]`, to
/// `[boolean(b/c)]` (see the module documentation).
fn rewrite_predicates(expression: &str) -> String {
    let mut out = String::with_capacity(expression.len());
    let mut quote = None;
    let mut index = 0;
    while index < expression.len() {
        let c = expression[index..].chars().next().expect("not at end");
        match quote {
            Some(open) if c == open => quote = None,
            Some(_) => {}
            None if c == '\'' || c == '"' => quote = Some(c),
            None if c == '[' => {
                if let Some(end) = closing_bracket(expression, index) {
                    let inner = rewrite_predicates(&expression[index + 1..end]);
                    if is_location_path(&inner) {
                        out.push_str(&format!("[boolean({inner})]"));
                    } else {
                        out.push('[');
                        out.push_str(&inner);
                        out.push(']');
                    }
                    index = end + 1;
                    continue;
                }
            }
            None => {}
        }
        out.push(c);
        index += c.len_utf8();
    }
    out
}

/// Index of the `]` that closes the `[` at `open`.
fn closing_bracket(expression: &str, open: usize) -> Option<usize> {
    let (mut depth, mut quote) = (0i32, None);
    for (index, c) in expression[open..].char_indices() {
        match (quote, c) {
            (Some(q), _) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '\'' | '"') => quote = Some(c),
            (None, '[') => depth += 1,
            (None, ']') => {
                depth -= 1;
                if depth == 0 {
                    return Some(open + index);
                }
            }
            _ => {}
        }
    }
    None
}

/// Whether an expression is a location path of name tests only, e.g.
/// `cac:TaxSubtotal`, `./@schemeID`, `ancestor::cac:Party/cbc:ID[1]`: its
/// value is always a node sequence, never a number.
fn is_location_path(expression: &str) -> bool {
    let expression = expression.trim();
    let mut chars = expression.chars();
    match (chars.next(), chars.next()) {
        (None, _) => return false,
        (Some(c), _) if c.is_ascii_digit() => return false,
        (Some('.'), Some(c)) if c.is_ascii_digit() => return false,
        _ => {}
    }
    let mut depth = 0i32;
    let mut quote = None;
    for c in expression.chars() {
        match (quote, c) {
            (Some(q), _) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '\'' | '"') if depth > 0 => quote = Some(c),
            (None, '[') => depth += 1,
            (None, ']') => depth -= 1,
            (None, _) if depth > 0 => {}
            (None, c) if is_name_char(c) || matches!(c, ':' | '*' | '@' | '/') => {}
            _ => return false,
        }
    }
    depth == 0
}

/// The boolean result of a test for a context item; `None` on errors and
/// engine panics.
fn test_result(test: &SequenceQuery, documents: &mut Documents, item: &Item) -> Option<bool> {
    guarded(|| {
        test.execute(documents, item)
            .ok()
            .and_then(|sequence| sequence.effective_boolean_value().ok())
    })
    .flatten()
}

/// The string value of a query result for a context item; empty on errors.
fn string_value(query: &SequenceQuery, documents: &mut Documents, item: &Item) -> String {
    guarded(|| {
        let sequence = query.execute(documents, item).ok()?;
        let first = sequence.iter().next()?;
        first.try_into_value::<String>().ok()
    })
    .flatten()
    .unwrap_or_default()
}

/// Runs engine code, turning a panic into `None`.
fn guarded<T>(f: impl FnOnce() -> T) -> Option<T> {
    catch_unwind(AssertUnwindSafe(f)).ok()
}

fn compile(queries: &Queries, expression: &str) -> Option<SequenceQuery> {
    guarded(|| queries.sequence(expression).ok()).flatten()
}

fn schematron_name<'a>(node: roxmltree::Node<'a, '_>) -> Option<&'a str> {
    (node.is_element() && node.tag_name().namespace() == Some(SCHEMATRON_NS))
        .then(|| node.tag_name().name())
}

fn children<'a, 'input>(
    node: roxmltree::Node<'a, 'input>,
    namespace: &'static str,
    name: &'static str,
) -> impl Iterator<Item = roxmltree::Node<'a, 'input>> {
    node.children().filter(move |child| {
        child.is_element()
            && child.tag_name().namespace() == Some(namespace)
            && child.tag_name().name() == name
    })
}

fn severity(flag: Option<&str>) -> Severity {
    match flag {
        Some("warning") => Severity::Warning,
        Some("information") => Severity::Information,
        _ => Severity::Error,
    }
}

fn text_content(node: roxmltree::Node) -> String {
    node.descendants()
        .filter(roxmltree::Node::is_text)
        .filter_map(|text| text.text())
        .collect()
}

fn normalize_space(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Turns an XSLT match pattern into an XPath expression selecting all nodes
/// it matches.
fn context_expression(pattern: &str) -> String {
    let mut branches = Vec::new();
    let (mut depth, mut quote, mut start) = (0i32, None, 0);
    for (index, c) in pattern.char_indices() {
        match (quote, c) {
            (None, '\'' | '"') => quote = Some(c),
            (Some(open), _) if c == open => quote = None,
            (None, '[' | '(') => depth += 1,
            (None, ']' | ')') => depth -= 1,
            (None, '|') if depth == 0 => {
                branches.push(&pattern[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    branches.push(&pattern[start..]);
    branches
        .iter()
        .map(|branch| {
            let branch = branch.trim();
            if branch.starts_with('/') {
                branch.to_string()
            } else {
                format!("//{branch}")
            }
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

/// 1-based location path of the element a rule was checked on, e.g.
/// `/rsm:CrossIndustryInvoice/rsm:SupplyChainTradeTransaction/ram:IncludedSupplyChainTradeLineItem[2]`.
/// For an attribute, its element.
fn location(xot: &xot::Xot, node: xot::Node, prefixes: &[(String, String)]) -> String {
    let mut steps = Vec::new();
    let mut current = Some(node);
    while let Some(node) = current {
        if let Some(element) = xot.element(node) {
            let name = element.name();
            let (local, namespace) = xot.name_ns_str(name);
            let prefix = prefixes
                .iter()
                .find(|(_, uri)| uri == namespace)
                .map_or("unknown", |(prefix, _)| prefix.as_str());
            let same_name: Vec<xot::Node> = xot
                .parent(node)
                .map(|parent| {
                    xot.children(parent)
                        .filter(|sibling| xot.element(*sibling).is_some_and(|e| e.name() == name))
                        .collect()
                })
                .unwrap_or_default();
            if same_name.len() > 1 {
                let position = same_name
                    .iter()
                    .position(|sibling| *sibling == node)
                    .map_or(1, |p| p + 1);
                steps.push(format!("{prefix}:{local}[{position}]"));
            } else {
                steps.push(format!("{prefix}:{local}"));
            }
        }
        current = xot.parent(node);
    }
    steps.reverse();
    format!("/{}", steps.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_union_branches_get_a_descendant_prefix() {
        assert_eq!(
            context_expression("cac:A[x = 'a|b'] | /ubl:Invoice | cac:B[(1|2)]"),
            "//cac:A[x = 'a|b'] | /ubl:Invoice | //cac:B[(1|2)]"
        );
    }

    #[test]
    fn variable_references_outside_literals() {
        assert_eq!(
            variable_references("($BT-89-path or $a) and '$no' = $a and $u:x"),
            ["BT-89-path", "a", "u:x"]
        );
    }

    #[test]
    fn function_calls_become_dynamic_calls() {
        let functions = [("xr:checkIBAN".to_string(), "fn_xr_checkIBAN".to_string())];
        assert_eq!(
            rewrite_calls(
                "xr:checkIBAN(.) and 'xr:checkIBAN(' and pre-xr:checkIBAN(1)",
                &functions
            ),
            "$fn_xr_checkIBAN(.) and 'xr:checkIBAN(' and pre-xr:checkIBAN(1)"
        );
    }

    #[test]
    fn top_level_and_or_short_circuit() {
        assert_eq!(
            short_circuit("matches($a, 'x') and f($a and $b) mod 97 = 1"),
            "if (matches($a, 'x')) then (boolean(f($a and $b) mod 97 = 1)) else false()"
        );
        assert_eq!(
            short_circuit("not($a and $b) or f($c)"),
            "if (not($a and $b)) then true() else (boolean(f($c)))"
        );
        assert_eq!(
            short_circuit("($a and $b) or $c"),
            "if ((if ($a) then (boolean($b)) else false())) then true() else (boolean($c))"
        );
        assert_eq!(
            short_circuit("if ($a) then $b or $c else $d"),
            "if ($a) then $b or $c else $d"
        );
        assert_eq!(short_circuit("$band"), "$band");
        assert_eq!(
            short_circuit("$a or $b or $c or $d or $e"),
            "$a or $b or $c or $d or $e"
        );
        assert_eq!(short_circuit("ram:or = 1"), "ram:or = 1");
    }

    #[test]
    fn context_functions_get_an_argument() {
        assert_eq!(
            rewrite_context_functions(
                "not(normalize-space( )) and string-length() = 'normalize-space()' and normalize-space(x) and my-string-length()"
            ),
            "not(normalize-space(string(.))) and string-length(string(.)) = 'normalize-space()' and normalize-space(x) and my-string-length()"
        );
    }

    #[test]
    fn path_predicates_become_boolean() {
        assert_eq!(
            rewrite_predicates("count(cac:TaxTotal[cac:TaxSubtotal]) = 1"),
            "count(cac:TaxTotal[boolean(cac:TaxSubtotal)]) = 1"
        );
        assert_eq!(
            rewrite_predicates("a[b[c]/@d][1][last()][. = '[x]'][$v][2 * 3]"),
            "a[boolean(b[boolean(c)]/@d)][1][last()][. = '[x]'][$v][2 * 3]"
        );
        assert_eq!(
            rewrite_predicates("x[ancestor::cac:Party/cbc:ID]"),
            "x[boolean(ancestor::cac:Party/cbc:ID)]"
        );
    }

    #[test]
    fn bindings_are_ordered_by_dependency() {
        let mut globals = HashMap::new();
        globals.insert(
            "b".to_string(),
            Binding {
                text: "$b := root(.) ! ($a + 1)".to_string(),
                references: vec!["a".to_string()],
            },
        );
        globals.insert(
            "a".to_string(),
            Binding {
                text: "$a := root(.) ! (1)".to_string(),
                references: vec![],
            },
        );
        let scope = Scope {
            globals: &globals,
            rule_lets: vec![(
                "r".to_string(),
                Binding {
                    text: "$r := ($b)".to_string(),
                    references: vec!["b".to_string()],
                },
            )],
        };
        assert_eq!(
            scope.wrap("$r = 2"),
            (
                "let $a := root(.) ! (1), $b := root(.) ! ($a + 1), $r := ($b) return ($r = 2)"
                    .to_string(),
                true
            )
        );
        assert_eq!(scope.wrap("/x"), ("/x".to_string(), false));
    }
}
