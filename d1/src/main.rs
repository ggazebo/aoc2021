use std::io;
use std::io::{BufRead, BufReader, Read};
use itertools::Itertools;

fn get_depths<R: Read>(rdr: R) -> impl Iterator<Item = u32> {
    let reader = BufReader::with_capacity(16, rdr);
    reader
        .lines()
        .map(|l| l.unwrap().parse::<u32>().unwrap())
}

fn main() {
    /*
    let c: u32 = get_depths(io::stdin().lock()).tuple_windows()
        .map(|(a, b)| if a > b { 0 } else { 1 })
        .sum();
        */
    let c: u32 = get_depths(io::stdin().lock()).tuple_windows::<(_,_,_)>()
        .map(|(a, b, c)| {
            let s = a + b + c;
            println!("{}", s);
            s
        })
        .tuple_windows()
        .map(|(a, b)| if b > a { 1 } else { 0 })
        .sum();

    println!("{}", c);
}
