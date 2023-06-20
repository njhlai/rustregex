use std::ops;

/// A function which parses a [`prim@str`] expression into an `Option<(T, &str)>` type.
type ParserFunction<T> = dyn Fn(&str) -> Option<(T, &str)>;

/// A structure to define a parser as a monad of a formal grammar, i.e. a monoid in the category of endofunctors of a formal grammar.
///
/// Here, we view a parser as an endofunctor of a formal (context free) grammar, viewed as a category over its alphabet set.
///
/// Viewing parsers as a monad allows us to apply functional programming paradigm to these parsers and perform calculus on them.
pub struct MonadicParser<T> {
    fcn: Box<ParserFunction<T>>,
}

impl<T: 'static> MonadicParser<T> {
    /// Returns a new [`MonadicParser`] with parsing function `F`.
    pub fn new<F: Fn(&str) -> Option<(T, &str)> + 'static>(f: F) -> Self {
        MonadicParser { fcn: Box::new(f) }
    }

    /// Parses `expr` into an `Option<(T, &str)>` type.
    pub fn parse<'a>(&self, expr: &'a str) -> Option<(T, &'a str)> {
        (self.fcn)(expr)
    }

    /// Takes two [`MonadicParser`] and creates a new [`MonadicParser`] which applies the parsing of both in sequence.
    pub fn chain<U: 'static>(self, other: MonadicParser<U>) -> MonadicParser<(T, U)> {
        MonadicParser::new(move |expr| {
            let (t, rst_t) = self.parse(expr)?;
            let (u, rst_u) = other.parse(rst_t)?;

            Some(((t, u), rst_u))
        })
    }

    /// Creates a [`MonadicParser`] which applies the closure `transform` on the result of `parse`.
    pub fn map<U: 'static, F: Fn(T) -> Option<U> + 'static>(self, transform: F) -> MonadicParser<U> {
        MonadicParser::new(move |expr| {
            let (t, rst) = self.parse(expr)?;
            Some((transform(t)?, rst))
        })
    }

    /// Creates a [`MonadicParser`] which yields the result of `parse` if the result satisfies `predicate`.
    pub fn filter<F: Fn(&T) -> bool + 'static>(self, predicate: F) -> Self {
        self.map(move |t| if predicate(&t) { Some(t) } else { None })
    }

    /// Creates a [`MonadicParser`] which yields the result of `parse` if the result does not satisfy `predicate`.
    pub fn exclude<F: Fn(&T) -> bool + 'static>(self, predicate: F) -> Self {
        self.filter(move |t| !predicate(t))
    }

    /// Creates a [`MonadicParser`] which repeatedly and greedily applies `parse` on the given expression.
    pub fn repeat(self) -> MonadicParser<Vec<T>> {
        MonadicParser::new(move |expr| {
            let mut current_expr = expr;
            let mut res = vec![];
            while let Some((t, rst)) = self.parse(current_expr) {
                res.push(t);
                current_expr = rst;
            }

            Some((res, current_expr))
        })
    }

    /// Creates a [`MonadicParser`] which repeatedly and greedily applies `parse` on the given expression, and returns the result on if `parse` yields a result at least once.
    pub fn one_or_more(self) -> MonadicParser<Vec<T>> {
        self.repeat().exclude(Vec::is_empty)
    }

    /// Creates a [`MonadicParser`] which optionally applies `parse` on the given expression.
    pub fn optional(self) -> MonadicParser<Option<T>> {
        MonadicParser::new(
            move |expr| {
                if let Some((t, rst)) = self.parse(expr) {
                    Some((Some(t), rst))
                } else {
                    Some((None, expr))
                }
            },
        )
    }

    /// Creates a [`MonadicParser`] which optionally applies `parse` on the given expression and returns `True` if the result of `parse` is a `(Some(T), ...)` value.
    pub fn exists(self) -> MonadicParser<bool> {
        self.optional().map(|x| Some(x.is_some()))
    }

    /// Creates a wrapper [`MonadicParser`] which calls the resulting [`MonadicParser`] from `closure` lazily, i.e. on parsing an expression.
    pub fn lazy<F: Fn() -> MonadicParser<T> + 'static>(closure: F) -> Self {
        MonadicParser::new(move |expr| closure().parse(expr))
    }
}

impl<T: 'static, U: 'static> ops::BitAnd<MonadicParser<U>> for MonadicParser<T> {
    type Output = MonadicParser<(T, U)>;

    /// Performs the `chain` operation, keeping both sides' output.
    fn bitand(self, rhs: MonadicParser<U>) -> Self::Output {
        self.chain(rhs)
    }
}

impl<T: 'static, U: 'static> ops::Shl<MonadicParser<U>> for MonadicParser<T> {
    type Output = MonadicParser<T>;

    /// Performs the `chain` operation, and reduces to the left side's output.
    fn shl(self, rhs: MonadicParser<U>) -> Self::Output {
        self.chain(rhs).map(|(t, _)| Some(t))
    }
}

impl<T: 'static, U: 'static> ops::Shr<MonadicParser<U>> for MonadicParser<T> {
    type Output = MonadicParser<U>;

    /// Performs the `chain` operation, and reduces to the right side's output.
    fn shr(self, rhs: MonadicParser<U>) -> Self::Output {
        self.chain(rhs).map(|(_, u)| Some(u))
    }
}

/// Creates a [`MonadicParser`] which applies `parse` of each member [`MonadicParser`] and returns the first non-`None` result.
#[macro_export]
macro_rules! union {
    ( $($x:expr),+ $(,)? ) => {
        {
            let parsers = vec![$($x),+];

            MonadicParser::new(move |expr| parsers.iter().find_map(|parser| parser.parse(expr)))
        }
    };
}
