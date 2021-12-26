use std::cmp::{min, max};
use std::fmt;
use std::io;
use std::io::BufRead;
use std::ops::RangeInclusive;

type ReactorIx = i32;
type ReactorRange = RangeInclusive<ReactorIx>;

#[derive(Clone, PartialEq, Eq)]
pub struct Cuboid {
    x: ReactorRange,
    y: ReactorRange,
    z: ReactorRange,
}

pub enum Overlap {
    Same,
    Intersection(Cuboid),
    Enclosing,
    Enclosed,
    None,
}

impl Cuboid {
    pub fn x(&self) -> ReactorRange { self.x.clone() }
    pub fn y(&self) -> ReactorRange { self.y.clone() }
    pub fn z(&self) -> ReactorRange { self.z.clone() }

    pub fn volume(&self) -> u64 {
        (self.x.end() - self.x.start() + 1).abs() as u64
            * (self.y.end() - self.y.start() + 1).abs() as u64
            * (self.z.end() - self.z.start() + 1).abs() as u64
    }

    pub fn overlaps(&self, other: &Cuboid) -> Overlap {
        if self == other {
            return Overlap::Same
        }

        enum Ov {
            Same,
            Enclosing,
            Enclosed,
            Intersecting,
            None,
        }

        let get_overlap = |a: &ReactorRange, b: &ReactorRange| match [
            a.contains(b.start()), a.contains(b.end()),
            b.contains(a.start()), b.contains(a.end())]
        {
            [true, true, true, true] => Ov::Same,
            [false, false, false, false] => Ov::None,
            [true, true, _, _] => Ov::Enclosing,
            [_, _, true, true] => Ov::Enclosed,
            _ => Ov::Intersecting,
        };

        let x_overlap = get_overlap(&self.x, &other.x);
        let y_overlap = get_overlap(&self.y, &other.y);
        let z_overlap = get_overlap(&self.z, &other.z);
        match [x_overlap, y_overlap, z_overlap] {
            [Ov::None, _, _] | [_, Ov::None, _] | [_, _, Ov::None] => Overlap::None,
            [Ov::Enclosing, Ov::Enclosing, Ov::Enclosing]
                | [Ov::Same, Ov::Enclosing, Ov::Enclosing]
                | [Ov::Enclosing, Ov::Same, Ov::Enclosing]
                | [Ov::Enclosing, Ov::Enclosing, Ov::Same]
                | [Ov::Same, Ov::Same, Ov::Enclosing]
                | [Ov::Same, Ov::Enclosing, Ov::Same]
                | [Ov::Enclosing, Ov::Same, Ov::Same]
                => Overlap::Enclosing,
            [Ov::Enclosed, Ov::Enclosed, Ov::Enclosed]
                | [Ov::Same, Ov::Enclosed, Ov::Enclosed]
                | [Ov::Enclosed, Ov::Same, Ov::Enclosed]
                | [Ov::Enclosed, Ov::Enclosed, Ov::Same]
                | [Ov::Same, Ov::Same, Ov::Enclosed]
                | [Ov::Same, Ov::Enclosed, Ov::Same]
                | [Ov::Enclosed, Ov::Same, Ov::Same]
                => Overlap::Enclosed,
            _ => Overlap::Intersection([
                max(*self.x.start(), *other.x.start())..=min(*self.x.end(), *other.x.end()),
                max(*self.y.start(), *other.y.start())..=min(*self.y.end(), *other.y.end()),
                max(*self.z.start(), *other.z.start())..=min(*self.z.end(), *other.z.end()),
            ].into())
        }
    }

    pub fn sub_into_parts(&self, hole: &Cuboid) -> Vec<Cuboid> {
        let mut l = Vec::with_capacity(6);
        // Y+
        if self.y.end() > hole.y.end() {
            l.push([self.x(), *hole.y.end()+1..=*self.y.end(), self.z()].into());
        }

        // Y-
        if self.y.start() < hole.y.start() {
            l.push([self.x(), *self.y.start()..=*hole.y.start()-1, self.z()].into());
        }
        
        // X+
        if self.x.end() > hole.x.end() {
            l.push([hole.x.end()+1..=*self.x.end(), hole.y(), self.z()].into());
        }

        // X-
        if self.x.start() < hole.x.start() {
            l.push([*self.x().start()..=*hole.x.start()-1, hole.y(), self.z()].into());
        }

        // Z+
        if self.z.end() > hole.z.end() {
            l.push([hole.x(), hole.y(), *hole.z.end()+1..=*self.z.end()].into());
        }

        // Z-
        if self.z.start() < hole.z.start() {
            l.push([hole.x(), hole.y(), *self.z.start()..=*hole.z.start()-1].into());
        }

        l
    }

    pub fn intersection(&self, other: &Cuboid) -> Option<Cuboid> {
        let x_overlaps = self.x.contains(other.x.start()) || self.x.contains(other.x.end());
        let y_overlaps = self.y.contains(other.y.start()) || self.y.contains(other.y.end());
        let z_overlaps = self.z.contains(other.z.start()) || self.z.contains(other.z.end());
        if x_overlaps && y_overlaps && z_overlaps {
            Some(Cuboid {
                x: max(*self.x.start(), *other.x.start())..=min(*self.x.end(), *other.x.end()),
                y: max(*self.y.start(), *other.y.start())..=min(*self.y.end(), *other.y.end()),
                z: max(*self.z.start(), *other.z.start())..=min(*self.z.end(), *other.z.end()),
            })
        } else {
            None
        }
    }

    pub fn into_off(&self) -> Instruction {
        Instruction { state: CubeState::Off, cuboid: self.clone() }
    }

    pub fn into_on(&self) -> Instruction {
        Instruction { state: CubeState::On, cuboid: self.clone() }
    }

    pub fn try_range_from(s: &str) -> Result<ReactorRange, &'static str> {
        let start_end = s.find('.').ok_or("failed to find \"..\"")?;
        let start = s[0..start_end].parse::<ReactorIx>().or(Err("parse fail"))?;
        let end = s[start_end+2..].parse::<ReactorIx>().or(Err("parse fail"))?;
        Ok(start..=end)
    }
}

impl From<[ReactorRange; 3]> for Cuboid {
    fn from([x, y, z]: [ReactorRange; 3]) -> Cuboid {
        Cuboid { x, y, z }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CubeState {
    On,
    Off,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Instruction {
    state: CubeState,
    cuboid: Cuboid,
}

impl Instruction {
    pub fn cuboid<'a>(&'a self) -> &'a Cuboid { &self.cuboid }

    pub fn is_boot(&self) -> bool {
        let limit = -50..=50;
        let c = self.cuboid();
        limit.contains(c.x.start()) && limit.contains(c.x.end())
            && limit.contains(c.y.start()) && limit.contains(c.y.end())
            && limit.contains(c.z.start()) && limit.contains(c.z.end())
    }

    pub fn is_on(&self) -> bool {
        self.state == CubeState::On
    }

    pub fn is_off(&self) -> bool {
        self.state == CubeState::Off
    }
}

pub trait Reactor {
    fn concat_instruction(&self, inst: &Instruction) -> Self;
}
impl Reactor for Vec<Cuboid> {
    fn concat_instruction(&self, inst: &Instruction) -> Self {
        let mut l: Vec<Cuboid> = Vec::with_capacity(self.len() + min(self.len() / 2, 10));
        let mut add_this = self.is_empty();
        let mut is_duplicate = false;
        let new_cuboid = inst.cuboid();
        for existing in self {
            match new_cuboid.overlaps(&existing) {
                Overlap::Enclosing => { add_this = inst.is_on(); },
                Overlap::Enclosed if inst.is_off() => {
                    // Remove from existing
                    l.extend(existing.sub_into_parts(new_cuboid));
                },
                Overlap::Same | Overlap::Enclosed => {
                    println!("!! Same/Enclosed: inverse={} new:{} vs existing:{}",
                        match existing.overlaps(new_cuboid) {
                            Overlap::Same => "Same",
                            Overlap::Enclosing => "Enclosing",
                            Overlap::Enclosed => "Enclosed",
                            Overlap::Intersection(_) => "Intersection",
                            Overlap::None => "None",
                        },
                        new_cuboid, existing);
                    is_duplicate = true;
                    l.push(existing.clone());
                },
                Overlap::Intersection(overlap) => {
                    let parts = existing.sub_into_parts(&overlap);
                    //l.extend(existing.sub_into_parts(&overlap));

                    if inst.is_on() {
                        for e in &parts {
                            match e.overlaps(new_cuboid) {
                                Overlap::None => (),
                                _ => { println!("!! INTERSECTION {} overlaps {}", new_cuboid, e) },
                            };
                        }
                    }

                    l.extend(parts);
                    add_this = inst.is_on();
                }
                Overlap::None => {
                    if !is_duplicate && inst.is_on() {
                        for e in &l {
                            match e.overlaps(new_cuboid) {
                                Overlap::None => (),
                                _ => { println!("!! BEFORE NO OVERLAP {} overlaps {}", new_cuboid, e); panic!("overlapping cuboid found") },
                            };
                        }
                    }

                    l.push(existing.clone());
                    add_this = !is_duplicate && inst.is_on();

                    if !is_duplicate && inst.is_on() {
                        for e in &l {
                            match e.overlaps(new_cuboid) {
                                Overlap::None => (),
                                _ => {
                                    println!("!! AFTER NO OVERLAP {} overlaps {}", new_cuboid, e);
                                    panic!("overlapping cuboid found")
                                },
                            };
                        }
                    }
                },
            };
        }
        if add_this {
            l.push(new_cuboid.clone());
        }
        l
    }
}

impl fmt::Display for Cuboid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "x={}..{},y={}..{},z={}..{}",
            self.x.start(), self.x.end(),
            self.y.start(), self.y.end(),
            self.z.start(), self.z.end())
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}",
            match self.state { CubeState::On => "on", CubeState::Off => "off"},
            self.cuboid())
    }
}

impl TryFrom<&str> for Cuboid {
    type Error = &'static str;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let y_start = s.find(",y=").ok_or("failed to find ,y=")?;
        let z_start = s.find(",z=").ok_or("failed to find ,z=")?;
        let x = Self::try_range_from(&s[2..y_start])?;
        let y = Self::try_range_from(&s[y_start+3..z_start])?;
        let z = Self::try_range_from(&s[z_start+3..])?;
        Ok(Cuboid { x, y, z })
    }
}

impl TryFrom<&str> for Instruction {
    type Error = &'static str;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let sep = s.find(' ').ok_or("failed to find space")?;
        let state = match &s[0..sep] {
            "on" => CubeState::On,
            "off" => CubeState::Off,
            _ => return Err("invalid state"),
        };
        let cuboid = Cuboid::try_from(&s[sep+1..])?;
        Ok(Instruction { state, cuboid })
    }
}


struct Instructions<I> where I: Iterator<Item = String> {
    lines: I
}

impl<I> From<I> for Instructions<I>
where I: Iterator<Item = String>
{
    fn from(lines: I) -> Self {
        Self { lines }
    }
}

impl<I> Iterator for Instructions<I>
where I: Iterator<Item = String>
{
    type Item = Instruction;
    fn next(&mut self) -> Option<Self::Item> {
        match self.lines.next() {
            Some(s) => Some(Instruction::try_from(s.as_str()).unwrap()),
            None => None,
        }
    }
}

fn _p1(instructions: &[Instruction]) {
    let mut reactor = vec![[[false; 101]; 101]; 101];
    println!("start...");
    for i in instructions.iter().filter(|&ist| ist.is_boot()) {
        for x in i.cuboid().x() {
            for y in i.cuboid().y() {
                for z in i.cuboid().z() {
                    let x = (x + 50) as usize;
                    let y = (y + 50) as usize;
                    let z = (z + 50) as usize;
                    reactor[x][y][z] = match i.state {
                        CubeState::On => true,
                        CubeState::Off => false,
                    }
                }
            }
        }
    }

    let on_count = reactor.iter()
        .map(|ys| ys.map(|zs| zs.iter().filter(|&&s| s).count()).iter().sum::<usize>())
        //.iter()
        .sum::<usize>();

    println!("ON: {}", on_count);
}

fn _p1v2(instructions: &Vec<Instruction>) {
    //let instructions: Vec<&Instruction> = instructions.iter().filter(|&i| i.is_boot()).collect();
    let instructions = instructions.iter().filter(|&i| i.is_boot()).cloned().collect();

    let sum = solve(&instructions);
    println!("result: {}", sum);
}

fn count_on(countable: &Vec<Instruction>) -> u64 {
    let mut sum = 0;
    for i in countable {
        let v = i.cuboid().volume();
        if i.is_on() {
            sum += v;
        } else {
            sum -= v;
        }
    }
    sum
}

fn solve(instructions: &Vec<Instruction>) -> u64 {
    let on_cuboids = instructions.iter().fold(vec!(), |accum, inst| accum.concat_instruction(inst));
    on_cuboids.iter().map(|c| c.volume()).sum()
}

fn _p2(instructions: &Vec<Instruction>) {
    let solution = solve(instructions);
    println!("result: {}", solution);
}

fn main() {
    let stdin = io::stdin();
    let lines = stdin.lock().lines().map(|l| l.unwrap());

    let instructions: Vec<Instruction> = Instructions::from(lines).collect();

    for inst in &instructions {
        println!("{}", inst);
    }

    //_p1(instructions.as_slice());
    //_p1v2(&instructions);
    _p2(&instructions);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiny_case() {
        let input = vec![
            Cuboid::from([10..=12, 10..=12, 10..=12]).into_on(),
            Cuboid::from([11..=13, 11..=13, 11..=13]).into_on(),
            Cuboid::from([9..=11, 9..=11, 9..=11]).into_off(),
            Cuboid::from([10..=10, 10..=10, 10..=10]).into_on(),
        ];

        assert_eq!(solve(&input), 39);
    }

    #[test]
    fn p1_example_case() {
        let input = vec![
            Cuboid::from([ -20..=26, -36..=17, -47..=7]).into_on(),
            Cuboid::from([-20..=33, -21..=23, -26..=28]).into_on(),
            Cuboid::from([-22..=28, -29..=23, -38..=16]).into_on(),
            Cuboid::from([-46..=7, -6..=46, -50..=-1]).into_on(),
            Cuboid::from([-49..=1, -3..=46, -24..=28]).into_on(),
            Cuboid::from([2..=47, -22..=22, -23..=27]).into_on(),
            Cuboid::from([-27..=23, -28..=26, -21..=29]).into_on(),
            Cuboid::from([-39..=5, -6..=47, -3..=44]).into_on(),
            Cuboid::from([-30..=21, -8..=43, -13..=34]).into_on(),
            Cuboid::from([-22..=26, -27..=20, -29..=19]).into_on(),
            Cuboid::from([-48..=-32, 26..=41, -47..=-37]).into_off(),
            Cuboid::from([-12..=35, 6..=50, -50..=-2]).into_on(),
            Cuboid::from([-48..=-32, -32..=-16, -15..=-5]).into_off(),
            Cuboid::from([-18..=26, -33..=15, -7..=46]).into_on(),
            Cuboid::from([-40..=-22, -38..=-28, 23..=41]).into_off(),
            Cuboid::from([-16..=35, -41..=10, -47..=6]).into_on(),
            Cuboid::from([-32..=-23, 11..=30, -14..=3]).into_off(),
            Cuboid::from([-49..=-5, -3..=45, -29..=18]).into_on(),
            Cuboid::from([18..=30, -20..=-8, -3..=13]).into_off(),
            Cuboid::from([-41..=9, -7..=43, -33..=15]).into_on(),
        ];

        assert_eq!(solve(&input), 590784);
    }

    #[test]
    fn p2_example_case() {
        let input = vec![
            Cuboid::from([-5..=47, -31..=22, -19..=33]).into_on(),
            Cuboid::from([-44..=5, -27..=21, -14..=35]).into_on(),
            Cuboid::from([-49..=-1, -11..=42, -10..=38]).into_on(),
            Cuboid::from([-20..=34, -40..=6, -44..=1]).into_on(),
            Cuboid::from([26..=39, 40..=50, -2..=11]).into_off(),
            Cuboid::from([-41..=5, -41..=6, -36..=8]).into_on(),
            Cuboid::from([-43..=-33, -45..=-28, 7..=25]).into_off(),
            Cuboid::from([-33..=15, -32..=19, -34..=11]).into_on(),
            Cuboid::from([35..=47, -46..=-34, -11..=5]).into_off(),
            Cuboid::from([-14..=36, -6..=44, -16..=29]).into_on(),
            Cuboid::from([-57795..=-6158, 29564..=72030, 20435..=90618]).into_on(),
            Cuboid::from([36731..=105352, -21140..=28532, 16094..=90401]).into_on(),
            Cuboid::from([30999..=107136, -53464..=15513, 8553..=71215]).into_on(),
            Cuboid::from([13528..=83982, -99403..=-27377, -24141..=23996]).into_on(),
            Cuboid::from([-72682..=-12347, 18159..=111354, 7391..=80950]).into_on(),
            Cuboid::from([-1060..=80757, -65301..=-20884, -103788..=-16709]).into_on(),
            Cuboid::from([-83015..=-9461, -72160..=-8347, -81239..=-26856]).into_on(),
            Cuboid::from([-52752..=22273, -49450..=9096, 54442..=119054]).into_on(),
            Cuboid::from([-29982..=40483, -108474..=-28371, -24328..=38471]).into_on(),
            Cuboid::from([-4958..=62750, 40422..=118853, -7672..=65583]).into_on(),
            Cuboid::from([55694..=108686, -43367..=46958, -26781..=48729]).into_on(),
            Cuboid::from([-98497..=-18186, -63569..=3412, 1232..=88485]).into_on(),
            Cuboid::from([-726..=56291, -62629..=13224, 18033..=85226]).into_on(),
            Cuboid::from([-110886..=-34664, -81338..=-8658, 8914..=63723]).into_on(),
            Cuboid::from([-55829..=24974, -16897..=54165, -121762..=-28058]).into_on(),
            Cuboid::from([-65152..=-11147, 22489..=91432, -58782..=1780]).into_on(),
            Cuboid::from([-120100..=-32970, -46592..=27473, -11695..=61039]).into_on(),
            Cuboid::from([-18631..=37533, -124565..=-50804, -35667..=28308]).into_on(),
            Cuboid::from([-57817..=18248, 49321..=117703, 5745..=55881]).into_on(),
            Cuboid::from([14781..=98692, -1341..=70827, 15753..=70151]).into_on(),
            Cuboid::from([-34419..=55919, -19626..=40991, 39015..=114138]).into_on(),
            Cuboid::from([-60785..=11593, -56135..=2999, -95368..=-26915]).into_on(),
            Cuboid::from([-32178..=58085, 17647..=101866, -91405..=-8878]).into_on(),
            Cuboid::from([-53655..=12091, 50097..=105568, -75335..=-4862]).into_on(),
            Cuboid::from([-111166..=-40997, -71714..=2688, 5609..=50954]).into_on(),
            Cuboid::from([-16602..=70118, -98693..=-44401, 5197..=76897]).into_on(),
            Cuboid::from([16383..=101554, 4615..=83635, -44907..=18747]).into_on(),
            Cuboid::from([-95822..=-15171, -19987..=48940, 10804..=104439]).into_off(),
            Cuboid::from([-89813..=-14614, 16069..=88491, -3297..=45228]).into_on(),
            Cuboid::from([41075..=99376, -20427..=49978, -52012..=13762]).into_on(),
            Cuboid::from([-21330..=50085, -17944..=62733, -112280..=-30197]).into_on(),
            Cuboid::from([-16478..=35915, 36008..=118594, -7885..=47086]).into_on(),
            Cuboid::from([-98156..=-27851, -49952..=43171, -99005..=-8456]).into_off(),
            Cuboid::from([2032..=69770, -71013..=4824, 7471..=94418]).into_off(),
            Cuboid::from([43670..=120875, -42068..=12382, -24787..=38892]).into_on(),
            Cuboid::from([37514..=111226, -45862..=25743, -16714..=54663]).into_off(),
            Cuboid::from([25699..=97951, -30668..=59918, -15349..=69697]).into_off(),
            Cuboid::from([-44271..=17935, -9516..=60759, 49131..=112598]).into_off(),
            Cuboid::from([-61695..=-5813, 40978..=94975, 8655..=80240]).into_on(),
            Cuboid::from([-101086..=-9439, -7088..=67543, 33935..=83858]).into_off(),
            Cuboid::from([18020..=114017, -48931..=32606, 21474..=89843]).into_off(),
            Cuboid::from([-77139..=10506, -89994..=-18797, -80..=59318]).into_off(),
            Cuboid::from([8476..=79288, -75520..=11602, -96624..=-24783]).into_off(),
            Cuboid::from([-47488..=-1262, 24338..=100707, 16292..=72967]).into_on(),
            Cuboid::from([-84341..=13987, 2429..=92914, -90671..=-1318]).into_off(),
            Cuboid::from([-37810..=49457, -71013..=-7894, -105357..=-13188]).into_off(),
            Cuboid::from([-27365..=46395, 31009..=98017, 15428..=76570]).into_off(),
            Cuboid::from([-70369..=-16548, 22648..=78696, -1892..=86821]).into_off(),
            Cuboid::from([-53470..=21291, -120233..=-33476, -44150..=38147]).into_on(),
            Cuboid::from([-93533..=-4276, -16170..=68771, -104985..=-24507]).into_off(),
        ];
        assert_eq!(solve(&input), 2758514936282235);
    }
}