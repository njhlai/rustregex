use super::monadic_parser::MonadicParser;

/// Constructs a new [`MonadicParser`] for [`prim@char`].
///
/// Note: The resulting [`MonadicParser`] **does not** consume the given `expr` in its `parse` function.
fn init() -> MonadicParser<char> {
    MonadicParser::new(|expr| expr.chars().next())
}

// Parsers defining the alphabet of Regex

/// Returns a [`MonadicParser`] which parses `ch`.
pub fn character(ch: char) -> MonadicParser<char> {
    init().filter(move |&c| c == ch).one()
}

/// Returns a [`MonadicParser`] which parses any legal [`prim@char`].
pub fn legal_character() -> MonadicParser<char> {
    init()
        .exclude(move |&c| matches!(c, '^' | '$' | '|' | '*' | '?' | '+' | '.' | '\\' | '-' | '(' | ')' | '{' | '}' | '[' | ']'))
        .one()
}

/// Returns a [`MonadicParser`] which parses [`prim@char`] which is an ASCII digit.
fn digit() -> MonadicParser<usize> {
    init()
        .filter(char::is_ascii_digit)
        .map(|c| usize::try_from(c.to_digit(10)?).ok())
        .one()
}

/// Returns a [`MonadicParser`] which parses numbers.
pub fn number() -> MonadicParser<usize> {
    digit().repeat().filter(|v| !v.is_empty()).map(|v| {
        let mut result = 0;

        for d in v {
            result *= 10;
            result += d;
        }

        Some(result)
    })
}

/// Returns a [`MonadicParser`] which parses escaped character satisfying `predicate`.
pub fn escaped_character(predicate: fn(&char) -> bool) -> MonadicParser<char> {
    MonadicParser::new(move |expr| {
        let mut it = expr.chars();

        if let Some('\\') = it.next() {
            if let Some(c) = it.next() {
                if predicate(&c) {
                    *expr = it.collect();

                    Some(c)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    })
}
