use super::grammar::{BasicExpression, Group, Quantifiable, Regex};

/// A trait that allows a struct's fields to be injected as grammatical contexts.
pub trait Context<T> {
    /// Processes type with [`self`]-defined grammatical contexts.
    fn process(&mut self, t: T) -> T;
}

/// A structure that holds Regex's [`Group`]-related contexts.
pub struct RegexContext(usize);

impl RegexContext {
    /// Initialise an instance of [`RegexContext`].
    pub fn init() -> Self {
        RegexContext(0)
    }
}

impl Context<Regex> for RegexContext {
    fn process(&mut self, regex: Regex) -> Regex {
        regex
            .into_iter()
            .map(|subexpr| {
                subexpr
                    .into_iter()
                    .map(|basic| match basic {
                        BasicExpression::Quantified(q) => {
                            let (quantifiable, quantifier) = q;

                            let modified_quantiable = match quantifiable {
                                Quantifiable::Group(g) => {
                                    self.0 += 1;

                                    Quantifiable::Group(Group { non_capturing: g.non_capturing, index: self.0, expr: g.expr })
                                }
                                _ => quantifiable,
                            };

                            BasicExpression::Quantified((modified_quantiable, quantifier))
                        }
                        BasicExpression::Anchor(_) => basic,
                    })
                    .collect()
            })
            .collect()
    }
}
