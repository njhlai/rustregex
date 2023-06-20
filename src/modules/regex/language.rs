use super::{Automata, Error};

use super::ast::AbstractSyntaxTree;
use super::grammar::{self, Grammar, Regex};

/// A modal for formal language
pub struct Language<T> {
    grammar: Grammar<T>,
}

impl<T: AbstractSyntaxTree + 'static> Language<T> {
    /// Constructs a [`Language`] associated to the formal grammar compiled using specification `spec`.
    pub fn new(spec: fn() -> Grammar<T>) -> Self {
        Language { grammar: Grammar::compile(spec) }
    }

    /// Parses `expr` using [`Language`]'s grammar.
    pub fn parse(&self, expr: &str) -> Result<Automata, Error> {
        self.syntax(expr)
            .ok_or_else(|| Error::from("Internal error: expr parsed into an empty Expression"))?
            .compile()
    }

    /// Returns the syntax representation of `expr` using [`Language`]'s grammar.
    pub fn syntax(&self, expr: &str) -> Option<T> {
        self.grammar.parse(expr).map(|(t, _)| t)
    }
}

/// Returns the [`Language`] defining the Regex language.
pub fn regex() -> Language<Regex> {
    Language::<Regex>::new(grammar::regex)
}
