use log::{debug, info};

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Plus, Minus, Left, Right, Out, In, LBr, RBr
}

impl Token {
    pub fn is_brace(&self) -> bool {
        use Token::*;
        match self {
            LBr | RBr => true,
            _ => false
        }
    }

    pub fn from_char(c: char) -> Option<Token> {
        use Token::*;
        let ret = match c {
            '+' => Some(Plus),
            '-' => Some(Minus),
            '>' => Some(Right),
            '<' => Some(Left),
            '[' => Some(LBr),
            ']' => Some(RBr),
            ',' => Some(In),
            '.' => Some(Out),
            _   => None
        };

        if let Some(t) = &ret {
            debug!("lexed {:?}", t);
        };
        
        ret
    }

}

pub fn parse(program: &String) -> Vec<Token> {
    let ret = program.chars()
        .map(Token::from_char)
        .filter(Option::is_some)
        .map(Option::unwrap)
        .collect();

    debug!("Parsed: {:?}", ret);
    ret
}
