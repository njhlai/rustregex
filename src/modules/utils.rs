const CONCAT_CHAR: char = 27 as char;

pub fn to_postfix(expr: &str) -> String {
    let mut postfix = String::new();
    let mut operator_stack = Vec::<char>::new();
    let precedence = |c| match c {
        '|' => 0,
        CONCAT_CHAR => 1,
        _ => 2,
    };

    for c in expr.chars() {
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

    for c in operator_stack {
        postfix.push(c);
    }

    postfix
}