use super::automata::Automata;
use super::error::Error;
use super::grammar::{
    Anchor, BasicExpression, CharacterClass, Expression, Match, Quantifiable, Quantified, Quantifier, SubExpression,
};
use super::state::Anchor as StateAnchor;

pub trait AbstractSyntaxTree {
    fn compile(&self) -> Result<Automata, Error>;
}

impl AbstractSyntaxTree for Expression {
    fn compile(&self) -> Result<Automata, Error> {
        let mut it = self.iter();
        let initial_sub_expr = it
            .next()
            .ok_or_else(|| Error::from("Internal error: No SubExpression in Regex"))?;

        it.fold(initial_sub_expr.compile(), |acc, basic_expr| Ok(acc?.or(basic_expr.compile()?)))
    }
}

impl AbstractSyntaxTree for SubExpression {
    fn compile(&self) -> Result<Automata, Error> {
        let mut it = self.iter();
        let initial_basic_expr = it
            .next()
            .ok_or_else(|| Error::from("Internal error: No BasicExpression in SubExpression"))?;

        it.fold(initial_basic_expr.compile(), |acc, basic_expr| Ok(acc?.concat(basic_expr.compile()?)))
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
        match self {
            Anchor::Start => Ok(Automata::from_anchor(StateAnchor::Start)),
            Anchor::End => Ok(Automata::from_anchor(StateAnchor::End)),
            Anchor::WordBoundary => Ok(Automata::from_anchor(StateAnchor::WordBoundary)),
            Anchor::NotWordBoundary => todo!(),
        }
    }
}

impl AbstractSyntaxTree for Quantified {
    fn compile(&self) -> Result<Automata, Error> {
        let (quantifiable, maybe_quantifier) = self;
        let automata = quantifiable.compile();

        if let Some(quantifier) = maybe_quantifier {
            match quantifier {
                Quantifier::ZeroOrMore => Ok(automata?.closure()),
                Quantifier::OneOrMore => Ok(automata?.plus()),
                Quantifier::ZeroOrOne => Ok(automata?.optional()),
                Quantifier::Range((lower, maybe_upper)) => (0..(*lower).saturating_sub(1))
                    .fold(automata, |acc, _| Ok(acc?.concat(quantifiable.compile()?)))
                    .and_then(|auto_lower| {
                        if let Some(upper) = maybe_upper {
                            (0..(*upper).saturating_sub(*lower))
                                .fold(Ok(auto_lower), |acc, _| Ok(acc?.concat(quantifiable.compile()?.optional())))
                        } else {
                            Ok(auto_lower.plus())
                        }
                    }),
            }
        } else {
            automata
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

impl AbstractSyntaxTree for Match {
    fn compile(&self) -> Result<Automata, Error> {
        match self {
            Match::Any => Ok(Automata::from_lambda(|_| true)),
            Match::CharacterClass(cc) => cc.compile(),
            Match::CharacterGroup(_) => todo!(),
            Match::Char(c) => Ok(Automata::from_token(*c)),
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
