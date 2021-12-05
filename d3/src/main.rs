use std::io;
use std::io::{BufRead, BufReader, Read};

type BitFrequency = [i8; 12];

fn get_numbers<R: Read>(rdr: R) -> impl Iterator<Item = BitFrequency> {
    let reader = BufReader::with_capacity(16, rdr);
    reader
        .lines()
        .map(|l| {
            let mut a = [0i8; 12];
            for (i, c) in l.unwrap().chars().enumerate() {
                a[i] = match c {
                    '0' => -1,
                    '1' => 1,
                    _ => panic!()
                }
            }
            a
        })
}

fn part1(stdin: io::Stdin) {
    let x = get_numbers(stdin.lock())
        .fold([0i8; 12], |a, next| {
            let mut r = [0i8; 12];
            for (i, c) in next.iter().enumerate() {
                r[i] = a[i] + c
            }
            r
        });
    println!("{:?}", x);

    let mut gamma = 0u32;
    for (i, &n) in x.iter().enumerate() {
        if n > 0 {
            gamma |= 1 << (11 - i);
        }
    }
    let epsilon = gamma ^ 0b111111111111;
    println!("gamma: {}, eps: {}", gamma, epsilon);

    let consumption = gamma * epsilon;
    println!("{}", consumption);

}

#[derive(Clone, Copy)]
enum FrequencyBias {
    More,
    Less,
}

fn filter_data(values: &Vec::<Vec<char>>, tie_bias: char, freq_bias: FrequencyBias) -> i32 {
    let num_bits = values[0].len();

    let mut f = values.iter().map(|v| v).collect();

    for i in 0..num_bits {
        f = filter_data_impl(f, i, tie_bias, freq_bias);
    };

    let v = f[0];
    println!("{:?}", v);

    v.iter().fold(0i32, |a, n| (a << 1) | match n { '1' => 1, _ => 0 })
}

fn filter_data_impl<'a>(values: Vec::<&'a Vec<char>>, bit_index: usize, tie_bias: char, freq_bias: FrequencyBias) -> Vec::<&'a Vec<char>> {
    if values.len() == 1 {
        return values
    }

    let bias = values
        .iter()
        .map(|v| { v[bit_index] })
        .fold(0i32, |b, c| {
            b + match c {
                '1' => 1,
                '0' => -1,
                _ => 0,
            }
        });

    let pick = if bias == 0 {
        tie_bias
    }
    else {
        match freq_bias {
            FrequencyBias::More => if bias > 0 { '1' } else { '0' }
            FrequencyBias::Less => if bias > 0 { '0' } else { '1' }
        }
    };

    values
        .iter()
        .filter(|v| v[bit_index] == pick)
        .map(|v| *v)
        .collect()
}

fn part2(stdin: io::Stdin) {
    let mut data = Vec::<Vec<char>>::with_capacity(1000);
    //let mut bit_biases = Vec::<[i32; 16]>::with_capacity(1000);
    for l in BufReader::with_capacity(16, stdin.lock()).lines() {
        data.push(l.unwrap().chars().collect());
        //bit_biases.push([0i32; 16]);
    }

    let oxygen = filter_data(&data, '1', FrequencyBias::More);
    println!("{:?}", oxygen);

    let co2 = filter_data(&data, '0', FrequencyBias::Less);
    println!("{:?}", co2);

    println!("{}", oxygen * co2)

    /*
    let num_bits = data[0].len();
    
    println!("{} bits", num_bits);

    for (i, d) in data.iter().enumerate() {
        for (j, c) in d.iter().enumerate() {
            let bit_index = num_bits - j - 1;
            bit_biases[i][bit_index] += match c {
                '0' => -1,
                '1' => 1,
                _ => 0,
            }
        }
    }

    for bias in bit_biases {
        println!("{:?}", bias);
    }
    */
}

fn main() {
    let stdin = io::stdin();
    //part1(stdin);
    part2(stdin);
}
