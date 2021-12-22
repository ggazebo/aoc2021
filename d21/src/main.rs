use std::collections::HashMap;
use std::fmt;
use std::ops::{Add, AddAssign};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position(u32);

pub type Roll = u32;
pub type Score = u32;

const BOARD_SIZE: u32 = 10;

impl Position {
    pub fn pos(&self) -> u32 { self.0 + 1 }
}

impl From<u32> for Position {
    fn from(start: u32) -> Position {
        Position(start - 1)
    }
}

impl Add<Roll> for Position {
    type Output = Self;
    fn add(self, roll: Roll) -> Self::Output { Position((self.0 + roll ) % BOARD_SIZE) }
}

impl AddAssign<Roll> for Position {
    fn add_assign(&mut self, roll: Roll)  { *self = *self + roll }
}

impl Add<Position> for Score {
    type Output = Self;
    fn add(self, pos: Position) -> Self::Output { self + pos.pos()}
}

impl AddAssign<Position> for Score {
    fn add_assign(&mut self, pos: Position) { *self = *self + pos }
}

pub struct DetermenisticDice {
    n: DiceRoll,
    max: DiceRoll,
    count: u32,
}

pub type DiceRoll = u32;

pub trait Dice {
    fn roll(&mut self) -> DiceRoll;
    fn count(&self) -> u32;
}

impl DetermenisticDice {
    pub fn new() -> Self {
        DetermenisticDice { n: 0, max: 100, count: 0 }
    }
}

impl Dice for DetermenisticDice {
    fn roll(&mut self) -> DiceRoll {
        let n = self.n;
        self.n = (self.n + 1) % self.max;
        self.count += 1;
        n + 1
    }

    fn count(&self) -> u32 { self.count }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Player {
    position: Position,
    score: Score,
}

impl Player {
    pub fn position(&self) -> Position { self.position }
    pub fn score(&self) -> Score { self.score }

    pub fn start_at(position: Position) -> Player {
        Player { position, score: 0 }
    }

    pub fn take_turn(&mut self, dice: &mut impl Dice) -> [DiceRoll; 3] {
        let mut rolls = [Default::default(); 3];
        rolls.fill_with(|| dice.roll());
        self.take_turn_det(&rolls);
        rolls
    }

    pub fn take_turn_det(&mut self, rolls: &[DiceRoll; 3]) {
        self.position += rolls.iter().sum();
        self.score += self.position();
    }
}

impl fmt::Debug for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", self.position().pos(), self.score)
    }
}

type DiracDiceStateCounter = HashMap<Player, usize>;

pub struct DiracDiceTurn {
    turn: usize,
    player1: DiracDiceStateCounter,
    player2: DiracDiceStateCounter,
    player1_wins: usize,
    player2_wins: usize,
}

impl DiracDiceTurn {
    pub fn turn(&self) -> usize { self.turn }

    pub fn from_starts(player1: Position, player2: Position) -> DiracDiceTurn {
        DiracDiceTurn {
            turn: 0,
            player1: DiracDiceStateCounter::from([(Player::start_at(player1), 1)]),
            player2: DiracDiceStateCounter::from([(Player::start_at(player2), 1)]),
            player1_wins: 0,
            player2_wins: 0,
        }
    }

    pub fn player_turn(now: &DiracDiceStateCounter, next: &mut DiracDiceStateCounter, other_player_states: usize) -> usize {
        let mut wins = 0;
        for (p, &count) in now.iter() {
            for roll in &Self::ROLLS {
                let player = &mut p.clone();
                player.take_turn_det(roll);

                if player.score() >= 21 {
                    wins += count;
                } else {
                    next.entry(*player).and_modify(|c| *c += count).or_insert(count);
                }
            }
        }
        wins * other_player_states
    }

    const ROLLS: [[Roll; 3]; 27] = [
        [3,3,1],[3,3,2],[3,3,3],
        [3,2,1],[3,2,2],[3,2,3],
        [3,1,1],[3,1,2],[3,1,3],
        [2,3,1],[2,3,2],[2,3,3],
        [2,2,1],[2,2,2],[2,2,3],
        [2,1,1],[2,1,2],[2,1,3],
        [1,1,1],[1,1,2],[1,1,3],
        [1,2,1],[1,2,2],[1,2,3],
        [1,3,1],[1,3,2],[1,3,3],
    ];
}

impl Iterator for DiracDiceTurn {
    type Item = Self;

    fn next(&mut self) -> Option<Self::Item> {
        if self.player1.is_empty() && self.player2.is_empty() {
            return None
        }

        let mut player1 = HashMap::new();
        let mut player2 = HashMap::new();
        let mut player1_wins = self.player1_wins;
        let mut player2_wins = self.player2_wins;

        // Player 1 takes turn
        player1_wins += Self::player_turn(&self.player1, &mut player1, self.player2.values().sum());

        // Player 2 turn
        player2_wins += Self::player_turn(&self.player2, &mut player2, player1.values().sum());

        Some(Self { turn: self.turn + 1, player1, player2, player1_wins, player2_wins })
    }
}

fn _p1(pos1: Position, pos2: Position, dice: &mut impl Dice) {
    let mut player1 = Player::start_at(pos1);
    let mut player2 = Player::start_at(pos2);

    loop {
        let rolls = player1.take_turn(dice);
        println!("player1 :: {:?} after {:?}", &player1, &rolls);
        if player1.score() >= 1000 {
            println!("player 1 wins!");
            println!("loser score: {}*{} = {}", player2.score, dice.count(), player2.score() * dice.count());
            break;
        }

        let rolls = player2.take_turn(dice);
        println!("player2 :: {:?} after {:?}", &player2, &rolls);
        if player2.score() >= 1000 {
            println!("player 2 wins!");
            println!("loser score: {}*{} = {}", player1.score, dice.count(), player1.score() * dice.count());
            break;
        }
    }
}

fn p2(pos1: Position, pos2: Position) {
    let mut turn = DiracDiceTurn::from_starts(pos1, pos2);
    for _ in 0..11 {
        turn = match turn.next() {
            Some(turn) => turn,
            None => break
        };
        
        println!("Turn {}: ", turn.turn());
        println!("wins: {} vs {}", turn.player1_wins, turn.player2_wins);
        println!("player1 states: {:?}", turn.player1);
        println!("player2 states: {:?}", turn.player2);
    }
}

fn main() {
    // Test:
    // Player 1 starting position: 4
    // Player 2 starting position: 8

    // Real:
    // Player 1 starting position: 1
    // Player 2 starting position: 2

    //let mut dice = DetermenisticDice::new();
    //let mut player1 = Player::start_at(Position::from(4));
    //let mut player2 = Player::start_at(Position::from(8));

    //let pos1 = Position::from(4);
    //let pos2 = Position::from(8);
    let pos1 = Position::from(1);
    let pos2 = Position::from(2);

    //p1(pos1, pos2, &mut dice);
    p2(pos1, pos2);
}
