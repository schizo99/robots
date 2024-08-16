/// This module contains the definitions of various structs used in the application.
///
/// The `GameState` struct represents the state of the game, including the current turn, level, and other game-related flags.
///
/// The `Player` struct represents a player in the game, with properties such as username, score, position, and abilities.
///
/// The `Item` struct represents an item in the game, with properties such as position, level, kind, and visibility.
///
/// The `DumbRobot` struct represents a dumb robot in the game, with properties such as position and kind.
///
/// The `JunkHeap` struct represents a junk heap in the game, with properties such as position.
///
/// The `Args` struct is used for parsing command line arguments using the `clap` crate. It contains properties such as username, path, and show_highscore flag.
use clap::Parser;

pub const PADDING_LEFT: i32 = 3;
pub const PADDING_TOP: i32 = 1;
pub const BOARD_WIDTH: i32 = 60;
pub const BOARD_HEIGHT: i32 = 24;

#[derive(Clone, Copy)]
pub struct GameState {
    pub turn: i32,
    pub level: i32,
    pub wait_for_end: bool,
    pub bomb_away: bool,
}

pub struct Player {
    pub username: String,
    pub is_alive: bool,
    pub score: i32,
    pub pos_x: i32,
    pub pos_y: i32,
    pub safe_teleports: i32,
    pub invincible: bool,
    pub bombs: i32,
}

pub struct Item {
    pub pos_x: i32,
    pub pos_y: i32,
    pub level: i32,
    pub kind: i32,
    pub visible: bool,
    pub picked_up: bool,
}

#[derive(Clone)]
pub struct DumbRobot {
    pub pos_x: i32,
    pub pos_y: i32,
    pub is_scrap: bool,
    pub kind: i32,
}

#[derive(Clone)]
pub struct JunkHeap {
    pub pos_x: i32,
    pub pos_y: i32,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Username
    #[arg(short, long, default_value = "show_highscore")]
    pub username: String,

    /// Path to highscore file
    #[arg(short, long, default_value = "highscore.txt")]
    pub path: String,

    /// Show highscore
    #[arg(short, long)]
    pub show_highscore: bool,
}
