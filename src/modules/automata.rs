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
        if let Some(matched) = self.greedy_search_impl(expr, true) {
            matched.len() == expr.len()
        } else {
            false
        }
    }

    pub fn greedy_search(&self, expr: &str) -> Option<String> {
        self.greedy_search_impl(expr, false)
    }

    fn greedy_search_impl(&self, expr: &str, full_match: bool) -> Option<String> {
        let mut current_states: Vec<Vec<StatePtr>> = vec![];
        let (mut start, mut end) = (0, -1);

		for transition in transition_iter(expr) {
			match transition {
				TransitionItem::Char(c) => {
					for states in current_states.iter_mut() {
						*states = states
							.iter()
							.filter_map(|s| s.borrow().transition(c))
							.collect();
					}
				},
				TransitionItem::Epsilon((r, anchors)) => {
					if !full_match || anchors.contains(&Anchor::Start) {
						current_states.push(vec![self.start.clone()]);
					}

					for (l, states) in current_states.iter_mut().enumerate() {
						*states = exhaust_epsilons(states, &anchors);

						if end - start < (r - l) as i32 && states.contains(&self.get_end()) {
							start = l as i32;
							end = r as i32;
						}
					}
				}
			}
		}

        if end >= start {
            Some(String::from(&expr[(start as usize)..(end as usize)]))
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

fn exhaust_epsilons(states: &[StatePtr], anchors: &[Anchor]) -> Vec<StatePtr> {
    fn traverse_epsilons(
        destinations: &mut Vec<StatePtr>,
        visited_states: &mut Vec<StatePtr>,
        state: &StatePtr,
        anchors: &[Anchor],
    ) {
        let state_locked = state.borrow();
        let reachables = state_locked.epsilon(anchors);

        if reachables.is_empty() {
            destinations.push(state.clone());
        }

        for candidate in reachables {
            if visited_states.contains(candidate) {
                continue;
            }

            visited_states.push(candidate.clone());
            traverse_epsilons(destinations, visited_states, candidate, anchors);
        }
    }

    let mut destinations: Vec<StatePtr> = Vec::new();
    let mut visited_states: Vec<StatePtr> = Vec::new();

    states
        .iter()
        .for_each(|s| traverse_epsilons(&mut destinations, &mut visited_states, s, anchors));

    destinations
}

fn get_anchors(prev: Option<(usize, char)>, next: Option<(usize, char)>) -> Vec<Anchor> {
    let mut anchors = vec![];
    match (prev, next) {
        (Some((_, p)), Some((_, n))) => {
            if p.is_alphanumeric() != n.is_alphanumeric() {
                anchors.push(Anchor::WordBoundary);
            }
        }
        _ => {
            if prev.is_none() {
                anchors.push(Anchor::Start);
            }
            if next.is_none() {
                anchors.push(Anchor::End);
            }
            anchors.push(Anchor::WordBoundary)
        }
    };
    anchors
}

enum TransitionItem {
	Char(char),
	Epsilon((usize, Vec<Anchor>))
}

struct TransitionIter<'a> {
	it: std::iter::Enumerate<std::str::Chars<'a>>,
	current: Option<(usize, char)>,
	char_next: bool
}

impl<'a> Iterator for TransitionIter<'a> {
	type Item = TransitionItem;

	fn next(&mut self) -> Option<Self::Item> {
		let return_char = self.char_next;
		self.char_next = !self.char_next;

		if return_char {
			self.current.map(|c| TransitionItem::Char(c.1))
		} else {
			let next = self.it.next();
			let pos = self.current.map(|c| c.0 + 1).unwrap_or(0);
			let eps = (pos, get_anchors(self.current, next));
			self.current = next;

			Some(TransitionItem::Epsilon(eps))
		}
	}
}

fn transition_iter(expr: &str) -> TransitionIter {
	return TransitionIter{ it:expr.chars().enumerate(), current: None, char_next:false};
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

        assert_eq!(nfa.greedy_search(""), None);
        assert_eq!(nfa.greedy_search("c"), None);
        assert_eq!(nfa.greedy_search("dc"), None);
        assert_eq!(nfa.greedy_search("cd"), Some(String::from("cd")));
        assert_eq!(nfa.greedy_search("abcde"), Some(String::from("cd")));
    }

    #[test]
    fn nfa_union() {
        let nfa = Automata::from_token('c').or(Automata::from_token('d'));
        assert!(nfa.full_match("c"));
        assert!(nfa.full_match("d"));
        assert!(!nfa.full_match("cd"));
        assert!(!nfa.full_match(""));
        assert!(!nfa.full_match("monty python"));

        assert_eq!(nfa.greedy_search(""), None);
        assert_eq!(nfa.greedy_search("cd"), Some(String::from("c")));
        assert_eq!(nfa.greedy_search("dc"), Some(String::from("d")));
        assert_eq!(nfa.greedy_search("ade"), Some(String::from("d")));
    }

    #[test]
    fn nfa_closure() {
        let nfa = Automata::from_token('a').closure();
        assert!(nfa.full_match(""));
        assert!(nfa.full_match("a"));
        assert!(nfa.full_match("aaa"));
        assert!(!nfa.full_match("b"));

        assert_eq!(nfa.greedy_search(""), Some(String::from("")));
        assert_eq!(nfa.greedy_search("b"), Some(String::from("")));
        assert_eq!(nfa.greedy_search("a"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("ab"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("ba"), Some(String::from("a")));
    }

    #[test]
    fn nfa_plus() {
        let nfa = Automata::from_token('a').plus();
        assert!(!nfa.full_match(""));
        assert!(nfa.full_match("a"));
        assert!(nfa.full_match("aaa"));
        assert!(!nfa.full_match("b"));

        assert_eq!(nfa.greedy_search(""), None);
        assert_eq!(nfa.greedy_search("b"), None);
        assert_eq!(nfa.greedy_search("a"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("ab"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("ba"), Some(String::from("a")));
    }

    #[test]
    fn nfa_optional() {
        let nfa = Automata::from_token('a').optional();
        assert!(nfa.full_match(""));
        assert!(nfa.full_match("a"));
        assert!(!nfa.full_match("b"));
        assert!(!nfa.full_match("ab"));
        assert!(!nfa.full_match("ba"));

        assert_eq!(nfa.greedy_search(""), Some(String::from("")));
        assert_eq!(nfa.greedy_search("b"), Some(String::from("")));
        assert_eq!(nfa.greedy_search("a"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("ab"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("ba"), Some(String::from("a")));
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

    // #[test]
    // fn nfa_greedy_search() {
    //     let nfa
    // }
}
