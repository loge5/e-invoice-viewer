//! Rewrites of XPath expressions from the official rule artefacts, so that
//! the engine evaluates them as Saxon does (see the parent module).

/// Rewrites calls of the rule set's functions to dynamic calls of their
/// variables, outside string literals.
pub(super) fn rewrite_calls(expression: &str, functions: &[(String, String)]) -> String {
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
pub(super) fn variable_references(expression: &str) -> Vec<String> {
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
pub(super) fn short_circuit(expression: &str) -> String {
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
pub(super) fn rewrite_context_functions(expression: &str) -> String {
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
pub(super) fn rewrite_predicates(expression: &str) -> String {
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

/// Turns an XSLT match pattern into an XPath expression selecting all nodes
/// it matches.
pub(super) fn context_expression(pattern: &str) -> String {
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

/// Rewrites `document('name')` of one of `documents` to
/// `doc('<scheme>name')`, which reads the document loaded under that
/// address; outside string literals.
pub(super) fn rewrite_document_calls(
    expression: &str,
    documents: &[(&str, &str)],
    scheme: &str,
) -> String {
    let mut out = expression.to_string();
    for (name, _) in documents {
        for quote in ['\'', '"'] {
            out = out.replace(
                &format!("document({quote}{name}{quote})"),
                &format!("doc({quote}{scheme}{name}{quote})"),
            );
        }
    }
    out
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
    fn document_calls_read_bundled_documents() {
        let documents = [("codes.xml", "<codes/>")];
        assert_eq!(
            rewrite_document_calls(
                "document('codes.xml')/c[@v = .] and document(\"codes.xml\") and document('other.xml')",
                &documents,
                "eiv-rules:/"
            ),
            "doc('eiv-rules:/codes.xml')/c[@v = .] and doc(\"eiv-rules:/codes.xml\") and document('other.xml')"
        );
    }
}
