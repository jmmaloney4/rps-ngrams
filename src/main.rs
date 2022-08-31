use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
};

use generic_array::{ArrayLength, GenericArray};
use inquire::Select;
use itertools::Itertools;
use rand::prelude::*;
use snafu::{prelude::*, Whatever};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RPS {
    Rock,
    Paper,
    Scissors,
}

impl RPS {
    fn all() -> Vec<RPS> {
        vec![RPS::Rock, RPS::Paper, RPS::Scissors]
    }

    fn rand() -> RPS {
        *RPS::all().choose(&mut rand::thread_rng()).unwrap()
    }
}

impl PartialOrd for RPS {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(match (self, other) {
            (RPS::Rock, RPS::Rock) => std::cmp::Ordering::Equal,
            (RPS::Rock, RPS::Paper) => std::cmp::Ordering::Less,
            (RPS::Rock, RPS::Scissors) => std::cmp::Ordering::Greater,
            (RPS::Paper, RPS::Rock) => std::cmp::Ordering::Greater,
            (RPS::Paper, RPS::Paper) => std::cmp::Ordering::Equal,
            (RPS::Paper, RPS::Scissors) => std::cmp::Ordering::Less,
            (RPS::Scissors, RPS::Rock) => std::cmp::Ordering::Less,
            (RPS::Scissors, RPS::Paper) => std::cmp::Ordering::Greater,
            (RPS::Scissors, RPS::Scissors) => std::cmp::Ordering::Equal,
        })
    }
}

impl Ord for RPS {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Display for RPS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RPS::Rock => f.write_str("Rock"),
            RPS::Paper => f.write_str("Paper"),
            RPS::Scissors => f.write_str("Scissors"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum GameResult {
    Human,
    CPU,
    Tie,
}

impl Display for GameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            GameResult::Human => "Human",
            GameResult::CPU => "CPU",
            GameResult::Tie => "Tie",
        })
    }
}

trait Player {
    fn turn(&mut self) -> Result<RPS, Whatever>;
    fn post_turn(&mut self, opponent_choice: RPS, outcome: GameResult) -> Result<(), Whatever>;
}

struct RNGPlayer {}

impl Player for RNGPlayer {
    fn turn(&mut self) -> Result<RPS, Whatever> {
        Ok(RPS::rand())
    }

    fn post_turn(&mut self, _opponent_choice: RPS, _outcome: GameResult) -> Result<(), Whatever> {
        Ok(())
    }
}

struct Human {}

impl Player for Human {
    fn turn(&mut self) -> Result<RPS, Whatever> {
        Select::new("Select your move.", RPS::all())
            .prompt()
            .whatever_context("Couldn't get user input")
    }

    fn post_turn(&mut self, _opponent_choice: RPS, outcome: GameResult) -> Result<(), Whatever> {
        println!("{}", outcome);
        Ok(())
    }
}

struct NGramsPlayer {
    n: usize,
    history: VecDeque<RPS>,
    table: HashMap<Vec<RPS>, usize>,
}

impl NGramsPlayer {
    fn ngrams(n: usize) -> Vec<Vec<RPS>> {
        RPS::all()
            .into_iter()
            .combinations_with_replacement(n)
            .map(|v| v.into_iter().permutations(n))
            .flatten()
            .unique()
            .collect()
    }

    fn new(n: usize) -> Self {
        NGramsPlayer {
            n: n,
            history: VecDeque::new(),
            table: HashMap::new(),
        }
    }
}

impl Player for NGramsPlayer {
    fn turn(&mut self) -> Result<RPS, Whatever> {
        if self.history.len() < self.n {
            Ok(RPS::rand())
        } else {
            let mut search_r = self.history.clone();
            search_r.truncate(self.n - 1);
            let mut search_p = search_r.clone();
            let mut search_s = search_r.clone();
            search_r.push_front(RPS::Rock);
            search_p.push_front(RPS::Paper);
            search_s.push_front(RPS::Scissors);

            let search_r = search_r.into_iter().collect::<Vec<_>>();
            let search_p = search_p.into_iter().collect::<Vec<_>>();
            let search_s = search_s.into_iter().collect::<Vec<_>>();

            let count_r = *self.table.get(&search_r).unwrap_or(&0);
            let count_p = *self.table.get(&search_p).unwrap_or(&0);
            let count_s = *self.table.get(&search_s).unwrap_or(&0);
            let max = *vec![count_r, count_p, count_s].iter().max().unwrap();

            if count_r == max {
                return Ok(RPS::Paper);
            } else if count_p == max {
                return Ok(RPS::Scissors);
            } else if count_s == max {
                return Ok(RPS::Rock);
            } else {
                whatever!(
                    "Unexpected maximum value {} {:?}",
                    max,
                    vec![count_r, count_p, count_s]
                );
            }
        }
    }

    fn post_turn(&mut self, opponent_choice: RPS, _outcome: GameResult) -> Result<(), Whatever> {
        self.history.push_front(opponent_choice);
        if self.history.len() > self.n {
            self.history.truncate(self.n);
            let key = self.history.iter().map(|x| x.clone()).collect::<Vec<RPS>>();
            let mut entry = match self.table.get(&key) {
                Some(x) => *x,
                None => 0,
            };
            entry += 1;
            self.table.insert(key, entry);
        }
        println!("{:?}", self.table);
        Ok(())
    }
}

fn main() -> Result<(), Whatever> {
    println!("Welcome to Rock Paper Scissors.");

    // println!("{:?}", NGramsPlayer::ngrams(2));

    let mut human = Human {};
    let mut cpu = NGramsPlayer::new(3);

    let mut results = vec![];
    loop {
        let res = turn(&mut human, &mut cpu)?;
        results.push(res);
        let winrate = results.iter().fold((0, 0), |(w, t), r| match r {
            GameResult::Human => (w + 1, t + 1),
            GameResult::CPU => (w, t + 1),
            GameResult::Tie => (w, t)
        });
        println!("{}", (winrate.0 as f64) / (winrate.1 as f64));
    }

    Ok(())
}

fn turn<P1, P2>(human: &mut P1, cpu: &mut P2) -> Result<GameResult, Whatever>
where
    P1: Player,
    P2: Player,
{
    let human_choice = human.turn()?;
    let cpu_choice = cpu.turn()?;
    let outcome = match human_choice.cmp(&cpu_choice) {
        std::cmp::Ordering::Equal => GameResult::Tie,
        std::cmp::Ordering::Less => GameResult::CPU,
        std::cmp::Ordering::Greater => GameResult::Human,
    };
    println!("H: {} C: {} O: {}", human_choice, cpu_choice, outcome);
    human.post_turn(cpu_choice, outcome)?;
    cpu.post_turn(human_choice, outcome)?;

    Ok(outcome)
}
