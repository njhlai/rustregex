use std::iter;

use super::{Automata, Error};

use super::grammar::{
    Anchor, BasicExpression, CharacterClass, CharacterGroup, CharacterGroupItem, CharacterRange, Expression, Group, Match,
    Quantifiable, Quantified, Quantifier, SubExpression,
};

/// A trait that allows types to be compiled into an [`Automata`].
pub trait AbstractSyntaxTree {
    /// Compiles type into an [`Automata`]
    fn compile(&self) -> Result<Automata, Error>;
}

/// Folds a non-empty `Iterator<Item = Result<T, Error>>` into a single [`Result<T, Error>`] using `f`.
fn fold<T, I: Iterator<Item = Result<T, Error>>, F: Fn(T, T) -> T>(mut it: I, f: F) -> Result<T, Error> {
    let initial = it
        .next()
        .ok_or_else(|| Error::from("Internal Error: Iterator was expected to be non-empty"))?;

    it.fold(initial, |acc, elem| Ok(f(acc?, elem?)))
}

// Implementation of AbstractSyntaxTree for elements of Regex

impl AbstractSyntaxTree for Expression {
    fn compile(&self) -> Result<Automata, Error> {
        fold(self.iter().map(AbstractSyntaxTree::compile), Automata::or)
    }
}

impl AbstractSyntaxTree for SubExpression {
    fn compile(&self) -> Result<Automata, Error> {
        fold(self.iter().map(AbstractSyntaxTree::compile), Automata::concat)
    }
}

impl AbstractSyntaxTree for BasicExpression {
    fn compile(&self) -> Result<Automata, Error> {
        match self {
            BasicExpression::Anchor(anchor) => anchor.compile(),
            BasicExpression::Quantified(quantified) => quantified.compile(),
        }
    }
}

impl AbstractSyntaxTree for Anchor {
    fn compile(&self) -> Result<Automata, Error> {
        Ok(Automata::from_anchor(*self))
    }
}

impl AbstractSyntaxTree for Quantified {
    fn compile(&self) -> Result<Automata, Error> {
        let (quantifiable, quantifier) = self;
        let make = || quantifiable.compile();

        match quantifier {
            None => make(),
            Some(Quantifier::ZeroOrMore) => Ok(make()?.closure()),
            Some(Quantifier::OneOrMore) => Ok(make()?.plus()),
            Some(Quantifier::ZeroOrOne) => Ok(make()?.optional()),
            Some(Quantifier::Range((lower, maybe_upper))) => {
                let lower_autos = (0..*lower).map(|_| make());

                let upper: Vec<Result<Automata, Error>> = if let Some(upper) = maybe_upper {
                    (*lower..*upper).map(|_| Ok(make()?.optional())).collect()
                } else {
                    iter::once(Ok(make()?.closure())).collect()
                };

                fold(lower_autos.chain(upper), Automata::concat)
            }
        }
    }
}

impl AbstractSyntaxTree for Quantifiable {
    fn compile(&self) -> Result<Automata, Error> {
        match self {
            Quantifiable::Group(g) => g.compile(),
            Quantifiable::Match(m) => m.compile(),
            Quantifiable::Backreference(_) => Err(Error::from("Internal Error: Backreference not implemented!")),
        }
    }
}

impl AbstractSyntaxTree for Group {
    fn compile(&self) -> Result<Automata, Error> {
        println!("Non capturing mode: {}", self.0);

        self.1.compile()
    }
}

impl AbstractSyntaxTree for Match {
    fn compile(&self) -> Result<Automata, Error> {
        match self {
            Match::Any => Ok(Automata::from_lambda(|_| true)),
            Match::CharacterClass(cc) => cc.compile(),
            Match::CharacterGroup(cg) => cg.compile(),
            Match::Char(c) => c.compile(),
        }
    }
}

impl AbstractSyntaxTree for CharacterClass {
    fn compile(&self) -> Result<Automata, Error> {
        Ok(match self {
            CharacterClass::Alphanumeric => Automata::from_lambda(|x| x.is_ascii_alphanumeric()),
            CharacterClass::NotAlphanumeric => Automata::from_lambda(|x| !x.is_ascii_alphanumeric()),
            CharacterClass::Digit => Automata::from_lambda(|x| x.is_ascii_digit()),
            CharacterClass::NotDigit => Automata::from_lambda(|x| !x.is_ascii_digit()),
            CharacterClass::Whitespace => Automata::from_lambda(|x| x.is_ascii_whitespace()),
            CharacterClass::NotWhitespace => Automata::from_lambda(|x| !x.is_ascii_whitespace()),
        })
    }
}

impl AbstractSyntaxTree for CharacterGroup {
    fn compile(&self) -> Result<Automata, Error> {
        fold(self.iter().map(AbstractSyntaxTree::compile), Automata::or)
    }
}

impl AbstractSyntaxTree for CharacterGroupItem {
    fn compile(&self) -> Result<Automata, Error> {
        match self {
            CharacterGroupItem::CharacterClass(cc) => cc.compile(),
            CharacterGroupItem::CharacterRange(cr) => cr.compile(),
            CharacterGroupItem::Char(c) => c.compile(),
        }
    }
}

impl AbstractSyntaxTree for CharacterRange {
    fn compile(&self) -> Result<Automata, Error> {
        let (lower, upper) = *self;

        Ok(Automata::from_closure(Box::new(move |c| (lower..=upper).contains(&c))))
    }
}

impl AbstractSyntaxTree for char {
    fn compile(&self) -> Result<Automata, Error> {
        Ok(Automata::from_token(*self))
    }
}
