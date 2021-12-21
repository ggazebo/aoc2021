use std::collections::HashSet;
use std::io;
use std::io::{BufRead};

mod ipos;
use ipos::*;

fn merge_if_overlap(beacons: &mut HashSet<Pos>, other: &Vec<Pos>) -> Option<(Rotation, Pos)>
{
    for rotation in Rotation::all() {
        let new_data: Vec<Pos> = other.iter().map(|p| p.rotate(rotation)).collect();
        for &pin in beacons.iter() {
            for &other_pin in new_data.iter() {
                let offset = pin - other_pin;
                let mut matched = 1;

                for pos in new_data.iter().map(|&p| p + offset) {
                    if beacons.contains(&pos) {
                        matched += 1;
                    }
                    if matched >= 12 {
                        beacons.extend(new_data.iter().map(|&p| p + offset));
                        return Some((rotation, offset))
                    }
                }
            }
        }
    }

    None
}

fn find_max_manhattan(positions: impl IntoIterator<Item = Pos>) -> Int {
    let beacons: Vec<Pos> = positions.into_iter().collect();
    let mut max = 0;
    for i in 0..beacons.len() {
        let sub = &beacons[i..];
        let a = sub[0];
        max = std::cmp::max(
            max,
            sub[1..].iter()
                .map(|&p| a.manhattan(p))
                .max()
                .unwrap_or(0));
    }
    max
}

fn read_input(lines: &mut impl Iterator<Item = String>) -> Vec<Vec<Pos>> {
    let mut scans = vec!();
    loop {
        // header
        match lines.next() {
            Some(_) => (),
            None => break,
        };

        let mut positions = vec!();
        loop {
            let p = match lines.next() {
                Some(s) if s.len() > 0 => Pos::try_from(s.as_ref()).unwrap(),
                _ => break,
            };
            positions.push(p);
        }
        scans.push(positions);
    }
    scans
}

fn main() {
    /*
    for r in Rotation::all() {
        let v = Pos::from([1, 2, 3]).rotate(r);
        println!("{}", v);
    }
    */
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines().map(|l| l.unwrap());
    let data = read_input(&mut lines);

    let reference = &data[0];
    let mut scanners = Vec::with_capacity(10);
    let mut beacons = HashSet::from_iter(reference.iter().copied());
    let mut to_match = vec!();
    for (i, info) in data[1..].iter().enumerate() {
        to_match.push((i + 1, info));
    }

    loop {
        let (sensor_id, sensor_data) = match to_match.last() {
            Some(x) => x,
            None => break,
        };

        let overlap = merge_if_overlap(&mut beacons, sensor_data);

        match overlap {
            Some((rot, offset)) => {
                println!("scanner {} matched with {:?} + {}", sensor_id, rot, offset);
                scanners.push((*sensor_id, rot, offset));
                to_match.pop();
            },
            None => {
                //println!("scanner {} has no match", sensor_id);
                to_match.rotate_left(1);
                assert_ne!(to_match.len(), 1);
            }
        };
    }

    println!("{} total beacons", beacons.len());

    let scanner_positions: Vec<Pos> = scanners.iter().map(|(_,_,p)| *p).collect();
    println!("max manhattan: {}", find_max_manhattan(scanner_positions));
}
