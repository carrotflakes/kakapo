use std::{iter::Peekable, str::Chars};

mod parse;

#[derive(Debug, Clone)]
pub enum Ast {
    Char(char),
    Repeat(u32, u32, Box<Ast>),
    Concat(Vec<Ast>),
    Or(Vec<Ast>),
}

impl Ast {
    pub fn r#match(&self, chars: &mut Peekable<Chars>) -> bool {
        match self {
            Ast::Char(c) => chars.next() == Some(*c),
            Ast::Repeat(min, max, ast) => {
                let mut chars_prev = chars.clone();
                for i in 0..*max {
                    if !ast.r#match(chars) {
                        return *min <= i;
                    }
                    chars_prev = chars.clone();
                }
                *chars = chars_prev;
                true
            }
            Ast::Concat(asts) => {
                for ast in asts {
                    if !ast.r#match(chars) {
                        return false;
                    }
                }
                true
            }
            Ast::Or(asts) => {
                for ast in asts {
                    let mut chars_ = chars.clone();
                    if ast.r#match(&mut chars_) {
                        *chars = chars_;
                        return true;
                    }
                }
                false
            }
        }
    }
}

pub struct Regex(Ast);

impl Regex {
    pub fn new(str: &str) -> Result<Self, parse::Error> {
        Ok(Regex(parse::parse(str)?))
    }

    pub fn r#match(&self, str: &str) -> bool {
        let mut chars = str.chars().peekable();
        let res = self.0.r#match(&mut chars);
        chars.peek() == None && res
    }
}
