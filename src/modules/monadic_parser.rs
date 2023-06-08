use std::ops;

type ParserFunction<T> = dyn Fn(& str) -> Option<(T, &str)>;

pub struct MonadicParser<T> {
    fcn: Box<ParserFunction<T>>,
}

impl<T: 'static> MonadicParser<T> {
    pub fn new<F: Fn(& str) -> Option<(T, &str)> + 'static>(f: F) -> Self {
        MonadicParser { fcn: Box::new(f) }
    }

    pub fn parse<'a>(&self, expr: &'a str) -> Option<(T, &'a str)> {
        (self.fcn)(expr)
    }

    pub fn chain<U: 'static>(self, other: MonadicParser<U>) -> MonadicParser<(T, U)> {
        MonadicParser::new(move |expr| {
            let (t, rst_t) = self.parse(expr)?;
            let (u, rst_u) = other.parse(rst_t)?;

            Some(((t, u), rst_u))
        })
    }

    pub fn map<U: 'static, F: Fn(T) -> Option<U> + 'static>(self, transform: F) -> MonadicParser<U> {
        MonadicParser::new(move |expr| {
            let (t, rst) = self.parse(expr)?;
            Some((transform(t)?, rst))
        })
    }

    pub fn filter<F: Fn(&T) -> bool + 'static>(self, predicate: F) -> Self {
        self.map(move |t| if predicate(&t) { Some(t) } else { None })
    }

    pub fn exclude<F: Fn(&T) -> bool + 'static>(self, predicate: F) -> Self {
        self.filter(move |t| !predicate(t))
    }

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

    pub fn one_or_more(self) -> MonadicParser<Vec<T>> {
        self.repeat().exclude(Vec::is_empty)
    }

    pub fn optional(self) -> MonadicParser<Option<T>> {
        MonadicParser::new(move |expr| {
            if let Some((t, rst)) = self.parse(expr) {
                Some((Some(t), rst))
            } else {
                Some((None, expr))
            }
        })
    }

    pub fn lazy<F: Fn() -> MonadicParser<T> + 'static>(closure: F) -> Self {
        MonadicParser::new(move |expr| closure().parse(expr))
    }
}

impl<T: 'static, U: 'static> ops::BitAnd<MonadicParser<U>> for MonadicParser<T> {
    type Output = MonadicParser<(T, U)>;

    fn bitand(self, rhs: MonadicParser<U>) -> Self::Output {
        self.chain(rhs)
    }
}

impl<T: 'static, U: 'static> ops::Shl<MonadicParser<U>> for MonadicParser<T> {
    type Output = MonadicParser<T>;

    fn shl(self, rhs: MonadicParser<U>) -> Self::Output {
        self.chain(rhs).map(|(t, _)| Some(t))
    }
}

impl<T: 'static, U: 'static> ops::Shr<MonadicParser<U>> for MonadicParser<T> {
    type Output = MonadicParser<U>;

    fn shr(self, rhs: MonadicParser<U>) -> Self::Output {
        self.chain(rhs).map(|(_, u)| Some(u))
    }
}

#[macro_export]
macro_rules! union {
    ( $($x:expr),+ $(,)? ) => {
        {
            let parsers = vec![$($x),+];

            MonadicParser::new(move |expr| parsers.iter().find_map(|parser| parser.parse(expr)))
        }
    };
}
