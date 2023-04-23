use super::automata::{closure, concat, optional, or, plus, Automata};
use super::state::Anchor;

const CONCAT_CHAR: char = 27 as char;
const POP_ERR: &str = "error popping from stack";

pub fn parse(expr: &str) -> Result<Automata, &'static str> {
    let postfix = to_postfix(expr);
        let mut automata_stack = Vec::<Automata>::new();

        for c in postfix.chars() {
            match c {
                '^' => automata_stack.push(Automata::from_anchor(Anchor::Start)),
                '$' => automata_stack.push(Automata::from_anchor(Anchor::End)),
                '|' => {
                    let b = automata_stack.pop().ok_or(POP_ERR)?;
                    let a = automata_stack.pop().ok_or(POP_ERR)?;
                    automata_stack.push(or(a, b));
                }
                CONCAT_CHAR => {
                    let b = automata_stack.pop().ok_or(POP_ERR)?;
                    let a = automata_stack.pop().ok_or(POP_ERR)?;
                    automata_stack.push(concat(a, b));
                }
                '*' => {
                    let a = automata_stack.pop().ok_or(POP_ERR)?;
                    automata_stack.push(closure(a));
                }
                '?' => {
                    let a = automata_stack.pop().ok_or(POP_ERR)?;
                    automata_stack.push(optional(a));
                }
                '+' => {
                    let a = automata_stack.pop().ok_or(POP_ERR)?;
                    automata_stack.push(plus(a));
                }
                '.' => automata_stack.push(Automata::from_lambda(|_| true)),
                _ => automata_stack.push(Automata::from_token(c)),
            }
        }

        if let Some(automata) = automata_stack.pop() {
            Ok( automata )
        } else {
            Err("Error parsing expression given!")
        }
}

fn to_postfix(expr: &str) -> String {
    let modified_expr = add_concat_char(expr);
    let mut postfix = String::new();
    let mut operator_stack = Vec::<char>::new();
    let precedence = |c| match c {
        '|' => 0,
        CONCAT_CHAR => 1,
        _ => 2,
    };

    for c in modified_expr.chars() {
        match c {
            '|' | CONCAT_CHAR | '*' | '?' | '+' => {
                let curr_precedence = precedence(c);
                while let Some(&op) = operator_stack.last() {
                    if (op == '(') || (precedence(op) < curr_precedence) { break; }
                    operator_stack.pop();
                    postfix.push(op);
                }
                operator_stack.push(c);
            }
            '(' => operator_stack.push(c),
            ')' => {
                while let Some(token) = operator_stack.pop() {
                    if token == '(' { break; }
                    postfix.push(token);
                }
            }
            _ => postfix.push(c),
        }
    }

    while let Some(op) = operator_stack.pop() {
        postfix.push(op);
    }

    postfix
}

fn add_concat_char(expr: &str) -> String {
    let mut result = String::new();
    let mut it = expr.chars();
    if let Some(first) = it.next() {
        result.push(first);
        let mut prev = first;

        for c in it {
            if !matches!(prev, '(' | '|') && !matches!(c, ')' | '|' | '*' | '?' | '+') {
                result.push(CONCAT_CHAR);
            }

            result.push(c);
            prev = c;
        }
    }

    result
}
