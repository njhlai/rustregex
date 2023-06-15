mod modules;
use modules::language;
use modules::regexp::RegExp;

use std::io::{self, Write};

fn main() {
    let expr = r"^@\w+\.\b\{\w{2,3}(.\w{2,})?|c?[a-z\w\s]+\1";
    let regex = language::regex();
    println!("{:#?}", regex.syntax(expr).unwrap());

    // something to get rid of unused code warnings
    loop {
        let regexp_input = get_user_input("regular expression: ");
        if regexp_input.is_empty() {
            break;
        }
        println!("{:#?}", regex.syntax(&regexp_input).unwrap());

        match RegExp::new(&regexp_input) {
            Ok(regexp) => {
                let s = get_user_input("match against:      ");
                println!("full match:         {}", regexp.full_match(&s));
                println!("greedy search:      {:?}", regexp.greedy_search(&s));

                let res = regexp.global_search(&s);
                if res.is_empty() {
                    println!("global search:      yielded no results");
                } else {
                    println!("global search:      yielded {} results -> \"{}\"", res.len(), res.join("\",\""));
                }
            }
            Err(err) => println!("{err:#?}"),
        }
        println!(); // empty line
    }
}

fn get_user_input(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().unwrap(); // just panic if there is an error ¯\_(ツ)_/¯

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("failed to get user input from stdin");
    input = input.trim_end_matches(&['\r', '\n']).to_string();

    if input != input.trim() {
        println!("WARNING: input starts or ends with whitespace");
    }

    input
}
