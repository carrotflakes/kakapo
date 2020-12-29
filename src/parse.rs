use crate::Ast;
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub enum Error {
    UnexpectedChar(char),
    UnexpectedParenClose,
    UnexpectedEOS,
}

pub fn parse(str: &str) -> Result<Ast, Error> {
    let mut cs = str.chars().peekable();
    let res = parse_or(&mut cs);
    if let Some(c) = cs.peek() {
        Err(Error::UnexpectedChar(c.clone()))
    } else {
        res
    }
}

fn parse_or(cs: &mut Peekable<impl Iterator<Item = char>>) -> Result<Ast, Error> {
    let mut asts = Vec::new();
    asts.push(parse_concat(cs)?);
    while let Some('|') = cs.peek() {
        cs.next();
        asts.push(parse_concat(cs)?);
    }
    match asts.len() {
        0 => Ok(Ast::Concat(Vec::new())),
        1 => Ok(asts.remove(0)),
        _ => Ok(Ast::Or(asts)),
    }
}

fn parse_concat(cs: &mut Peekable<impl Iterator<Item = char>>) -> Result<Ast, Error> {
    let mut asts = Vec::new();
    while let Some(c) = cs.peek() {
        match c {
            '|' | ')' | '*' | '+' => {
                break;
            }
            _ => {
                asts.push(parse_repeat(cs)?);
            }
        }
    }
    match asts.len() {
        1 => Ok(asts.remove(0)),
        _ => Ok(Ast::Concat(asts)),
    }
}

fn parse_repeat(cs: &mut Peekable<impl Iterator<Item = char>>) -> Result<Ast, Error> {
    if let Some(c) = cs.peek() {
        let ast = match c {
            '|' | ')' | '*' | '+' => {
                return Err(Error::UnexpectedChar(cs.peek().unwrap().clone()));
            }
            '(' => {
                cs.next();
                parse_paren(cs)?
            }
            '.' => {
                cs.next();
                Ast::Any
            }
            '[' => {
                cs.next();
                parse_char_class(cs)?
            }
            _ => parse_char(cs)?,
        };
        Ok(match cs.peek() {
            Some('*') => {
                cs.next();
                Ast::Repeat(0, std::u32::MAX, Box::new(ast))
            }
            Some('+') => {
                cs.next();
                Ast::Repeat(1, std::u32::MAX, Box::new(ast))
            }
            Some('?') => {
                cs.next();
                Ast::Repeat(0, 1, Box::new(ast))
            }
            Some('{') => {
                cs.next();
                let min = if let Ok(n) = parse_number(cs) {
                    n
                } else {
                    0
                };
                expect_char(cs, ',')?;
                let max = if let Ok(n) = parse_number(cs) {
                    n
                } else {
                    std::u32::MAX
                };
                expect_char(cs, '}')?;
                Ast::Repeat(min, max, Box::new(ast))
            }
            Some(_) | None => ast,
        })
    } else {
        Err(Error::UnexpectedEOS)
    }
}

fn parse_paren(cs: &mut Peekable<impl Iterator<Item = char>>) -> Result<Ast, Error> {
    let ast = if cs.peek() == Some(&'?') {
        cs.next();
        match cs.peek() {
            Some(':') => {
                cs.next();
                parse_or(cs)?
            }
            Some('=') => {
                cs.next();
                Ast::Lookahead(Box::new(parse_or(cs)?))
            }
            Some('!') => {
                cs.next();
                Ast::Not(Box::new(parse_or(cs)?))
            }
            Some(c) => {
                return Err(Error::UnexpectedChar(*c));
            }
            None => {
                return Err(Error::UnexpectedEOS);
            }
        }
    } else {
        Ast::Capture(Box::new(parse_or(cs)?))
    };
    if let Some(')') = cs.peek() {
        cs.next();
    } else {
        return Err(Error::UnexpectedChar(cs.peek().unwrap().clone()));
    }
    Ok(ast)
}

fn parse_char_class(cs: &mut Peekable<impl Iterator<Item = char>>) -> Result<Ast, Error> {
    let mut asts = Vec::new();
    let invert = matches!(cs.peek(), Some('^'));
    loop {
        match cs.peek() {
            Some(']') => {
                cs.next();
                break;
            }
            _ => {
                let c = parse_char_(cs)?;
                if let Some('-') = cs.peek() {
                    cs.next();
                    let c2 = parse_char_(cs)?;
                    asts.push(Ast::Range(c, c2));
                } else {
                    asts.push(Ast::Char(c));
                }
            }
        }
    }
    let ast = match asts.len() {
        1 => asts.remove(0),
        _ => Ast::Or(asts),
    };
    if invert {
        Ok(Ast::Concat(vec![Ast::Not(Box::new(ast)), Ast::Any]))
    } else {
        Ok(ast)
    }
}

fn parse_char(cs: &mut Peekable<impl Iterator<Item = char>>) -> Result<Ast, Error> {
    if matches!(cs.peek(), Some('\\')) {
        cs.next();
        return match cs.next() {
            Some('r') => Ok(Ast::Char('\r')),
            Some('n') => Ok(Ast::Char('\n')),
            Some('t') => Ok(Ast::Char('\t')),
            Some('0') => Ok(Ast::Char('\0')),
            Some('d') => {
                Ok(Ast::Range('0', '9'))
            }
            Some(c) => {
                Err(Error::UnexpectedChar(c))
            }
            None => {
                Err(Error::UnexpectedEOS)
            }
        };
    }
    parse_char_(cs).map(|c| Ast::Char(c))
}

fn parse_char_(cs: &mut Peekable<impl Iterator<Item = char>>) -> Result<char, Error> {
    if let Some(c) = cs.peek().cloned() {
        cs.next();
        if c == '\\' {
            let c = match cs.peek() {
                Some('r') => '\r',
                Some('n') => '\n',
                Some('t') => '\t',
                Some('0') => '\0',
                Some(_) | None => {
                    return Err(Error::UnexpectedEOS);
                }
            };
            cs.next();
            Ok(c)
        } else {
            Ok(c.clone())
        }
    } else {
        Err(Error::UnexpectedEOS)
    }
}

fn parse_number(cs: &mut Peekable<impl Iterator<Item = char>>) -> Result<u32, Error> {
    let mut n = 0;
    let mut failed = true;
    while let Some(c) = cs.peek() {
        if let Some(d) = c.to_digit(10) {
            n = d + n * 10;
            failed = false;
        } else {
            if failed {
                return Err(Error::UnexpectedChar(*c));
            }
            break;
        }
        cs.next();
    }
    Ok(n)
}

fn expect_char(cs: &mut Peekable<impl Iterator<Item = char>>, char: char) -> Result<(), Error> {
    match cs.next() {
        Some(c) if c == char => {
            Ok(())
        }
        Some(c) => {
            Err(Error::UnexpectedChar(c))
        }
        None => {
            Err(Error::UnexpectedEOS)
        }
    }
}

#[test]
fn test() {
    dbg!(parse("abc"));
    dbg!(parse("(abc)"));
    dbg!(parse("a|b|c"));
    dbg!(parse("(a|bc)"));
    dbg!(parse("(a|bc)*"));
    dbg!(parse("(a|b+c)"));
    dbg!(parse("(a|bc+)"));
}
