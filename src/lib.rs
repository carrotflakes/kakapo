use std::{iter::Peekable, str::Chars};

// mod vm;
mod parse;

#[derive(Debug, Clone)]
pub enum Ast {
    Char(char),
    Range(char, char),
    Any,
    Not(Box<Ast>),
    Repeat(u32, u32, Box<Ast>),
    Concat(Vec<Ast>),
    Or(Vec<Ast>),
    Capture(Box<Ast>),
}

impl Ast {
    pub fn r#match(&self, chars: &mut Peekable<Chars>) -> bool {
        match self {
            Ast::Char(c) => chars.next() == Some(*c),
            Ast::Range(c1, c2) => {
                if let Some(c) = chars.next() {
                    *c1 <= c && c <= *c2
                } else {
                    false
                }
            }
            Ast::Any => {
                if let Some(_) = chars.next() {
                    true
                } else {
                    false
                }
            }
            Ast::Not(ast) => {
                !ast.r#match(&mut chars.clone())
            }
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
            Ast::Capture(_) => {todo!()}
        }
    }
}

pub struct Runtime<'a> {
    pub chars: Chars<'a>,
    _captured: Vec<(*const Ast, &'a str, &'a str)>,
}

impl<'a> Runtime<'a> {
    pub fn new(chars: Chars<'a>) -> Self {
        Runtime {
            chars,
            _captured: Vec::new(),
        }
    }

    fn next(&mut self) -> Option<char> {
        self.chars.next()
    }

    pub fn run(&mut self, ast: &Ast) -> bool {
        match ast {
            Ast::Char(c) => self.next() == Some(*c),
            Ast::Range(c1, c2) => {
                if let Some(c) = self.next() {
                    *c1 <= c && c <= *c2
                } else {
                    false
                }
            }
            Ast::Any => {
                if let Some(_) = self.next() {
                    true
                } else {
                    false
                }
            }
            Ast::Not(ast) => {
                !self.child().run(ast)
            }
            Ast::Repeat(min, max, ast) => {
                let mut chars_prev = self.chars.clone();
                for i in 0..*max {
                    if !self.run(ast) {
                        return *min <= i;
                    }
                    chars_prev = self.chars.clone();
                }
                self.chars = chars_prev;
                true
            }
            Ast::Concat(asts) => {
                for ast in asts {
                    if !self.run(ast) {
                        return false;
                    }
                }
                true
            }
            Ast::Or(asts) => {
                let chars_ = self.chars.clone();
                for ast in asts {
                    if self.run(ast) {
                        return true;
                    } else {
                        self.chars = chars_.clone();
                    }
                }
                false
            }
            Ast::Capture(ast) => {
                let begin = self.chars.as_str();
                if self.run(ast) {
                    let end = self.chars.as_str();
                    self._captured.push((ast.as_ref() as *const Ast, begin, end));
                    true
                } else {
                    false
                }
            }
        }
    }

    fn child(&self) -> Self {
        Runtime {
            chars: self.chars.clone(),
            _captured: Vec::new(),
        }
    }

    fn captured(&self) -> Vec<String> {
        self._captured.iter().map(|(_, b, e)| {
            b[..b.len() - e.len()].to_string()
        }).collect()
    }
}

pub struct Regex(Ast);

impl Regex {
    pub fn new(str: &str) -> Result<Self, parse::Error> {
        Ok(Regex(parse::parse(str)?))
    }

    pub fn r#match(&self, str: &str) -> bool {
        // let mut chars = str.chars().peekable();
        // let res = self.0.r#match(&mut chars);
        // chars.peek() == None && res

        let mut rt = Runtime::new(str.chars());
        let res = rt.run(&self.0);
        let ret = rt.chars.next() == None && res;
        if ret {
            dbg!(rt.captured());
        }
        ret
    }
}
