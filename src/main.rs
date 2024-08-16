mod display;
mod highscore;
mod logic;
mod structs;

use crate::logic::*;
use crate::structs::*;
use clap::Parser;

fn main() {
    let args = Args::parse();
    run_game(&args);
}
