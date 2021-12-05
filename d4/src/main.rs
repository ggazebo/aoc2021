use std::fmt;
use std::io;
use std::io::{BufRead, BufReader};


type BingoCell = u8;

#[derive(Copy,Clone)]
struct BingoBoard {
    values: [BingoCell; 25],
}

struct BingoBoardState {
    board: BingoBoard,
    stamps: [bool; 25],
    bingo: Option<BingoCell>,
}

impl BingoBoard {
    fn read_board(reader: &mut dyn BufRead) -> Option<BingoBoard> {
        let mut values = [BingoCell::default(); 25];
        let mut buf = String::with_capacity(500);
        for r in 0..5 {
            buf.clear();
            match reader.read_line(&mut buf) {
                Ok(0) => return None,
                Ok(_) => (),
                Err(_) => panic!("IO error while reading board")
            };

            let row_values = buf
                .trim_end()
                .split_whitespace()
                .map(|s| s.parse::<BingoCell>().unwrap());

            for (c, v) in row_values.enumerate() {
                values[r * 5 + c] = v;
            }
        }

        Some(BingoBoard { values })
    }
}

impl fmt::Display for BingoBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok(for r in 0..5 {
            write!(f, "{:?}\n", &self.values[r*5..(r+1)*5])?
        })
    }
}

impl BingoBoardState {
    fn from_board(board: BingoBoard) -> BingoBoardState {
        BingoBoardState { board, stamps: [false; 25], bingo: None }
    }

    fn try_mark_value(&mut self, value: BingoCell) -> Option<(usize, usize, bool)> {
        match self.bingo {
            Some(_) => return Some((0, 0, true)),
            _ => ()
        };
        for r in 0..5 {
            for c in 0..5 {
                let i = r * 5 + c;
                if self.board.values[i] == value {
                    self.stamps[i] = true;

                    let bingo = self.stamps[r*5..r*5+5].iter().all(|b| *b)
                        || [self.stamps[c], self.stamps[c + 5], self.stamps[c + 10], self.stamps[c + 15], self.stamps[c + 20]].iter().all(|b| *b);

                    if bingo {
                        self.bingo = Some(value);
                    }
                    return Some((r, c, bingo));
                }
            }
        }
        None
    }

    fn score(&self) -> u32 {
        let sum_uncalled: u32 = self.board.values.iter()
            .zip(self.stamps)
            .filter(|(_, stamped)| !*stamped)
            .map(|(v, _)| *v as u32)
            .sum();
        sum_uncalled * (self.bingo.unwrap() as u32)
    }
}

impl fmt::Display for BingoBoardState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok(
            for r in 0..5 {
                for c in 0..5 {
                    let i = r * 5 + c;
                    let v = self.board.values[i];
                    if self.stamps[i] {
                        write!(f, "|{:2}| ", v)?
                    }
                    else {
                        write!(f, " {:2}  ", v)?
                    }
                }
                write!(f, "\n")?
            }
        )
    }
}

fn read_input(stdin: io::Stdin) -> (Vec<BingoCell>, Vec<BingoBoard>) {
    let mut reader = BufReader::with_capacity(32, stdin.lock());
    let mut buf = String::with_capacity(1204);

    reader.read_line(&mut buf).unwrap();
    let calls = buf.trim_end().split(',')
        .map(|s| s.parse::<BingoCell>().unwrap())
        .collect();

    reader.read_line(&mut buf).unwrap();
    buf.clear();

    let mut boards = vec!();
    while let Some(board) = BingoBoard::read_board(&mut reader) {
        boards.push(board);
        reader.read_line(&mut buf).unwrap();
    }

    (calls, boards)
}

fn p1(calls: Vec<BingoCell>, base_boards: Vec<BingoBoard>) {
    let mut boards: Vec<BingoBoardState> = base_boards
        .iter()
        .map(|b| BingoBoardState::from_board(*b))
        .collect();

    let mut winner = None;
    for call in calls {

        for (b, board) in boards.iter_mut().enumerate() {
            match board.try_mark_value(call) {
                Some((_, _, true)) => winner = Some((b, call)),
                _ => (),
            }
        }

        match winner {
            Some((b, call)) => {
                println!("BINGO on board {}: {}\n", b, call);
                break;
            },
            _ => ()
        }
    }

    for board in &boards {
        println!("{}", board);
    }

    let (winning_board, winning_call) = winner.unwrap();

    let score = &boards[winning_board].score();
    println!("{}", score);
}

fn p2(calls: Vec<BingoCell>, base_boards: Vec<BingoBoard>) {
    let mut boards: Vec<BingoBoardState> = base_boards
        .iter()
        .map(|b| BingoBoardState::from_board(*b))
        .collect();

    //let mut winner : Option<&BingoBoardState> = None;
    //let mut winner_num = None;
    //let mut loser_num = None;
    for call in calls {
        for (b, board) in boards.iter_mut().enumerate() {
            match board.bingo {
                None => match board.try_mark_value(call) {
                    Some((_, _, true)) => {
                        println!("BINGO on board {}: {}", b, call);
                        println!("score: {}", board.score());
                        println!("{}", board);
                        /*
                        match winner_num { 
                            None => winner_num = Some(b),
                            _ => {
                                loser_num = Some(b);
                                //break;
                            }
                        }
                        */
                    },
                    _ => (),
                },
                Some(_) => (),
            }
        }
    }

    /*
    for board in &boards {
        if board.bingo.is_some() {
            println!("{}", board);
        }
    }
    */

    /*
    let loser_board = &boards[loser_num.unwrap()];
    let score = loser_board.score();
    println!("{}", score);
    */
}

fn main() {
    let stdin = io::stdin();
    let (calls, base_boards) = read_input(stdin);

    println!("{:?}", calls);
    println!("");

    /*
    for board in &base_boards {
        println!("{}", board);
    }
    */

    //p1(calls, base_boards);
    p2(calls, base_boards);
}
