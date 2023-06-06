type ParserFunction<T> = dyn Fn(& str) -> Option<(T, &str)>;

pub struct MonadicParser<T> {
    fcn: Box<ParserFunction<T>>,
}

impl<T: 'static> MonadicParser<T> {
    pub fn new<F: Fn(& str) -> Option<(T, &str)> + 'static>(f: F) -> Self {
        MonadicParser::unit(f)
    }

    fn unit<F: Fn(& str) -> Option<(T, &str)> + 'static>(f: F) -> Self {
        MonadicParser { fcn: Box::new(f) }
    }

    pub fn parse<'a>(&self, expr: &'a str) -> Option<(T, &'a str)> {
        (self.fcn)(expr)
    }

    pub fn chain<U: 'static>(self, other: MonadicParser<U>) -> MonadicParser<(T, U)> {
        MonadicParser::unit(move |expr| {
            let (t, rst_t) = self.parse(expr)?;
            let (u, rst_u) = other.parse(rst_t)?;

            Some(((t, u), rst_u))
        })
    }

    pub fn map<U: 'static, F: Fn(T) -> Option<U> + 'static>(self, transform: F) -> MonadicParser<U> {
        MonadicParser::unit(move |expr| {
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
        MonadicParser::unit(move |expr| {
            let mut current_expr = expr;
            let mut res = vec![];
            while let Some((t, rst)) = self.parse(current_expr) {
                res.push(t);
                current_expr = rst;
                // *expr = expr.split_off(1);
                // println!("after: {expr}");
            }

            Some((res, current_expr))
        })
    }

    pub fn optional(self) -> MonadicParser<Option<T>> {
        MonadicParser::unit(move |expr| {
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

#[macro_export]
macro_rules! union {
    ( $($x:expr),+ $(,)? ) => {
        {
            let parsers = vec![$($x),+];

            MonadicParser::new(move |expr| parsers.iter().find_map(|parser| parser.parse(expr)))
        }
    };
}
