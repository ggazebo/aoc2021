use std::fmt;
use std::io;
use std::io::BufRead;
use std::ops;

pub enum Segment {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

#[derive(PartialEq, Clone, Copy)]
pub struct SevenSegDisplay(u8);

impl SevenSegDisplay {
    pub fn empty() -> SevenSegDisplay {
        SevenSegDisplay(0)
    }

    pub fn from_str(s: &str) -> Result<SevenSegDisplay, &'static str> {
        let mut segments = 0u8;
        for c in s.chars() {
            segments |= match c {
                'a' => 0b01000000,
                'b' => 0b00100000,
                'c' => 0b00010000,
                'd' => 0b00001000,
                'e' => 0b00000100,
                'f' => 0b00000010,
                'g' => 0b00000001,
                _ => return Err("bad segment specifier"),
            };
        }

        Ok(SevenSegDisplay(segments))
    }

    pub const fn raw(&self) -> u8 {
        self.0
    }

    pub const fn count_segments(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn has_segment(&self, s: Segment) -> bool {
        let mask = match s {
            Segment::A => 0b01000000,
            Segment::B => 0b00100000,
            Segment::C => 0b00010000,
            Segment::D => 0b00001000,
            Segment::E => 0b00000100,
            Segment::F => 0b00000010,
            Segment::G => 0b00000001,
        };
        self.0 & mask == mask
    }

    pub fn to_value(&self) -> Option<u8> {
        Some(match self.0.count_ones() {
            2 => 1,
            4 => 4,
            3 => 7,
            7 => 8,
            _ => return None,
        })
    }

    pub fn to_decoded_value(&self, decoder: &Decoder) -> Option<u8> {
        decoder.decode(&self)
    }
}

impl ops::BitAnd for SevenSegDisplay {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl fmt::Display for SevenSegDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok({
            writeln!(f, " {} ", if self.has_segment(Segment::A) { "XX" } else { ".." })?;
            for _ in 0..2 {
                writeln!(f, "{}  {}",
                    if self.has_segment(Segment::B) { "X" } else { "." },
                    if self.has_segment(Segment::C) { "X" } else { "." })?;
            }
            writeln!(f, " {} ", if self.has_segment(Segment::D) { "XX" } else { ".." })?;
            for _ in 0..2 {
                writeln!(f, "{}  {}",
                    if self.has_segment(Segment::E) { "X" } else { "." },
                    if self.has_segment(Segment::F) { "X" } else { "." })?;
            }
            writeln!(f, " {} ", if self.has_segment(Segment::G) { "XX" } else { ".." })?;
        })
    }
}

pub struct Decoder([SevenSegDisplay; 10]);

impl Decoder {
    pub fn from_samples(samples: &Vec<SevenSegDisplay>) -> Decoder {
        let mut map = [SevenSegDisplay::empty(); 10];

        for &s in samples {
            match s.to_value() {
                Some(v) => map[v as usize] = s,
                _ => (),
            };
        }

        for &s in samples.iter().filter(|&d| d.to_value().is_none()) {
            let num = match (s, s.count_segments()) {
                (m, c) if c == 5 && (m & map[4]).count_segments() == 2 => 2,
                (m, c) if c == 5 && (m & map[7]).count_segments() == 3 => 3,
                (m, c) if c == 5 && (m & map[7]).count_segments() == 2 => 5,
                (m, c) if c == 6 && (m & map[7]).count_segments() == 2 => 6,
                (m, c) if c == 6 && (m & map[4]).count_segments() == 3 => 0, // will also match 6 case, so  order matters
                (m, c) if c == 6 && (m & map[4]).count_segments() == 4 => 9,
                _ => panic!(),
            };
            map[num] = s;
        }

        Decoder(map)
    }

    pub fn decode(&self, d: &SevenSegDisplay) -> Option<u8> {
        match self.0.iter().position(|v| v == d) {
            Some(i) => Some(i as u8),
            None => None,
        }
    }
}

fn part1(actual: &Vec<SevenSegDisplay>) -> usize {
    actual.iter()
        .filter(|d| d.to_value().is_some())
        .count()
}

fn parse_line(l: &String) -> (Vec<SevenSegDisplay>, Vec<SevenSegDisplay>) {
    let mut parts = l.split('|');
    let samples = parts.next().unwrap().trim_end().split_ascii_whitespace()
        .map(|s| SevenSegDisplay::from_str(s).unwrap())
        .collect();

    let actual = parts.next().unwrap().trim().split_ascii_whitespace()
        .map(|s| SevenSegDisplay::from_str(s).unwrap())
        .collect();

    (samples, actual)
}

fn main() {
    let stdin = io::stdin();
    let mut p1_total: usize = 0;
    let mut sum = 0u32;
    for l in stdin.lock().lines() {
        let (samples, actual) = parse_line(&l.unwrap());
        let decoder = Decoder::from_samples(&samples);
        for d in &samples {
            match d.to_decoded_value(&decoder) {
                Some(d) => print!("{} ", d),
                None => print!("? "),
            }
        }
        print!("| ");
        for d in &actual {
            match d.to_value() {
                Some(d) => print!("{} ", d),
                None => print!("? "),
            }
        }
        let c = part1(&actual);
        print!("({})", part1(&actual));
        p1_total += c;

        // Part 2
        let mut num = 0u32;
        print!(" || ");
        for d in &actual {
            match d.to_decoded_value(&decoder) {
                Some(v) => {
                    print!("{} ", v);
                    num = num * 10 + v as u32;
                },
                None => print!("? "),
            }
        }
        print!("| {}", num);
        sum += num;

        println!("");
    }
    println!("p1: {}", p1_total);
    println!("p2: {}", sum);
}
