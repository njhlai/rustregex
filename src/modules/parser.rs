use super::automata::{concat, or, closure, plus, optional, Automata};
use super::state::Anchor;
use super::utils;

const POP_ERR: &str = "error popping from stack";

pub struct Parser {
    automata: Automata,
}

impl Parser {
    pub fn new(expr: &str) -> Result<Self, &'static str> {
        let postfix = utils::to_postfix(expr);
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
                utils::CONCAT_CHAR => {
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
            Ok(Parser { automata })
        } else {
            Err("Error parsing expression given!")
        }
    }

    pub fn full_match(&self, expr: &str) -> bool {
        self.automata.full_match(expr)
    }

    pub fn greedy_search(&self, expr: &str) -> Option<String> {
        self.automata.greedy_search(expr)
    }
}