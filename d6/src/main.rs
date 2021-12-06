use std::io;
use std::io::BufRead;

type Age = u32;
type Count = u64;

#[derive(Copy, Clone)]
pub struct Population {
    dist: [Count; 300],
}

impl Population {
    pub fn from_ages(ages: impl Iterator<Item = Age>) -> Population {
        let mut dist = [0 as Count; 300];
        for age in ages {
            dist[age as usize] += 1;
        }

        Population { dist }
    }

    pub fn tick_day(&mut self) {
        let spawning_pop = self.dist[0];
        for i in 0..9 {
            self.dist[i] = self.dist[i+1];
        }
        self.dist[6] += spawning_pop;
        self.dist[8] += spawning_pop;

        self.dist[self.dist.len()-1] = 0;
    }

    pub fn total(&self) -> Count {
        self.dist.iter().sum()
    }
}

pub fn part1(population: &mut Population) {
    for _ in 0..18 {
        population.tick_day();
    }
    println!("day 18: {}", population.total());

    for _ in 18..80 {
        population.tick_day();
    }
    println!("day 80: {}", population.total());

    for _ in 80..256 {
        population.tick_day();
    }
    println!("day 256: {}", population.total());
}

fn main() {
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut line = String::with_capacity(1200);
    stdin_lock.read_line(&mut line).unwrap();
    let ages = line.trim_end()
        .split(',')
        .map(|s| s.parse::<Age>().unwrap());
    let mut population = Population::from_ages(ages);

    part1(&mut population);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_input() {
        let ages = [3 as Age, 4, 3, 1, 2];
        let pop = Population::from_ages(ages.iter().copied());

        assert_eq!([0, 1, 1, 2, 1, 0, 0, 0, 0, 0], pop.dist);
    }

    #[test]
    fn get_population_total() {
        let pop = Population::from_ages([1, 1, 2, 1, 3].iter().copied());
        let total = pop.total();

        assert_eq!(5, total);
    }

    #[test]
    fn tick_ages_population() {
        let mut pop = Population::from_ages([1, 2, 2, 3, 4, 4, 4].iter().copied());
        pop.tick_day();

        assert_eq!([1, 2, 1, 3, 0, 0, 0, 0, 0, 0], pop.dist);
    }
}
