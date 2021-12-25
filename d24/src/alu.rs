use std::fmt;
use std::ops::{AddAssign, DivAssign, Index, IndexMut, RemAssign, MulAssign};

pub type Word = i64;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    Op1(Op1, RegisterId),
    Op2(Op2, RegisterId, Operand),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Op1 {
    Input
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Op2 {
    Add,
    Mul,
    Div,
    Mod,
    Eql,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RegisterId {
    X,
    Y,
    Z,
    W,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    Register(RegisterId),
    Literal(Word),
}

#[derive(Clone, PartialEq, Eq)]
pub struct Register(Word);
impl Register {
    pub fn value(&self) -> Word { self.0 }

    pub fn set(&mut self, v: Word) {
        self.0 = v
    }
}
impl From<Word> for Register {
    fn from(r: Word) -> Self { Register(r) }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Alu {
    x: Register,
    y: Register,
    z: Register,
    w: Register,
}

impl AddAssign<Word> for Register {
    fn add_assign(&mut self, lhs: Word) { self.0 = self.0 + lhs }
}
impl DivAssign<Word> for Register {
    fn div_assign(&mut self, lhs: Word) { self.0 = self.0 / lhs }
}
impl MulAssign<Word> for Register {
    fn mul_assign(&mut self, lhs: Word) { self.0 = self.0 * lhs }
}
impl RemAssign<Word> for Register {
    fn rem_assign(&mut self, lhs: Word) { self.0 = self.0 % lhs }
}

impl Alu {
    pub fn new() -> Alu {
        Alu { x: 0.into(), y: 0.into(), z: 0.into(), w: 0.into() }
    }

    pub fn initialized(x: Word, y: Word, z: Word, w: Word) -> Alu {
        Alu { x: x.into(), y: y.into(), z: z.into(), w: w.into() }
    }

    pub fn execute<'a, 'b>(
        &mut self,
        instructions: impl Iterator<Item = &'a Instruction>,
        inputs: impl IntoIterator<Item = &'b Word>
    ) -> (Word, Word, Word, Word) {
        let mut inputs = inputs.into_iter();
        for i in instructions {
            match i {
                Instruction::Op1(Op1::Input, r) => self[*r].set(*inputs.next().unwrap()),
                Instruction::Op2(op, r, l) => self.execute_op2(*op, *r, *l),
                _ => panic!("Attempted to execute invalid instruction"),
            }
        }
        (self.x.value(), self.y.value(), self.z.value(), self.w.value())
    }

    pub fn execute_op2(&mut self, op: Op2, reg: RegisterId, operand: Operand) {
        let v = match operand {
            Operand::Literal(n) => n,
            Operand::Register(r) => self[r].value(),
        };
        let r0 = &mut self[reg];
        match op {
            Op2::Add => *r0 += v,
            Op2::Div => *r0 /= v,
            Op2::Mod => *r0 %= v,
            Op2::Mul => *r0 *= v,
            Op2::Eql => r0.set(if r0.value() == v { 1 } else { 0 }),
            _ => panic!()
        }
    }
}
impl Index<RegisterId> for Alu {
    type Output = Register;

    fn index(&self, r: RegisterId) -> &Self::Output {
        match r {
            RegisterId::X => &self.x,
            RegisterId::Y => &self.y,
            RegisterId::Z => &self.z,
            RegisterId::W => &self.w,
        }
    }
}
impl IndexMut<RegisterId> for Alu {
    fn index_mut(&mut self, r: RegisterId) -> &mut Self::Output {
        match r {
            RegisterId::X => &mut self.x,
            RegisterId::Y => &mut self.y,
            RegisterId::Z => &mut self.z,
            RegisterId::W => &mut self.w,
        }
    }
}

impl fmt::Debug for Alu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ x={} y={} z={} w={} }}", self.x.value(), self.y.value(), self.z.value(), self.w.value())
    }
}

pub type ParseErr = &'static str;

impl TryFrom<u8> for RegisterId {
    type Error = ParseErr;
    fn try_from(c: u8) -> Result<Self, Self::Error> {
        match c {
            b'x' => Ok(RegisterId::X),
            b'y' => Ok(RegisterId::Y),
            b'z' => Ok(RegisterId::Z),
            b'w' => Ok(RegisterId::W),
            _ => Err("Unknown register ID"),
        }
    }
}

impl Operand {
    fn try_from(s: &str) -> Result<Self, ParseErr> {
        if s.len() == 1 {
            match RegisterId::try_from(s.as_bytes()[0]) {
                Ok(r) => return Ok(Operand::Register(r)),
                Err(_) => (),
            };
        }

        return Ok(Operand::Literal(s.parse::<Word>().map_err(|_| "Cold not parse operand")?))
    }
}

impl TryFrom<&str> for Instruction
{
    type Error = ParseErr;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let bs = s.as_bytes();
        let args_start = bs.iter().position(|&b| b == b' ').ok_or("Invalid instruction string")? + 1;
        let instr = &bs[0..args_start-1];
        let reg = bs[args_start].try_into()?;

        Ok(
            if instr == b"inp" {
                Instruction::Op1(Op1::Input, reg)
            } else {
                let op = Operand::try_from(&s[args_start + 2..])?;
                match instr {
                    b"add" => Instruction::Op2(Op2::Add, reg, op),
                    b"mul" => Instruction::Op2(Op2::Mul, reg, op),
                    b"div" => Instruction::Op2(Op2::Div, reg, op),
                    b"mod" => Instruction::Op2(Op2::Mod, reg, op),
                    b"eql" => Instruction::Op2(Op2::Eql, reg, op),
                    _ => return Err("Unknown instruction"),
                }
            }
        )
    }
}
impl TryFrom<&String> for Instruction
{
    type Error = &'static str;
    fn try_from(s: &String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negate() {
        let instr = [
            Instruction::Op1(Op1::Input, RegisterId::X),
            Instruction::Op2(Op2::Mul, RegisterId::X, Operand::Literal(-1))
        ];

        let mut alu = Alu::new();
        let (x, ..) = alu.execute(instr.iter(), [5].iter());

        assert_eq!(-5, x);
    }
}