mod display;
mod highscore;
mod logic;
mod structs;

use crate::logic::*;

fn main() {
    let args = argh::from_env();
    run_game(&args);
}
