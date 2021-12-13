use std::collections::HashSet;
use std::cmp;
use std::fmt;
use std::io;
use std::io::BufRead;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Dot(i32, i32);

impl Dot {
    pub fn from_str(s: &str) -> Option<Dot> {
        let mut it = s.split(',');
        let x = it.next()?.parse::<i32>().ok()?;
        let y = it.next()?.parse::<i32>().ok()?;
        Some(Dot(x, y))
    }

    pub fn fold_by(&self, fold: &Fold) -> (Dot, bool) {
        match fold {
            Fold::Horizontal(x) if self.0 > *x => (Dot(x - (self.0 - x), self.1), true),
            Fold::Vertical(y) if self.1 > *y => (Dot(self.0, y - (self.1 - y)), true),
            _ => (*self, false),
        }
    }
}

impl fmt::Display for Dot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.0, self.1)
    }
}

impl fmt::Debug for Dot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Fold {
    Horizontal(i32),
    Vertical(i32),
}

impl Fold {
    pub fn from_str(s: &str) -> Option<Fold> {
        let mut it = s.split('=');
        let axis = it.next()?.chars().last()?;
        let n = it.next()?.parse::<i32>().ok()?;

        match axis {
            'x' => Some(Fold::Horizontal(n)),
            'y' => Some(Fold::Vertical(n)),
            _ => None
        }
    }
}

impl fmt::Display for Fold {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = match self {
            Fold::Horizontal(n) => ('x', n),
            Fold::Vertical(n) => ('y', n),
        };
        write!(f, "fold along {}={}", v.0, v.1)
    }
}

pub fn fold_paper(paper: &mut HashSet<Dot>, fold: &Fold) -> usize {
    let mut moves = vec!();
    for dot in paper.iter() {
        match dot.fold_by(fold) {
            (d, true) => {
                moves.push((*dot, d.clone()));
            }
            (_, false) => (),
        }
    }

    for (from, to) in moves {
        //println!("{} -> {}", to, from);
        paper.insert(to);
        paper.remove(&from);
    }

    paper.len()
}

fn print_paper(paper: &HashSet<Dot>) {
    let (width, height) = paper.iter()
        .fold((0, 0), |a, d| (cmp::max(a.0, d.0), cmp::max(a.1, d.1)));

    let (width, height) = ((width + 1) as usize, (height + 1) as usize);

    let mut grid = vec![false; width * height];
    for d in paper.iter() {
        let i = (d.1 * width as i32 + d.0) as usize;
        grid[i] = true;
    }

    for y in 0..height {
        for x in 0..width {
            print!("{}", if grid[y * width + x] { '#' } else { '.' });
        }
        println!();
    }
}

fn main() {
    let stdin = io::stdin();

    let mut lines = stdin.lock().lines().map(|l| l.unwrap());

    let mut dots = HashSet::new();
    loop {
        let l = lines.next().unwrap();
        let l = l.trim_end();
        if l.len() == 0 {
            break;
        }

        let dot = Dot::from_str(l).unwrap();
        dots.insert(dot);
    }

    let folds: Vec<Fold> = lines.map(|s| Fold::from_str(s.trim_end()).unwrap()).collect();

    /*
    for d in &dots {
        println!("{}", d);
    }
    */
    //print_paper(&dots);
    println!("starting  with {} dots", dots.len());
    /*
    for f in &folds {
        println!("{}", f);
    }
    */

    println!();

    for f in &folds {
        fold_paper(&mut dots, f);
        println!("after {}: {} dots", f, dots.len());
    }

    print_paper(&dots);
}
