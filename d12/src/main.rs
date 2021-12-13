use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::io::BufRead;
use typed_arena::Arena;

#[derive(PartialEq, Eq, PartialOrd, Hash, Clone)]
pub struct Cave(String);

impl Cave {
    pub fn from(s: &str) -> Cave {
        Cave(s.to_owned())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn is_start(&self) -> bool {
        self.as_str() == "start"
    }

    pub fn is_end(&self) -> bool {
        self.as_str() == "end"
    }

    pub fn is_small(&self) -> bool {
        !self.as_str().chars().any(|c| c.is_ascii_uppercase())
    }
}

impl Ord for Cave {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.as_str() {
            "start" => Ordering::Less,
            "end" => Ordering::Greater,
            s => s.cmp(other.0.as_str()),
        }
    }
}

impl fmt::Display for Cave {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for Cave {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub struct CaveMap<'a> {
    index: HashMap<&'a Cave, Vec<&'a Cave>>
}

impl<'a> CaveMap<'a> {
    fn from_input<'b>(specs: impl Iterator<Item = String>, arena: &'b Arena<Cave>) -> CaveMap<'b> {
        let mut map = CaveMap { index: HashMap::new() };

        for s in specs {
            let mut splits = s.split('-');

            let a = Cave::from(splits.next().unwrap());
            let a = match map.index.get_key_value(&a) {
                Some((&k, _)) => k,
                None => arena.alloc(a),
            };

            let b = Cave::from(splits.next().unwrap());
            let b = match map.index.get_key_value(&b) {
                Some((&k, _)) => k,
                None => arena.alloc(b),
            };

            map.index.entry(a).and_modify(|p| p.push(b)).or_insert(vec!(b));
            map.index.entry(b).and_modify(|p| p.push(a)).or_insert(vec!(a));
        }

        map
    }

    pub fn next_from<'b>(&self, c: &Cave) -> Option<&'b Vec<&Cave>> {
        self.index.get(c)
    }

    pub fn each_path<F>(&self, f: &F) -> usize
        where F: Fn(&Vec<&Cave>) -> ()
    {
        let start = Cave::from("start");
        let mut path = vec!(*self.index.get_key_value(&start).unwrap().0);
        self.traverse_all(&mut path, None, &f)
    }

    fn traverse_all<'b, F>(&self,
        path: &'b mut Vec<&'a Cave>,
        big_small: Option<&Cave>,
        on_end: &F) -> usize
    where F: Fn(&Vec<&Cave>) -> ()
    {
        //println!("=>: {:?}", big_small);
        let this_cave = *path.last().unwrap();

        if this_cave.is_end() {
            on_end(path);
            return 1
        }

        let branches = self.index.get(&this_cave).unwrap();

        let mut sum = 0;
        for &c in branches {
            let repeated_small = if c.is_small() && path.iter().any(|&visited| c == visited) {
                match big_small {
                    None if !c.is_start() && !c.is_end() => {
                        /*
                        println!("bonus: {} ({:?})", c, big_small);
                        path.push(c);
                        sum += self.traverse_all(path, Some(c), on_end);
                        path.pop();
                        */
                        Some(c)
                    }
                    _ => {
                        continue;
                    }
                }
            } else {
                big_small
            };

            path.push(c);
            sum += self.traverse_all(path, repeated_small, on_end);
            path.pop();
        }
        sum
    }
}

fn main() {
    let stdin = std::io::stdin();
    let arena = Arena::new();

    let map = CaveMap::from_input(stdin.lock().lines().map(|l| l.unwrap()), &arena);

    /*
    for (k, v) in map.index.iter() {
        for c in v {
            println!("{} -> {}", k.as_str(), c.as_str());
        }
    }
    */

    let count = map.each_path(&|path| {
        /*
        for c in path {
            print!("->{}", c);
        }
        println!();
        */
    });

    println!("{} paths", count);
}
