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

struct DescendingModelNumbers([Word; 14]);
impl DescendingModelNumbers {
    pub fn new() -> Self {
        Self([9; 14])
    }
}
impl From<[Word; 14]> for DescendingModelNumbers {
    fn from(a: [Word; 14]) -> Self {
        Self(a)
    }
}
impl Iterator for DescendingModelNumbers {
    type Item = [Word; 14];

    fn next(&mut self) -> Option<Self::Item> {
        if self.0[0] == 0 {
            return None
        }
        let r = self.0.clone();

        self.0[13] -= 1;
        let a = &mut self.0;
        for i in (1..=13).rev() {
            if a[i] == 0 {
                a[i] = 9;
                a[i-1] -= 1;
            }
        }

        Some(r)
    }
}

fn find_valid(digits: &mut Vec<i64>, sieve: &[Vec<(i64, i64, i64)>]) -> Option<u64> {
    if sieve.len() == 0 {
        let mut n = 0;
        for d in digits {
            n = n * 10 + *d as u64;
        }
        println!("{}", n);
        return Some(n);
    }

    let candidates = &sieve[0];

    for (w, ..) in candidates {
        digits.push(*w);
        match find_valid(digits, &sieve[1..]) {
            Some(n) => return Some(n),
            None => (),
        }
        digits.pop();
    }
    None
}

fn main() {
    let instructions = read_instructions();
    let mut inst_chunks = Vec::with_capacity(14);
    for i in 0..14 {
        inst_chunks.push(&instructions[i*18..i*18+18]);
    }

    let mut z_matches = vec![vec!(); 15];
    z_matches[14].push((9, 0, 0));

    for digit in (0..14).rev() {
        let inst = &inst_chunks[digit];
        for z_init in -20000..=20000 {
            let z_wanted: Vec<i64> = z_matches[digit+1].iter().map(|p| p.1).collect();
            let zs = &mut z_matches[digit];
            for d in (1..=9).rev() {
                let mut alu = Alu::initialized(0,0, z_init, 0);
                let (.., z, _) = alu.execute(inst.iter(), [d].iter());

                if z_wanted.contains(&z) {
                    //println!("{} PASS on z:={} w={} : {:?}", digit+1, z_init, d, &alu);
                    zs.push((d, z_init, z));
                }
            }
        }
    }

    let mut sieve = vec![vec!(); 14];
    //let min_or_max = |(w1, ..): &(i64, i64, i64), (w2, ..): &(i64, i64, i64)| w1.cmp(w2); // min
    let min_or_max = |(w1, ..): &(i64, i64, i64), (w2, ..): &(i64, i64, i64)| w2.cmp(w1); // max

    // Seed solution for first digit
    for (w, z_init, z) in &z_matches[0] {
        if *z_init == 0 {
            sieve[0].push((*w, *z_init, *z));
        }
    }
    // Remove so
    sieve[0].sort_by(min_or_max);
    println!("{:?}", &sieve[0]);

    for digit in 1..14 {
        let allowed_zs: Vec<Word> = sieve[digit-1].iter().map(|(_, _, z)| *z).collect();
        for (w, z_init, z) in &z_matches[digit] {
            if allowed_zs.contains(z_init) {
                sieve[digit].push((*w, *z_init, *z));
            }
            sieve[digit].sort_by(min_or_max);
        }
        println!("{:?}", &sieve[digit]);
    }

    let mut solution = Vec::with_capacity(14);
    find_valid(&mut solution, &sieve[0..]);

    let mut alu = Alu::new();
    alu.execute(instructions.iter(), solution.iter());
    println!("{:?}", &alu);
}
