
use super::{parser::{parse, Error}, automata::Automata};

pub struct RegExp {
    automata: Automata,
}

impl RegExp {
    pub fn new(expr: &str) -> Result<Self, Error> {
        Ok( RegExp{ automata: parse(expr)? } )
    }

    pub fn full_match(&self, expr: &str) -> bool {
        self.automata.full_match(expr)
    }

    pub fn greedy_search(&self, expr: &str) -> Option<String> {
        self.automata.greedy_search(expr)
    }
}
