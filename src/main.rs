use std::fs::OpenOptions;
use std::io::prelude::*;
use clap::{CommandFactory, Parser};
use rand::Rng;
use console::Term;

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

fn add_dummy_score(username: &str, path: &str) {
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .unwrap();

    println!("Adding dummy score for user: {}", username);
    let score = rand::thread_rng().gen_range(0..100);
    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    if let Err(e) = writeln!(file, "{};{};{}", username, score, timestamp) {
        eprintln!("Couldn't write to file: {}", e);
    }
}

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
    let term = Term::stdout();
    term.clear_screen().expect("Failed to clear terminal");
    term.hide_cursor().expect("Failed to hide cursor");
    term.write_line("Level 1").expect("Failed to write to terminal");
    let     box_top = format!("{:#^60}", "");
    let     box_side = format!("#{: ^58}#", "");
    term.write_line(&box_top).expect("Failed to write to terminal");
    for _ in 0..20 {
        term.write_line(&box_side).expect("Failed to write to terminal");
    }
    term.write_line(&box_top).expect("Failed to write to terminal");
    term.show_cursor().expect("Failed to show cursor");
}


fn main() {
    let args = Args::parse();
    println!("{:?}", args);
    handle_highscore(&args);
    draw_level();
    add_dummy_score(&args.username, &args.path)
}
