use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use super::state::{Anchor, AnchorState, LambdaState, State, TokenState, TrivialState};

type StatePtr = Rc<RefCell<dyn State>>;

pub struct Automata {
    start: StatePtr,
    end: Rc<RefCell<TrivialState>>,
}

impl Automata {
    pub fn from_token(token: char) -> Self {
        let end = TrivialState::make_rc();
        let start = Rc::new(RefCell::new(TokenState::new(token, end.clone())));

        Automata { start, end }
    }

    pub fn from_lambda(lambda: fn(char) -> bool) -> Self {
        let end = TrivialState::make_rc();
        let start = Rc::new(RefCell::new(LambdaState::new(lambda, end.clone())));

        Automata { start, end }
    }

    pub fn from_anchor(anchor: Anchor) -> Self {
        let end = TrivialState::make_rc();
        let start = Rc::new(RefCell::new(AnchorState::new(anchor, end.clone())));

        Automata { start, end }
    }

    pub fn concat(mut self, other: Automata) -> Self {
        self.push_to_end(other.start.clone());
        self.end = other.end.clone();
        self
    }

    pub fn or(mut self, mut other: Automata) -> Self {
        let start = TrivialState::make_rc();
        let end = TrivialState::make_rc();

        start.borrow_mut().push(self.start.clone());
        start.borrow_mut().push(other.start.clone());

        self.push_to_end(end.clone());
        other.push_to_end(end.clone());

        self.start = start;
        self.end = end;
        self
    }

    pub fn closure(mut self) -> Self {
        let start = TrivialState::make_rc();
        let end = TrivialState::make_rc();

        start.borrow_mut().push(self.start.clone());
        start.borrow_mut().push(end.clone());

        self.push_to_end(self.start.clone());
        self.push_to_end(end.clone());

        self.start = start;
        self.end = end;
        self
    }

    pub fn optional(mut self) -> Self {
        let start = TrivialState::make_rc();
        let end = TrivialState::make_rc();

        start.borrow_mut().push(self.start.clone());
        start.borrow_mut().push(end.clone());

        self.push_to_end(end.clone());

        self.start = start;
        self.end = end;
        self
    }

    pub fn plus(mut self) -> Self {
        let start = TrivialState::make_rc();
        let end = TrivialState::make_rc();

        start.borrow_mut().push(self.start.clone());

        self.push_to_end(self.start.clone());
        self.push_to_end(end.clone());

        self.start = start;
        self.end = end;
        self
    }

    pub fn full_match(&self, expr: &str) -> bool {
        let mut current_states = exhaust_epsilons(vec![self.start.clone()], &Some(Anchor::Start));

        for c in expr.chars() {
            current_states = exhaust_epsilons(
                current_states
                    .iter()
                    .filter_map(|s| s.borrow().transition(c))
                    .collect(),
                &None,
            );
        }
        current_states = exhaust_epsilons(current_states, &Some(Anchor::End));

        current_states.contains(&self.get_end())
    }

    pub fn greedy_search(&self, expr: &str) -> Option<String> {
        let mut current_states = vec![];
        let (mut start, mut end) = (0, 0);

        let mut expr_peekable = expr.chars().enumerate().peekable();
        let mut anchor = Some(Anchor::Start);
        while let Some((r, c)) = expr_peekable.next() {
            current_states.push(exhaust_epsilons(vec![self.start.clone()], &anchor));

            anchor = if let Some((_, peek_c)) = expr_peekable.peek() {
                if c.is_alphanumeric() == peek_c.is_alphanumeric() {
                    None
                } else {
                    Some(Anchor::WordBoundary)
                }
            } else {
                Some(Anchor::End)
            };

            let mut found = false;
            for (l, states) in current_states.iter_mut().enumerate() {
                *states = exhaust_epsilons(
                    states
                        .iter()
                        .filter_map(|s| s.borrow().transition(c))
                        .collect(),
                    &anchor,
                );

                if !found && states.contains(&self.get_end()) && end - start < r - l + 1 {
                    start = l;
                    end = r + 1;
                    found = true;
                }
            }
        }

        if end - start > 0 {
            Some(String::from(&expr[start..end]))
        } else {
            None
        }
    }

    fn get_end(&self) -> StatePtr {
        // the `clone' function only clones the reference-counted pointer, so this should be ok...
        self.end.clone() as StatePtr
    }

    // It's not necessary to consume a mutable reference, but this function does modify
    // the underlying states. Requiring a mutable reference makes this clearer.
    fn push_to_end(&mut self, state: StatePtr) {
        self.end.borrow_mut().push(state);
    }
}

impl Debug for Automata {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("NFA").field("start", &self.start).finish()
    }
}

fn exhaust_epsilons(states: Vec<StatePtr>, anchor: &Option<Anchor>) -> Vec<StatePtr> {
    fn traverse_epsilons(
        destinations: &mut Vec<StatePtr>,
        visited_states: &mut Vec<StatePtr>,
        state: &StatePtr,
        anchor: &Option<Anchor>,
    ) {
        let state_locked = state.borrow();
        let reachables = state_locked.epsilon(anchor);

        if reachables.is_empty() {
            destinations.push(state.clone());
        }

        for candidate in reachables {
            if visited_states.contains(candidate) {
                continue;
            }

            visited_states.push(candidate.clone());
            traverse_epsilons(destinations, visited_states, candidate, anchor);
        }
    }

    let mut destinations: Vec<StatePtr> = Vec::new();
    let mut visited_states: Vec<StatePtr> = Vec::new();

    states
        .iter()
        .for_each(|s| traverse_epsilons(&mut destinations, &mut visited_states, s, anchor));

    destinations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nfa_concat() {
        let nfa = Automata::from_token('c').concat(Automata::from_token('d'));
        assert!(nfa.full_match("cd"));
        assert!(!nfa.full_match("c"));
        assert!(!nfa.full_match("d"));
        assert!(!nfa.full_match(""));
        assert!(!nfa.full_match("monty python"));
    }

    #[test]
    fn nfa_union() {
        let nfa = Automata::from_token('c').or(Automata::from_token('d'));
        assert!(nfa.full_match("c"));
        assert!(nfa.full_match("d"));
        assert!(!nfa.full_match("cd"));
        assert!(!nfa.full_match(""));
        assert!(!nfa.full_match("monty python"));
    }

    #[test]
    fn nfa_closure() {
        let nfa = Automata::from_token('a').closure();
        assert!(nfa.full_match(""));
        assert!(nfa.full_match("a"));
        assert!(nfa.full_match("aaa"));
        assert!(!nfa.full_match("b"));
    }

    #[test]
    fn nfa_plus() {
        let nfa = Automata::from_token('a').plus();
        assert!(!nfa.full_match(""));
        assert!(nfa.full_match("a"));
        assert!(nfa.full_match("aaa"));
        assert!(!nfa.full_match("b"));
    }

    #[test]
    fn nfa_optional() {
        let nfa = Automata::from_token('a').optional();
        assert!(nfa.full_match(""));
        assert!(nfa.full_match("a"));
        assert!(!nfa.full_match("b"));
        assert!(!nfa.full_match("ab"));
        assert!(!nfa.full_match("ba"));
    }

    #[test]
    fn nfa_full_match() {
        // (ab?)*|c
        let nfa = Automata::from_token('a')
            .concat(Automata::from_token('b').optional())
            .closure()
            .or(Automata::from_token('c'));
        assert!(nfa.full_match("abaaaaaa"));
        assert!(nfa.full_match("c"));
        assert!(nfa.full_match(""));
        assert!(!nfa.full_match("bb"));
        assert!(!nfa.full_match("aaaaaaac"));
        assert!(!nfa.full_match("cc"));
    }
}
