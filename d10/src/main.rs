use std::fmt;
use std::io;
use std::io::BufRead;
use std::iter::Iterator;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Bracket {
    Paren,
    Square,
    Brace,
    Angle,
    Any,
}

impl Bracket {
    pub fn score(&self) -> usize {
        match self {
            Bracket::Paren => 3,
            Bracket::Square => 57,
            Bracket::Brace => 1197,
            Bracket::Angle => 25137,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum BracketType {
    Open,
    Close,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Token {
    b: Bracket,
    t: BracketType,
}

const TOK_OPEN_PAREN: Token = Token::open(Bracket::Paren);
const TOK_CLOSE_PAREN: Token = Token::close(Bracket::Paren);
const TOK_OPEN_SQUARE: Token = Token::open(Bracket::Square);
const TOK_CLOSE_SQUARE: Token = Token::close(Bracket::Square);
const TOK_OPEN_BRACE: Token = Token::open(Bracket::Brace);
const TOK_CLOSE_BRACE: Token = Token::close(Bracket::Brace);
const TOK_OPEN_ANGLE: Token = Token::open(Bracket::Angle);
const TOK_CLOSE_ANGLE: Token = Token::close(Bracket::Angle);

impl Token {
    pub const fn open(b: Bracket) -> Token {
        Token{ b, t: BracketType::Open }
    }

    pub const fn close(b: Bracket) -> Token {
        Token{ b, t: BracketType::Close }
    }

    pub fn from_char(c: char) -> Option<Token> {
        Some(match c {
            '(' => TOK_OPEN_PAREN,
            ')' => TOK_CLOSE_PAREN,
            '[' => TOK_OPEN_SQUARE,
            ']' => TOK_CLOSE_SQUARE,
            '{' => TOK_OPEN_BRACE,
            '}' => TOK_CLOSE_BRACE,
            '<' => TOK_OPEN_ANGLE,
            '>' => TOK_CLOSE_ANGLE,
            _ => return None,
        })
    }

    pub fn is_open(&self) -> bool {
        self.t == BracketType::Open
    }

    pub fn is_close(&self) -> bool {
        self.t == BracketType::Close
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            TOK_OPEN_PAREN => '(',
            TOK_CLOSE_PAREN => ')',
            TOK_OPEN_SQUARE => '[',
            TOK_CLOSE_SQUARE => ']',
            TOK_OPEN_BRACE => '{',
            TOK_CLOSE_BRACE => '}',
            TOK_OPEN_ANGLE => '<',
            TOK_CLOSE_ANGLE => '>',
            _ => '?',
        })
    }
}

pub struct ParseErr {
    expected: Bracket,
    found: Option<Bracket>,
    pos: usize,
}

impl ParseErr {
    pub fn score(&self) -> usize {
        match self.found {
            Some(b) => b.score(),
            _ => 0,
        }
    }
}

pub fn score_completion(l: &Vec<Token>) -> usize {
    l.iter()
        .map(|t| match t.b {
            Bracket::Paren => 1,
            Bracket::Square => 2,
            Bracket::Brace => 3,
            Bracket::Angle => 4,
            _ => 0,
        })
        .fold(0, |total, score| total * 5 + score)
}

pub fn parse_line(s: &String, completion: &mut Vec<Token>) -> Result<(), ParseErr> {
    let mut stack = Vec::with_capacity(10);

    for (i, c) in s.trim_end().chars().enumerate() {
        let token = Token::from_char(c).unwrap();
        if token.is_open() {
            stack.push(token);
        }
        else {
            match stack.pop() {
                Some(opener) if opener.b == token.b => (),
                Some(opener) => return Err(ParseErr{ expected: opener.b, found: Some(token.b), pos: i}),
                None => return Err(ParseErr{ expected: Bracket::Any, found: None, pos: i}),
            }
        }
    }

    for &t in stack.iter().rev() {
        completion.push(Token::close(t.b));
    }

    Ok(())
}

fn main() {
    let stdin = io::stdin();

    let mut errors = Vec::new();
    let mut completions: Vec<Vec<Token>> = Vec::new();
    for l in stdin.lock().lines() {
        let s = l.unwrap();
        println!("{}", s);
        let mut completion = vec!();
        match parse_line(&s, &mut completion) {
            Ok(_) => {
                if !completion.is_empty() {
                    let m = completion.iter().cloned().collect();
                    print!(" PARTIAL: missing ");
                    for t in &m {
                        print!("{}", t);
                    }
                    println!();
                    completions.push(m);
                }
                else {
                    println!(" OK!");
                }
            },
            Err(e) => {
                println!(" {}: Expected {} but found {}",
                    e.pos,
                    Token::close(e.expected),
                    Token::close(e.found.unwrap_or(Bracket::Any)));
                errors.push(e);
            }
        }
    }

    let syntax_score: usize = errors.iter().map(|e| e.score()).sum();
    println!("syntax score: {}", syntax_score);

    let mut completion_scores: Vec<usize> = completions
        .iter()
        .map(|c| score_completion(c))
        .collect();
    completion_scores.sort();
    let middle = completion_scores[completion_scores.len() / 2];
    println!("completion score: {}", middle);
}
