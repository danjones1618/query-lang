use query_lang::parse_query_string;

fn main() {
    let to_parse = r#"(name     = "hi ehllo * there _ yaes" or blah =~ "7*") and location = "eu-1" and (yes != "okay" or yes = "hmm nice")"#;
    match parse_query_string(&to_parse) {
        Ok(e) => println!("{e:?}"),
        Err(e) => println!("{e}"),
    }
}
