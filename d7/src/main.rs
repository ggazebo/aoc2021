use std::io;
use std::io::BufRead;

type Position = i32;
type Fuel = i32;
type PosInput = Vec<Position>;
type FuelCalculator = fn(&PosInput, Position) -> (Position, Fuel);

fn get_fuel_cost(positions: &PosInput, target_pos: Position) -> Fuel {
    positions
        .iter()
        .map(|p| (target_pos - p).abs() as Fuel)
        .sum()
}

fn get_true_fuel_cost(positions: &PosInput, target_pos: Position) -> Fuel {
    positions
        .iter()
        .map(|p| {
            let d = (target_pos - p).abs();
            (d + 1) * d / 2 as Fuel
        })
        .sum()
}

fn get_optimal_pos(positions: &PosInput, get_fuel: &dyn Fn(&PosInput, Position)->Fuel) -> (Position, Fuel) {
    let min_pos = *positions.iter().min().unwrap();
    let max_pos = *positions.iter().max().unwrap();

    (min_pos..=max_pos)
        .map(|p| (p as Position, get_fuel(&positions, p as Position)))
        .min_by(|(_, a), (_, b)| a.cmp(b))
        .unwrap()
}

fn part1(positions: &Vec<Position>) {
    let (min_pos, min_fuel) = get_optimal_pos(positions, &get_fuel_cost);

    println!("{} for {}", min_pos, min_fuel);
}

fn part2(positions: &Vec<Position>) {
    let (min_pos, min_fuel) = get_optimal_pos(positions, &get_true_fuel_cost);

    println!("{} for {}", min_pos, min_fuel);
}

fn main() {
    let stdin = io::stdin();
    let line = stdin.lock().lines().next().unwrap().unwrap();
    let positions = line.trim_end()
        .split(',')
        .map(|s| s.parse::<Position>().unwrap())
        .collect();

    //part1(&positions);
    part2(&positions);
}
