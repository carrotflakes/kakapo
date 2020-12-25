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
                let ast = parse_or(cs)?;
                if let Some(')') = cs.peek() {
                    cs.next();
                } else {
                    return Err(Error::UnexpectedChar(cs.peek().unwrap().clone()));
                }
                ast
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
            Some(_) | None => ast,
        })
    } else {
        Err(Error::UnexpectedEOS)
    }
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
