use log::debug;

#[derive(PartialEq, Eq)]
pub enum Op {
    Modify(i16),
    Move(i16),
    Outp(u16),
    Inp(u16),
    LBr,
    RBr,
}

impl std::fmt::Debug for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Modify(x) => write!(f, "(mod {x})"),
            Op::Move(x) => write!(f, "(mov {x})"),
            Op::Outp(x) => write!(f, "(out {x})"),
            Op::Inp(x) => write!(f, "(inp {x})"),
            Op::LBr => write!(f, "( [ )"),
            Op::RBr => write!(f, "( ] )"),
        }
    }
}

impl Op {
    fn is_reducible(&self) -> bool {
        use Op::*;

        match self {
            Modify(_) => true,
            Move(_) => true,
            _ => false,
        }
    }

    fn is_same_operation(&self, other: &Op) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    pub fn get_val(&self) -> i16 {
        use Op::*;
        match self {
            Modify(x) => *x,
            Move(x) => *x,
            Outp(x) => *x as i16,
            Inp(x) => *x as i16,
            LBr => 1,
            RBr => 1,
            _ => unreachable!(),
        }
    }

    pub fn is_brace(&self) -> bool {
        use Op::*;
        match self {
            LBr | RBr => true,
            _ => false,
        }
    }

    pub fn from_char(c: char) -> Option<Op> {
        use Op::*;
        match c {
            '+' => Some(Modify(1)),
            '-' => Some(Modify(-1)),
            '>' => Some(Move(1)),
            '<' => Some(Move(-1)),
            '[' => Some(LBr),
            ']' => Some(RBr),
            ',' => Some(Inp(1)),
            '.' => Some(Outp(1)),
            _ => None,
        }
    }
}

fn squash(mut vec: Vec<Op>, next_token: Op) -> Vec<Op> {
    use Op::*;

    let Some(token) = vec.last_mut() else {
        vec.push(next_token);
        return vec;
    };

    if !token.is_reducible() {
        vec.push(next_token);
        return vec;
    }

    if !token.is_same_operation(&next_token) {
        vec.push(next_token);
        return vec;
    }

    let update = next_token.get_val();

    *token = match token {
        Move(x) => Move(*x + update),
        Modify(x) => Modify(*x + update),
        Outp(x) => Outp(*x + update as u16),
        Inp(x) => Inp(*x + update as u16),
        _ => unreachable!(),
    };

    vec
}

pub fn parse(program: &String) -> Vec<Op> {
    let unflattened = program
        .chars()
        .filter_map(Op::from_char)
        .fold(Vec::new(), squash);

    debug!("Parsed: {:?}", unflattened);
    println!("{unflattened:?}");
    unflattened
}
