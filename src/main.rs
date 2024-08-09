use std::fs::OpenOptions;
use std::io::prelude::*;
use clap::{CommandFactory, Parser};
use rand::Rng;
use console::Term;

const PADDING_LEFT: i32 = 3;
const PADDING_TOP: i32 = 1;

struct Player {
    username: String,
    score: i32,
    pos_x: i32,
    pos_y: i32,
    safe_teleports: i32
}

struct Dumb_Robot {
    pos_x: i32,
    pos_y: i32
}

struct Smart_Robot {
    pos_x: i32,
    pos_y: i32
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Username
    #[arg(short, long, default_value = "show_highscore")]
    username: String,

    /// Path to highscore file
    #[arg(short, long, default_value = "highscore.txt")]
    path: String,

    /// Show highscore
    #[arg(short, long)]
    show_highscore: bool,

}

fn validate_highscore_file(path: &str) {
    println!("Validating highscore file at path: {}", path);
    match std::fs::read_to_string(path) {
        Ok(_) => {
            return;
        }
        Err(_) => {
            match std::fs::write(path, "") {
                Ok(_) => {
                    println!("Highscorefile created successfully");
                }
                Err(err) => {
                    eprintln!("Error creating highscore file {}: {}", path, err);
                }
            }
        }
    }
}

// fn add_dummy_score(username: &str, path: &str) {
//     let mut file = OpenOptions::new()
//         .append(true)
//         .open(path)
//         .unwrap();

//     println!("Adding dummy score for user: {}", username);
//     let score = rand::thread_rng().gen_range(0..100);
//     let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
//     if let Err(e) = writeln!(file, "{};{};{}", username, score, timestamp) {
//         eprintln!("Couldn't write to file: {}", e);
//     }
// }

fn handle_highscore(args: &Args) {
    let username = &args.username;
    if username == "show_highscore" {
        Args::command().print_help().unwrap();
        std::process::exit(0);
    }
    let path = &args.path;
    validate_highscore_file(&path);
    if args.show_highscore {
        let content = top_highscores(&path).join("\n");
        let term = Term::stdout();
        term.clear_screen().expect("Failed to clear terminal");
        term.hide_cursor().expect("Failed to hide cursor");
        term.write_line(&content).expect("Failed to write to terminal");
        term.write_line("<More>").expect("Failed to write to terminal");
        term.read_key().expect("Failed to read key");
        term.show_cursor().expect("Failed to show cursor");
        std::process::exit(0);
    }
}

fn top_highscores(path: &str) -> Vec<String> {
    let mut highscores: Vec<(String, i32)> = Vec::new();
    let content = std::fs::read_to_string(path).unwrap();
    for line in content.lines() {
        let parts: Vec<&str> = line.split(";").collect();
        if parts.len() == 3 {
            let username = parts[0];
            let score = parts[1].parse::<i32>().unwrap();
            highscores.push((username.to_string(), score));
        }
    }
    highscores.sort_by(|a, b| b.1.cmp(&a.1));
    let mut result = vec![];
    result.push(format!("Top 10 highscores:"));
    for (i, (username, score)) in highscores.iter().take(10).enumerate() {
        result.push(format!("{}. {}: {}", i + 1, username, score));
    }
    return result;
}

fn draw_level() {
    // let term = Term::stdout();
    let mut safe_teleports: i32 = 3;
    
    // First just draw the boundaries of the level
    draw_boundaries(safe_teleports);

}

// A very busy redraw function. However. This is the final version!
fn draw_boundaries(safe_teleports: i32) {
    let terminal = Term::stdout(); 
    terminal.clear_screen().unwrap();
    
    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize).unwrap(); // Adjusted for 0-based indexing
    println!("Directions:");

    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 2).unwrap(); // Adjusted for 0-based indexing
    println!("y k u");

    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 3).unwrap(); // Adjusted for 0-based indexing
    println!(" \\|/");

    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 4).unwrap(); // Adjusted for 0-based indexing
    println!("h- -l");

    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 5).unwrap(); // Adjusted for 0-based indexing
    println!("/ | \\");

    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 6).unwrap(); // Adjusted for 0-based indexing
    println!("b j n");

    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 8).unwrap(); // Adjusted for 0-based indexing
    println!("Commands:");

    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 10).unwrap(); // Adjusted for 0-based indexing
    println!("w:  wait for end");
   
    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 11).unwrap(); // Adjusted for 0-based indexing
    println!("t:  teleport");
 
    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 12).unwrap(); // Adjusted for 0-based indexing
    println!("s:  safe teleport! ({})", safe_teleports);

    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 13).unwrap(); // Adjusted for 0-based indexing
    println!("^L: redraw screen");

    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 14).unwrap(); // Adjusted for 0-based indexing
    println!("q:  quit");
    
    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 16).unwrap(); // Adjusted for 0-based indexing
    println!("Legend:");

    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 18).unwrap(); // Adjusted for 0-based indexing
    println!("+:  robot");

    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 19).unwrap(); // Adjusted for 0-based indexing
    println!("&:  super robot");

    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 20).unwrap(); // Adjusted for 0-based indexing
    println!("N:  killer robot");

    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 21).unwrap(); // Adjusted for 0-based indexing
    println!("*   junk heap");

    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 22).unwrap(); // Adjusted for 0-based indexing
    println!("@:  you");

    terminal.move_cursor_to(PADDING_LEFT as usize + 66, PADDING_TOP as usize + 24).unwrap(); // Adjusted for 0-based indexing
    println!("Score:  0");
   
    terminal.move_cursor_to(0, 0).unwrap(); // Adjusted for 0-based indexing

    print!("{}", "\n".repeat(PADDING_TOP as usize));
    print!("{}", " ".repeat(PADDING_LEFT as usize)); 
    println!("/--------------------------------------------------------------\\");
    for _ in 0..24 {
        print!("{}", " ".repeat(PADDING_LEFT as usize));    
        println!("|..............................................................|");
    }
    print!("{}", " ".repeat(PADDING_LEFT as usize)); 
    println!("\\--------------------------------------------------------------/");
}

fn main() {
    let args = Args::parse();
    // println!("{:?}", args);
    // handle_highscore(&args);
    draw_level();
    // add_dummy_score(&args.username, &args.path)
}
