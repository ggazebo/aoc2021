use std::io;
use std::io::{BufRead};

mod alu;
use alu::*;

fn read_instructions() -> Vec<Instruction> {
    let mut instructions = Vec::with_capacity(100);

    let stdin = io::stdin();

    for l in stdin.lock().lines() {
        let s = l.unwrap();
        instructions.push(Instruction::try_from(&s).unwrap());
    }

    instructions
}

fn main() {
    let instructions = read_instructions();

    let input = [1, 3, 5, 7, 9, 2, 4, 6, 8, 9, 9, 9, 9, 9];

    let mut alu = Alu::new();
    alu.execute(instructions, input);

    println!("{:?}", &alu);
}
