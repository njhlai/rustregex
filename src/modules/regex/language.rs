use super::{Automata, Error};

use super::ast::AbstractSyntaxTree;
use super::context::Context;
use super::grammar::Grammar;

/// A modal for formal language
pub struct Language<T> {
    grammar: Grammar<T>,
}

impl<T: AbstractSyntaxTree + 'static> Language<T> {
    /// Constructs a [`Language`] associated to the formal grammar defined by [`Grammar`].
    pub fn new(grammar: Grammar<T>) -> Self {
        Language { grammar }
    }

    /// Parses `expr` using [`Language`]'s grammar.
    pub fn parse<C: Context<T>>(&self, expr: &str, mut context: C) -> Result<Automata, Error> {
        context
            .process(
                self.syntax(expr)
                    .ok_or_else(|| Error::from("Internal error: `expr` parsed into an `None` Expression"))?,
            )
            .compile()
    }

    /// Returns the syntax representation of `expr` using [`Language`]'s grammar.
    pub fn syntax(&self, expr: &str) -> Option<T> {
        self.grammar.parse(expr).map(|(t, _)| t)
    }
}
