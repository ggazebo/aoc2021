use std::cmp;
use std::fmt;
use std::io;

type Height = i8;

#[derive(Clone, Copy)]
struct Vent {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy)]
struct VentInput {
    a: Vent,
    b: Vent,
}

struct SeaFloor {
    floor: Vec<Vec<Height>>,
    x_dim: usize,
    y_dim: usize,
}

impl Vent {
    pub fn from_str(s: &str) -> Result<Vent, std::string::ParseError> {
        let comma_pos = s.find(',').unwrap();
        Ok(Vent {
            x: s[0..comma_pos].parse::<i32>().unwrap(),
            y: s[comma_pos+1..].parse::<i32>().unwrap(),
        })
    }
}

impl VentInput {
    pub fn from_str(s: &str) -> Result<VentInput, std::string::ParseError> {
        let a_end = s.find(" ->").unwrap();
        let b_start = s.find("-> ").unwrap() + 3;
        let a = Vent::from_str(&s[0..a_end]).unwrap();
        let b = Vent::from_str(&s[b_start..]).unwrap();

        Ok(VentInput { a, b })
    }

    pub fn iter(&self) -> VentIter {
        let dx = match self.b.x - self.a.x {
            0 => 0,
            d if d < 0 => -1,
            _ => 1,
        };
        let dy = match self.b.y - self.a.y {
            0 => 0,
            d if d < 0 => -1,
            _ => 1,
        };

        VentIter {
            x: self.a.x as isize, y: self.a.y as isize,
            dx, dy,
            len: (cmp::max((self.b.x - self.a.x).abs(), (self.b.y - self.a.y).abs()) + 1) as isize,
            i: 0,
        }
    }

    pub fn apply_to_map(&self, map: &mut Vec<Vec<Height>>) {
        for (xi, yi) in self.iter() {
            let x = xi as usize;
            let y = yi as usize;
            map[y][x] += 1;
        }
    }
}

struct VentIter {
    x: isize,
    y: isize,
    dx: isize,
    dy: isize,
    len: isize,
    i: isize,
}

impl Iterator for VentIter {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.len {
            return None;
        }
        let r = Some((self.x as usize, self.y as usize));
        self.i += 1;
        self.x += self.dx;
        self.y += self.dy;
        r
    }
}

impl SeaFloor {
    pub fn from_lines(lines: &Vec<VentInput>, (x_dim, y_dim): (usize, usize)) -> SeaFloor {
        let mut floor = vec!(vec!(0i8; x_dim); y_dim);

        for l in lines {
            l.apply_to_map(&mut floor);
        }

        SeaFloor { floor, x_dim, y_dim }
    }

    pub fn count_overlaps(&self) -> usize {
        let mut c = 0;
        for y in 0..self.y_dim {
            for x in 0..self.x_dim {
                if self.floor[y][x] > 1 {
                    c += 1;
                }
            }
        }
        c
    }
}

impl fmt::Display for SeaFloor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok(
            for row in &self.floor {
                for col in row {
                    match col {
                        0 => write!(f, ".")?,
                        h => write!(f, "{}", h)?,
                    }
                }
                write!(f, "\n")?;
            }
        )
    }
}

fn read_input(reader: impl io::BufRead) -> (Vec<VentInput>, usize, usize) {
    let mut lines = vec!();
    let mut x_dim: usize = 0;
    let mut y_dim: usize = 0;

    for l in reader.lines() {
        let s = l.unwrap();
        let input = VentInput::from_str(s.trim_end()).unwrap();
        lines.push(input);

        x_dim = cmp::max(x_dim, (cmp::max(input.a.x, input.b.x) + 1) as usize);
        y_dim = cmp::max(y_dim, (cmp::max(input.a.y, input.b.y) + 1) as usize);
    }

    (lines, x_dim, y_dim)
}

fn main() {
    let stdin = io::stdin();
    let (lines, x_dim, y_dim) = read_input(stdin.lock());

    let map = SeaFloor::from_lines(&lines, (x_dim, y_dim));
    println!("{}x{}", x_dim, y_dim);
    println!("{}", map);

    println!("overlaps: {}", map.count_overlaps());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_vent_line() {
        let s = "0,9 -> 5,9";
        let line = VentInput::from_str(s).unwrap();

        assert_eq!(line.a.x, 0);
        assert_eq!(line.a.y, 9);
        assert_eq!(line.b.x, 5);
        assert_eq!(line.b.y, 9);
    }
}