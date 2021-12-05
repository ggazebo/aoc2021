use std::io;
use std::fmt;
use std::io::{BufRead, BufReader, Read};

#[derive(Clone,Copy)]
enum Direction {
    Forward,
    Up,
    Down,
}

#[derive(Clone,Copy)]
struct Movement {
    direction: Direction,
    distance: i32,
}

#[derive(Default,Clone,Copy)]
struct Position {
    depth: i32,
    horizontal: i32,
    aim: i32,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Direction::Forward => "FORWARD",
            Direction::Up => "UP",
            Direction::Down => "DOWN",
        })
    }
}

impl Movement {
    fn from_string(s: String) -> Result<Movement, &'static str> {
        let mut iter = s.split_ascii_whitespace();
        let dir = match iter.next().unwrap() {
            "forward" => Direction::Forward,
            "up" => Direction::Up,
            "down" => Direction::Down,
            _ => panic!("bad direction")
        };
        let dist = iter.next().unwrap();

        Ok(Movement {
            direction: dir,
            distance: dist.parse::<i32>().unwrap(),
        })
    }
}

impl fmt::Display for Movement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.direction, self.distance)
    }
}

impl Position {
    fn new(depth: i32, horizontal: i32, aim: i32) -> Self {
        Self { depth, horizontal, aim}
    }

    fn move_by(&self, m: Movement) -> Position {
        match m.direction {
            Direction::Forward => Position::new(self.depth + self.aim * m.distance, self.horizontal + m.distance, self.aim),
            Direction::Up => Position::new(self.depth, self.horizontal, self.aim - m.distance),
            Direction::Down => Position::new(self.depth, self.horizontal, self.aim + m.distance),
        }
    }
}

fn get_orders<R: Read>(rdr: R) -> impl Iterator<Item = Movement> {
    let reader = BufReader::with_capacity(16, rdr);
    reader
        .lines()
        .map(|l| {
            let m = Movement::from_string(l.unwrap()).unwrap();
            //println!("{} {}", m.direction, m.distance);
            m
        })
}

fn main() {
    let stdin = io::stdin();
    /*
    for order in get_orders(stdin.lock()) {
        println!("{}", order);
    }
    */
    let x = get_orders(stdin.lock())
        .fold(Position::default(), |p, m| {
            let new_p = p.move_by(m);
            println!("{} ({}, {})", m, new_p.depth, new_p.horizontal);
            new_p
        });

    println!("{}", x.depth * x.horizontal)
}
