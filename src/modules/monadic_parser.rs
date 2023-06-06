type ParserFunction<T> = dyn Fn(&mut String) -> Option<T>;

pub struct MonadicParser<T> {
    fcn: Box<ParserFunction<T>>,
}

impl<T: 'static> MonadicParser<T> {
    pub fn new<F: Fn(&mut String) -> Option<T> + 'static>(f: F) -> Self {
        MonadicParser::unit(f)
    }

    fn unit<F: Fn(&mut String) -> Option<T> + 'static>(f: F) -> Self {
        MonadicParser { fcn: Box::new(f) }
    }

    pub fn parse(&self, expr: &mut String) -> Option<T> {
        (self.fcn)(expr)
    }

    pub fn chain<U: 'static>(self, other: MonadicParser<U>) -> MonadicParser<(T, U)> {
        MonadicParser::unit(move |expr| {
            let t = self.parse(expr)?;
            let u = other.parse(expr)?;

            Some((t, u))
        })
    }

    pub fn map<U: 'static, F: Fn(T) -> Option<U> + 'static>(self, transform: F) -> MonadicParser<U> {
        MonadicParser::unit(move |expr| transform(self.parse(expr)?))
    }

    pub fn filter<F: Fn(&T) -> bool + 'static>(self, predicate: F) -> Self {
        self.map(move |t| if predicate(&t) { Some(t) } else { None })
    }

    pub fn exclude<F: Fn(&T) -> bool + 'static>(self, predicate: F) -> Self {
        self.filter(move |t| !predicate(t))
    }

    pub fn one(self) -> Self {
        MonadicParser::unit(move |expr| {
            if let Some(res) = self.parse(expr) {
                *expr = expr.split_off(1);

                Some(res)
            } else {
                None
            }
        })
    }

    pub fn repeat(self) -> MonadicParser<Vec<T>> {
        MonadicParser::unit(move |expr| {
            let mut res = vec![];
            while let Some(t) = self.parse(expr) {
                res.push(t);
                // *expr = expr.split_off(1);
                // println!("after: {expr}");
            }

            Some(res)
        })
    }

    pub fn optional(self) -> MonadicParser<Option<T>> {
        MonadicParser::unit(move |expr| {
            let prev = expr.clone();

            if let Some(t) = self.parse(expr) {
                Some(Some(t))
            } else {
                *expr = prev;
                Some(None)
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
