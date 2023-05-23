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
    let modified_expr = add_concat_char(expr)?;
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
