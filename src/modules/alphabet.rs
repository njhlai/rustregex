use crate::union;

use super::monadic_parser::MonadicParser;


// Parsers defining the alphabet of Regex

/// Constructs a new [`MonadicParser`] for [`prim@char`].
pub fn any() -> MonadicParser<char> {
    MonadicParser::new(|expr| Some((expr.chars().next()?, &expr[1..])))
}

/// Returns a [`MonadicParser`] which parses `ch`.
pub fn character(ch: char) -> MonadicParser<char> {
    any().filter(move |&c| c == ch)
}

/// Returns a [`MonadicParser`] which parses `s`.
pub fn string(s: &'static str) -> MonadicParser<&'static str> {
    MonadicParser::new(move |expr| expr.strip_prefix(s).map(|rst| (s, rst)))
}

/// Returns a [`MonadicParser`] which parses [`prim@char`] which is an ASCII digit.
pub fn digit() -> MonadicParser<u32> {
    any().map(|c| c.to_digit(10))
}

/// Returns a [`MonadicParser`] which parses numbers.
pub fn number() -> MonadicParser<u32> {
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
pub fn escaped() -> MonadicParser<char> {
    character('\\') >> any()
}
