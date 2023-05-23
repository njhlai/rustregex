use std::str::Chars;

use super::automata::Automata;
use super::error::Error;
use super::state::Anchor;

const CONCAT_CHAR: char = 27 as char;
const POP_ERR: &str = "error popping from stack";

pub fn parse(expr: &str) -> Result<Automata, Error> {
    let postfix = to_postfix(expr)?;
    let mut automata_stack = Vec::<Automata>::new();

    let mut it = postfix.chars();
    while let Some(c) = it.next() {
        match c {
            '^' => automata_stack.push(Automata::from_anchor(Anchor::Start)),
            '$' => automata_stack.push(Automata::from_anchor(Anchor::End)),
            '|' => {
                let b = automata_stack.pop().ok_or_else(|| Error::from(POP_ERR))?;
                let a = automata_stack.pop().ok_or_else(|| Error::from(POP_ERR))?;
                automata_stack.push(a.or(b));
            }
            CONCAT_CHAR => {
                let b = automata_stack.pop().ok_or_else(|| Error::from(POP_ERR))?;
                let a = automata_stack.pop().ok_or_else(|| Error::from(POP_ERR))?;
                automata_stack.push(a.concat(b));
            }
            '*' => {
                let a = automata_stack.pop().ok_or_else(|| Error::from(POP_ERR))?;
                automata_stack.push(a.closure());
            }
            '?' => {
                let a = automata_stack.pop().ok_or_else(|| Error::from(POP_ERR))?;
                automata_stack.push(a.optional());
            }
            '+' => {
                let a = automata_stack.pop().ok_or_else(|| Error::from(POP_ERR))?;
                automata_stack.push(a.plus());
            }
            '.' => automata_stack.push(Automata::from_lambda(|_| true)),
            '\\' => {
                let d = it.next().unwrap();
                automata_stack.push(match d {
                    'b' => Automata::from_anchor(Anchor::WordBoundary),
                    'd' => Automata::from_lambda(|x| x.is_ascii_digit()),
                    'D' => Automata::from_lambda(|x| !x.is_ascii_digit()),
                    'w' => Automata::from_lambda(|x| x.is_ascii_alphanumeric()),
                    'W' => Automata::from_lambda(|x| !x.is_ascii_alphanumeric()),
                    's' => Automata::from_lambda(|x| x.is_ascii_whitespace()),
                    'S' => Automata::from_lambda(|x| !x.is_ascii_whitespace()),
                    '^' | '$' | '|' | '*' | '?' | '+' | '.' | '\\' | '(' | ')' | '{' | '}' => Automata::from_token(d),
                    't' => Automata::from_token('\t'),
                    'n' => Automata::from_token('\n'),
                    'r' => Automata::from_token('\r'),
                    'v' => Automata::from_token('\x0b'),
                    'f' => Automata::from_token('\x0c'),
                    '0' => Automata::from_token('\0'),
                    _ => return Err(Error::from(format!("Unknown escape sequence \\{d}").as_str())),
                });
            }
            _ => automata_stack.push(Automata::from_token(c)),
        }
    }

    if automata_stack.len() > 1 {
        Err(Error::from("Internal error: final stack is too large"))
    } else {
        automata_stack
            .pop()
            .ok_or_else(|| Error::from("Internal error: final stack is empty"))
    }
}

fn to_postfix(expr: &str) -> Result<String, Error> {
    let modified_expr = modify_expression(expr)?;
    let mut postfix = String::new();
    let mut operator_stack = Vec::<char>::new();
    let precedence = |c| match c {
        '|' => 0,
        CONCAT_CHAR => 1,
        _ => 2,
    };

    let mut it = modified_expr.chars();
    while let Some(c) = it.next() {
        match c {
            '\\' => {
                postfix.push(c);
                postfix.push(
                    it.next()
                        .ok_or(Error::from("\\ does not escape anything"))?,
                );
            }
            '|' | CONCAT_CHAR | '*' | '?' | '+' => {
                let curr_precedence = precedence(c);
                while let Some(&op) = operator_stack.last() {
                    if (op == '(') || (precedence(op) < curr_precedence) {
                        break;
                    }
                    operator_stack.pop();
                    postfix.push(op);
                }
                operator_stack.push(c);
            }
            '(' => operator_stack.push(c),
            ')' => {
                while let Some(token) = operator_stack.pop() {
                    if token == '(' {
                        break;
                    }
                    postfix.push(token);
                }
            }
            _ => postfix.push(c),
        }
    }

    while let Some(op) = operator_stack.pop() {
        if op == '(' {
            return Err(Error::from("Unmatched marking paranthesis '('"));
        }

        postfix.push(op);
    }

    Ok(postfix)
}

fn modify_expression(expr: &str) -> Result<String, Error> {
    add_concat_char(expand_quantifiers(expr)?.as_str())
}

fn add_concat_char(expr: &str) -> Result<String, Error> {
    if let Some(pos) = expr.find(CONCAT_CHAR) {
        return Err(Error::from(format!("Illegal character at index {pos}").as_str()));
    }

    let mut result = String::new();
    let mut it = expr.chars();
    if let Some(first) = it.next() {
        result.push(first);
        let mut prev = first;

        let mut prev_was_escaped = false;
        for c in it {
            if (prev_was_escaped || !matches!(prev, '(' | '|' | '\\')) && !matches!(c, ')' | '|' | '*' | '?' | '+') {
                result.push(CONCAT_CHAR);
            }

            result.push(c);
            prev_was_escaped = !prev_was_escaped && prev == '\\';
            prev = c;
        }
    }

    Ok(result)
}

fn expand_quantifiers(expr: &str) -> Result<String, Error> {
    let mut result_stack = Vec::<String>::new();

    let mut it = expr.chars();
    while let Some(c) = it.next() {
        match c {
            '\\' => {
                let mut curr = String::from(c);
                curr.push(
                    it.next()
                        .ok_or(Error::from("\\ does not escape anything"))?,
                );

                result_stack.push(curr);
            }
            ')' => {
                let pos = result_stack
                    .iter()
                    .rev()
                    .position(|x| x == "(")
                    .ok_or_else(|| Error::from("Unmatched marking paranthesis ')'"))?;
                let mut curr = result_stack
                    .drain(result_stack.len().saturating_sub(pos + 1)..)
                    .collect::<String>();
                curr.push(c);

                result_stack.push(curr);
            }
            '{' => {
                if let Some(word) = result_stack.pop() {
                    if matches!(word.as_str(), "(" | "|" | "*" | "?" | "+") {
                        return Err(Error::from("Invalid preceeding regular expression prior to quantifier expression"));
                    }

                    let (min, max) = read_quantifier(&mut it)?;
                    let mut curr_word = String::new();

                    for _ in 0..min {
                        curr_word.push_str(word.as_str());
                    }

                    if let Some(max) = max {
                        let maybe = format!("{word}?");
                        for _ in 0..max.saturating_sub(min) {
                            curr_word.push_str(maybe.as_str());
                        }
                    } else {
                        curr_word.push('+');
                    }

                    result_stack.push(curr_word);
                }
            }
            _ => result_stack.push(String::from(c)),
        }
    }

    Ok(result_stack.join(""))
}

fn read_quantifier(it: &mut Chars) -> Result<(usize, Option<usize>), Error> {
    fn parse_to_usize(num: usize, c: char) -> Result<usize, Error> {
        let mut result = num * 10;
        result += usize::try_from(
            c.to_digit(10)
                .ok_or_else(|| Error::from("Non-numeric character in quantifier range"))?,
        )
        .map_err(|e| Error::from(&e.to_string()))?;

        Ok(result)
    }

    let (mut min, mut max) = (0, None);

    for c in it.by_ref() {
        if c == '}' {
            return Ok((min, Some(min)));
        } else if c == ',' {
            break;
        }

        min = parse_to_usize(min, c)?;
    }

    for c in it.by_ref() {
        if c == '}' {
            break;
        }

        max = Some(parse_to_usize(max.unwrap_or_default(), c)?);
    }

    Ok((min, max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_read_quantifier_normal() {
        let mut testquantifier = r"30,5935}a sdf\\ad\(f}".chars();

        assert_eq!(Ok((30, Some(5935))), read_quantifier(&mut testquantifier))
    }

    #[test]
    fn parser_read_quantifier_no_max() {
        let mut testquantifier = r"123,}a sdf\\ad\(f}".chars();

        assert_eq!(Ok((123, None)), read_quantifier(&mut testquantifier))
    }

    #[test]
    fn parser_read_quantifier_no_min() {
        let mut testquantifier = r",45}a sdf\\ad\(f}".chars();

        assert_eq!(Ok((0, Some(45))), read_quantifier(&mut testquantifier))
    }

    #[test]
    fn parser_read_quantifier_specific() {
        let mut testquantifier = r"7890}a sdf\\ad\(f}".chars();

        assert_eq!(Ok((7890, Some(7890))), read_quantifier(&mut testquantifier))
    }

    #[test]
    fn parser_expand_quantifier_simple() {
        let testexpr = "a{3,6}";
        let expanded_testexpr = expand_quantifiers(testexpr);

        assert!(expanded_testexpr.is_ok());
        assert_eq!(String::from("aaaa?a?a?"), expanded_testexpr.unwrap());
    }

    #[test]
    fn parser_expand_quantifier_realistic() {
        let testexpr = r"@\w+\.\w{2,3}(\.\w{2,})?";
        let expanded_testexpr = expand_quantifiers(testexpr);

        assert!(expanded_testexpr.is_ok());
        assert_eq!(String::from(r"@\w+\.\w\w\w?(\.\w\w+)?"), expanded_testexpr.unwrap());
    }

    #[test]
    fn parser_expand_quantifier_nested_brackets() {
        let testexpr = "((x{2,}y){3,4}z{5}){,3}";
        let expanded_testexpr = expand_quantifiers(testexpr);

        assert!(expanded_testexpr.is_ok());
        assert_eq!(
            String::from(
                "((xx+y)(xx+y)(xx+y)(xx+y)?zzzzz)?((xx+y)(xx+y)(xx+y)(xx+y)?zzzzz)?((xx+y)(xx+y)(xx+y)(xx+y)?zzzzz)?"
            ),
            expanded_testexpr.unwrap()
        );
    }

    #[test]
    fn parser_expand_quantifier_escaped_brackets() {
        let testexpr = r"(\{\(x){2,5}\}";
        let expanded_testexpr = expand_quantifiers(testexpr);

        assert!(expanded_testexpr.is_ok());
        assert_eq!(
            String::from(
                r"(\{\(x)(\{\(x)(\{\(x)?(\{\(x)?(\{\(x)?\}"
            ),
            expanded_testexpr.unwrap()
        );
    }
}
