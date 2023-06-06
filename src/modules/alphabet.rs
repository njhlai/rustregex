use super::monadic_parser::MonadicParser;

/// Constructs a new [`MonadicParser`] for [`prim@char`].
fn init() -> MonadicParser<char> {
    MonadicParser::new(|expr| Some((expr.chars().next()?, &expr[1..])))
}

// Parsers defining the alphabet of Regex

/// Returns a [`MonadicParser`] which parses `ch`.
pub fn character(ch: char) -> MonadicParser<char> {
    init().filter(move |&c| c == ch)
}

/// Returns a [`MonadicParser`] which parses any legal [`prim@char`].
pub fn legal_character() -> MonadicParser<char> {
    init()
        .exclude(move |&c| matches!(c, '^' | '$' | '|' | '*' | '?' | '+' | '.' | '\\' | '-' | '(' | ')' | '{' | '}' | '[' | ']'))
}

/// Returns a [`MonadicParser`] which parses [`prim@char`] which is an ASCII digit.
fn digit() -> MonadicParser<usize> {
    init()
        .filter(char::is_ascii_digit)
        .map(|c| usize::try_from(c.to_digit(10)?).ok())
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
    character('\\')
        .chain(init().filter(predicate))
        .map(|(_, c)| Some(c))
}
