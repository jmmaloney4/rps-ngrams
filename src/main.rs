use std::fmt::Display;

use inquire::Select;
use rand::prelude::*;
use snafu::{prelude::*, Whatever};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RPS {
    Rock,
    Paper,
    Scissors,
}

impl RPS {
    fn all() -> Vec<RPS> {
        vec![RPS::Rock, RPS::Paper, RPS::Scissors]
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

#[derive(Debug)]
enum Winner {
    Human,
    CPU,
}

trait Player {
    fn turn(&mut self) -> Result<RPS, Whatever>;
}

struct RNGPlayer {}

impl Player for RNGPlayer {
    fn turn(&mut self) -> Result<RPS, Whatever> {
        Ok(*RPS::all().choose(&mut rand::thread_rng()).unwrap())
    }
}

struct Human {}
impl Player for Human {
    fn turn(&mut self) -> Result<RPS, Whatever> {
        Select::new("Select your move.", RPS::all())
            .prompt()
            .whatever_context("Couldn't get user input")
    }
}

fn main() -> Result<(), Whatever> {
    println!("Welcome to Rock Paper Scissors.");

    let mut human = Human {};
    let mut cpu = RNGPlayer {};

    let mut results = vec![];
    loop {
        let res = turn(&mut human, &mut cpu)?;
        match res {
            Winner::Human => println!("You Won!"),
            Winner::CPU => println!("You Lost :("),
        }
        results.push(res);
    }

    Ok(())
}

fn turn<P1, P2>(human: &mut P1, cpu: &mut P2) -> Result<Winner, Whatever>
where
    P1: Player,
    P2: Player,
{
    loop {
        return match human.turn()?.cmp(&cpu.turn()?) {
            std::cmp::Ordering::Equal => continue,
            std::cmp::Ordering::Less => Ok(Winner::Human),
            std::cmp::Ordering::Greater => Ok(Winner::CPU),
        };
    }
}
