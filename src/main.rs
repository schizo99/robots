use std::fs::OpenOptions;
use std::io::prelude::*;
use clap::{CommandFactory, Parser};
use rand::Rng;
use console::Term;
use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

const PADDING_LEFT: i32 = 3;
const PADDING_TOP: i32 = 1;

// TODO: Use these variables..
const BOARD_WIDTH: i32 = 60;
const BOARD_HEIGHT : i32 = 24;

struct Player {
    username: String,
    is_alive: bool,
    score: i32,
    pos_x: i32,
    pos_y: i32,
    safe_teleports: i32
}

#[derive(Clone)]
struct Dumb_Robot {
    pos_x: i32,
    pos_y: i32,
    is_scrap: bool,
    id: i32,
}

struct Smart_Robot {
    pos_x: i32,
    pos_y: i32
}

struct Killer_Robot {
    pos_x: i32,
    pos_y: i32
}

struct Junk_Heap {
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


fn draw_active_objects(player: &Player, dumb_robots: &Vec<Dumb_Robot>) {    
    let terminal = Term::stdout();

    // Draw the player
    terminal.move_cursor_to(player.pos_x as usize + PADDING_LEFT as usize, player.pos_y as usize + PADDING_TOP as usize).unwrap(); // Adjusted for 0-based indexing
    print!("@");

    for robot in dumb_robots {
        terminal.move_cursor_to(robot.pos_x as usize + PADDING_LEFT as usize, robot.pos_y as usize + PADDING_TOP as usize).unwrap(); // Adjusted for 0-based indexing
        if !robot.is_scrap {
            print!("+");
        }
    }

    terminal.move_cursor_to(0, PADDING_TOP as usize + BOARD_HEIGHT as usize + 3).unwrap(); // Adjusted for 0-based indexing

}

// A very busy redraw function. However. This is the final version!
fn draw_boundaries(player: &Player) {
    let terminal = Term::stdout(); 
    terminal.clear_screen().unwrap();
    
    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize).unwrap();
    println!("Directions:");

    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 2).unwrap();
    println!("y k u");

    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 3).unwrap();
    println!(" \\|/");

    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 4).unwrap();
    println!("h- -l");

    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 5).unwrap();
    println!("/ | \\");

    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 6).unwrap();
    println!("b j n");

    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 8).unwrap();
    println!("Commands:");

    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 10).unwrap();
    println!("w:  wait for end");
   
    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 11).unwrap();
    println!("t:  teleport");
 
    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 12).unwrap();
    println!("s:  safe teleport! ({})", player.safe_teleports);

    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 13).unwrap();
    println!("^L: redraw screen");

    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 14).unwrap();
    println!("q:  quit");
    
    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 16).unwrap();
    println!("Legend:");

    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 18).unwrap();
    println!("+:  robot");

    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 19).unwrap();
    println!("&:  super robot");

    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 20).unwrap();
    println!("N:  killer robot");

    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 21).unwrap();
    println!("*   junk heap");

    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 22).unwrap();
    println!("@:  you");

    terminal.move_cursor_to(PADDING_LEFT as usize + BOARD_WIDTH as usize + 4, PADDING_TOP as usize + 24).unwrap(); 
    println!("Score:  0");
   
    terminal.move_cursor_to(0, 0).unwrap();

    print!("{}", "\n".repeat(PADDING_TOP as usize));
    print!("{}", " ".repeat(PADDING_LEFT as usize)); 
    print!("/");
    print!("{}", "-".repeat(BOARD_WIDTH as usize));
    println!("\\");
    for _ in 0..24 {
        print!("{}", " ".repeat(PADDING_LEFT as usize));
        print!("|");
        print!("{}", " ".repeat(BOARD_WIDTH as usize));
        println!("|");
    }
    print!("{}", " ".repeat(PADDING_LEFT as usize)); 
    print!("\\");
    print!("{}", "-".repeat(BOARD_WIDTH as usize));
    println!("/");


}

fn player_input(player: &mut Player) {
    enable_raw_mode().expect("Failed to enable raw mode");

    match read().expect("Failed to read event") {
        Event::Key(event) => {
            match event.code {
                KeyCode::Char(c) => {
                    match c {
                        'y' => move_player(player, -1, -1), // Move diagonally up and left 
                        'k' => move_player(player, 0, -1),  // Move up
                        'u' => move_player(player, 1, -1),  // Move diagonally up and right
                        'h' => move_player(player, -1, 0),  // Move left
                        'l' => move_player(player, 1, 0),   // Move right,
                        'b' => move_player(player, -1, 1),  // Move diagonally down and left
                        'j' => move_player(player, 0, 1),   // Nove down
                        'n' => move_player(player, 1, 1),   // Move diagonally down and right
                        'q' => player.is_alive = false,
                        _ => (),
                    }
                },
                _ => ()
            }
        },
        _ => {}
    }

    disable_raw_mode().expect("Failed to disable raw mode");    
}

fn move_player(player: &mut Player, d_pos_x: i32, d_pos_y: i32) {
    player.pos_x += d_pos_x;
    player.pos_y += d_pos_y;
    if player.pos_x < 1 {
        player.pos_x = 1;
    }
    if player.pos_x > BOARD_WIDTH {
        player.pos_x = BOARD_WIDTH;
    }
    if player.pos_y < 1 {
        player.pos_y = 1;
    }
    if player.pos_y > BOARD_HEIGHT {
        player.pos_y = BOARD_HEIGHT;
    }
}

fn update_robot_positions(player: &Player, dumb_robots: &mut Vec<Dumb_Robot>) {
    // All dumb_robots should move towards the player in a straight line
    for robot in dumb_robots {
        if robot.pos_x < player.pos_x {
            robot.pos_x += 1;
        } else if robot.pos_x > player.pos_x {
            robot.pos_x -= 1;
        }
        if robot.pos_y < player.pos_y {
            robot.pos_y += 1;
        } else if robot.pos_y > player.pos_y {
            robot.pos_y -= 1;
        }
    }
}

fn evaluate_state(player: &mut Player, dumb_robots: &mut Vec<Dumb_Robot>, junk_heaps: &mut Vec<Junk_Heap>) {
    // Create a copy of the dumb_robots vector to avoid borrowing issues
    let dumb_robots_copy = dumb_robots.clone();


    for robot in dumb_robots {
        if robot.pos_x == player.pos_x && robot.pos_y == player.pos_y {
            player.is_alive = false;
        }

        // Check if two robots are in the same space. If so. Remove it
        for other_robot in &dumb_robots_copy {
            if robot.pos_x == other_robot.pos_x && robot.pos_y == other_robot.pos_y &&
                robot.id != other_robot.id {
                robot.is_scrap = true;
            }
        }

        if robot.is_scrap {
            junk_heaps.push(Junk_Heap { pos_x: robot.pos_x, pos_y: robot.pos_y });        }
    }
}

fn main() {
    // let args = Args::parse();
    // println!("{:?}", args);
    // handle_highscore(&args);

    // We need a playing ground..
    let mut rng = rand::thread_rng();

    let mut dumb_robots: Vec<Dumb_Robot> = Vec::new();
    let mut smart_robots: Vec<Smart_Robot> = Vec::new();
    let mut killer_robots: Vec<Killer_Robot> = Vec::new();
    let mut junk_heaps: Vec<Junk_Heap> = Vec::new();

    let mut player = Player {
        username: "Kalle".to_string(),
        score: 0,
        is_alive: true,
        pos_x: rng.gen_range(1..BOARD_WIDTH),
        pos_y: rng.gen_range(1..BOARD_HEIGHT),
        safe_teleports: 3,
    };

    dumb_robots.push(Dumb_Robot { pos_x: rng.gen_range(1..BOARD_WIDTH), pos_y: rng.gen_range(1..BOARD_HEIGHT), is_scrap: false, id: 1 });
    dumb_robots.push(Dumb_Robot { pos_x: rng.gen_range(1..BOARD_WIDTH), pos_y: rng.gen_range(1..BOARD_HEIGHT), is_scrap: false, id: 2 });

    while player.is_alive {
        draw_boundaries(&player);
        draw_active_objects(&player, &dumb_robots);
        player_input(&mut player);
        update_robot_positions(&player, &mut dumb_robots);
        evaluate_state(&mut player, &mut dumb_robots, &mut junk_heaps);
    }

    draw_boundaries(&player);
    draw_active_objects(&player, &dumb_robots);

    // add_dummy_score(&args.username, &args.path)
}
