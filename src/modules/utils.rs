pub const CONCAT_CHAR: char = 27 as char;

pub fn to_postfix(expr: &str) -> String {
    let modified_expr = add_concat_char(expr);
    let mut postfix = String::new();
    let mut operator_stack = Vec::<char>::new();
    let precedence = |c| match c {
        '|' => 0,
        CONCAT_CHAR => 1,
        _ => 2,
    };

    for c in modified_expr.chars() {
        match c {
            '|' | CONCAT_CHAR | '*' | '?' | '+' => {
                let curr_precedence = precedence(c);
                while let Some(&op) = operator_stack.last() {
                    if (op == '(') || (precedence(op) < curr_precedence) { break; }
                    operator_stack.pop();
                    postfix.push(op);
                }
                operator_stack.push(c);
            }
            '(' => operator_stack.push(c),
            ')' => {
                while let Some(token) = operator_stack.pop() {
                    if token == '(' { break; }
                    postfix.push(token);
                }
            }
            _ => postfix.push(c),
        }
    }

    while let Some(op) = operator_stack.pop() {
        postfix.push(op);
    }

    postfix
}

pub fn add_concat_char(expr: &str) -> String {
    let mut result = String::new();
    let mut it = expr.chars();
    if let Some(first) = it.next() {
        result.push(first);
        let mut prev = first;

        for c in it {
            if !matches!(prev, '(' | '|') && !matches!(c, ')' | '|' | '*' | '?' | '+') {
                result.push(CONCAT_CHAR);
            }

            result.push(c);
            prev = c;
        }
    }

    result
}
