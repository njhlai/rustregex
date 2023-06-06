use super::{
    grammar,
    grammar::{Grammar, Regex},
};
use super::automata::Automata;
use super::error::Error;

/// A modal for formal language
pub struct Language<S> {
    grammar: Grammar<S>,
}

impl<S: 'static> Language<S> {
    // pub fn new(spec: Spec<T>) -> Self {
    //     Language { grammar: grammar::compile(spec) }
    // }

    /// Constructs a [`Language`] associated to the formal grammar compiled using specification `spec`.
    pub fn new(spec: fn() -> Grammar<S>) -> Self {
        Language { grammar: Grammar::compile(spec) }
    }

    /// Parses `expr` using [`Language`]'s grammar.
    pub fn parse(&self, expr: & str) -> Option<S> {
        self.grammar.parse(expr).map(|(s, _)| s)
    }

}

/// Returns the [`Language`] defining the Regex language.
pub fn regex() -> Language<Regex> {
    Language::<Regex>::new(grammar::regex)
}
