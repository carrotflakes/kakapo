use kakapo::Regex;

fn main() {
    println!("{:?}", Regex::new("abc").unwrap().r#match("abc"));
    println!("{:?}", Regex::new("abc").unwrap().r#match("abcd"));
    println!("{:?}", Regex::new("a*").unwrap().r#match("ab"));
    println!("{:?}", Regex::new("a*").unwrap().r#match(""));
    println!("{:?}", Regex::new("a*").unwrap().r#match("a"));
    println!("{:?}", Regex::new("a*").unwrap().r#match("aa"));
    println!("{:?}", Regex::new("a+").unwrap().r#match(""));
    println!("{:?}", Regex::new("a+").unwrap().r#match("a"));
    println!("{:?}", Regex::new("a+").unwrap().r#match("aa"));
    println!("{:?}", Regex::new("a?").unwrap().r#match(""));
    println!("{:?}", Regex::new("a?").unwrap().r#match("a"));
    println!("{:?}", Regex::new("a?").unwrap().r#match("aa"));
    println!("{:?}", Regex::new("(a|b)+(c|d)+").unwrap().r#match("aabadddc"));
    println!("{:?}", Regex::new("(a|b)+(c|d)+").unwrap().r#match("aabcadddc"));
    println!("{:?}", Regex::new(".").unwrap().r#match(""));
    println!("{:?}", Regex::new(".").unwrap().r#match("a"));
    println!("{:?}", Regex::new(".").unwrap().r#match("aa"));
    println!("{:?}", Regex::new("[a-c]").unwrap().r#match("a"));
    println!("{:?}", Regex::new("[a-c]").unwrap().r#match("b"));
    println!("{:?}", Regex::new("[a-c]").unwrap().r#match("c"));
    println!("{:?}", Regex::new("[a-c]").unwrap().r#match("d"));
}
