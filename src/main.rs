use std::fs::OpenOptions;
use std::io::{self, prelude::*};
use clap::{CommandFactory, Parser};
use rand::Rng;
use crossterm::{
    execute,
    event::{read, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    cursor::{Hide, Show, MoveTo}
};

const PADDING_LEFT: i32 = 3;
const PADDING_TOP: i32 = 1;

// TODO: Use these variables..
const BOARD_WIDTH: i32 = 60;
const BOARD_HEIGHT : i32 = 24;

struct Game_State {
    score: i32,
    turn: i32,
    level: i32,
}

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
    kind: i32,
    id: i32,
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
    if username == "show_highscore" && !args.show_highscore {
        Args::command().print_help().unwrap();
        std::process::exit(0);
    }
    let path = &args.path;
    validate_highscore_file(&path);
    if args.show_highscore {
        let content = top_highscores(&path).join("\n");
        execute!(io::stdout(), Clear(ClearType::All)).expect("Failed to clear screen");
        execute!(io::stdout(), MoveTo(0, 0)).expect("Failed to move cursor");
        execute!(io::stdout(), Hide).expect("Failed to hide cursor");
        println!("{}\n<More>", &content);
        execute!(io::stdout(), Show).expect("Failed to show cursor");
        enable_raw_mode().expect("Failed to enable raw mode");
        read().expect("Failed to read event");
        disable_raw_mode().expect("Failed to disable raw mode");
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


fn draw_active_objects(player: &Player, dumb_robots: &Vec<Dumb_Robot>, junk_heaps: &Vec<Junk_Heap>) {
    // Draw the player
    execute!(io::stdout(), MoveTo(player.pos_x as u16 + PADDING_LEFT as u16, player.pos_y as u16 + PADDING_TOP as u16)).unwrap();
    print!("@");

    // Draw the robots
    for robot in dumb_robots {
        execute!(io::stdout(), MoveTo(robot.pos_x as u16 + PADDING_LEFT as u16, robot.pos_y as u16 + PADDING_TOP as u16)).unwrap();
        if !robot.is_scrap {
            print!("+");
        }
    }

    // Draw the junk heaps
    for junk in junk_heaps {
        execute!(io::stdout(), MoveTo(junk.pos_x as u16 + PADDING_LEFT as u16, junk.pos_y as u16 + PADDING_TOP as u16)).unwrap();
        print!("#");
    }

    execute!(io::stdout(), MoveTo(0, PADDING_TOP as u16 + BOARD_HEIGHT as u16 + 3)).unwrap();

}

// A very busy redraw function. However. This is the final version!
fn draw_boundaries(player: &Player) {
    execute!(io::stdout(), Clear(ClearType::All)).expect("Failed to clear screen");
    execute!(io::stdout(), MoveTo(0, 0)).expect("Failed to move cursor");
    let menu = vec![
        "Directions:",
        "",
        "y k u",
        " \\|/",
        "h- -l",
        " /|\\",
        "b j n",
        "",
        "Commands:",
        "",
        "w:  wait for end",
        "t:  teleport",
        "s:  safe teleport! (3)",
        "^L: redraw screen",
        "q:  quit",
        "",
        "Legend:",
        "",
        "+:  robot",
        "&:  super robot",
        "N:  killer robot",
        "*   junk heap",
        "@:  you",
        "",
        "Score:  0",
    ];
    for ((line, i)) in menu.iter().zip(0..) {
        execute!(io::stdout(), MoveTo(PADDING_LEFT as u16 + BOARD_WIDTH as u16 + 4, PADDING_TOP as u16 + i)).expect("Failed to move cursor");
        print!("{}", line);
    }

    execute!(io::stdout(), MoveTo(0, 0)).expect("Failed to move cursor");

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

fn player_input(player: &mut Player, robots: &Vec<Dumb_Robot>) {
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
                        't' => teleport_player(player, robots.clone()),
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
        if !robot.is_scrap {
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

fn teleport_player(player: &mut Player, dumb_robots: Vec<Dumb_Robot>) {

    let mut safe_teleport = false;
    // Check if the player has any safe teleports left
    if player.safe_teleports > 0 {
        safe_teleport = true;
        player.safe_teleports -= 1;
    }

    let mut rng = rand::thread_rng();
    let mut new_x = rng.gen_range(1..BOARD_WIDTH);
    let mut new_y = rng.gen_range(1..BOARD_HEIGHT);

    // Is this a safe teleport?
    if safe_teleport {
        let mut safe_location = false;
        while !safe_location {
            safe_location = true;
            // Check that the distance to a robot to a safe spot is at least 2
            for robot in &dumb_robots {
                if (robot.pos_x - new_x).abs() < 2 && (robot.pos_y - new_y).abs() < 2 {
                    safe_location = false;
                }
            }
            if !safe_location {
                new_x = rng.gen_range(1..BOARD_WIDTH);
                new_y = rng.gen_range(1..BOARD_HEIGHT);
            }
        }
        player.pos_x = new_x;
        player.pos_y = new_y;
    }
    else {
        player.pos_x = new_x;
        player.pos_y = new_y;
    }
 

}

fn validate_board(player: &mut Player, dumb_robots: &mut Vec<Dumb_Robot>) {

}

fn main() {
    // let args = Args::parse();
    // println!("{:?}", args);
    // handle_highscore(&args);

    // We need a playing ground..
    let mut rng = rand::thread_rng();

    let mut dumb_robots: Vec<Dumb_Robot> = Vec::new();
    let mut junk_heaps: Vec<Junk_Heap> = Vec::new();

    let mut player = Player {
        username: "Kalle".to_string(),
        score: 0,
        is_alive: true,
        pos_x: rng.gen_range(1..BOARD_WIDTH),
        pos_y: rng.gen_range(1..BOARD_HEIGHT),
        safe_teleports: 3,
    };

    let mut game_state = Game_State {
        score: 0,
        turn: 0,
        level: 1,
    };

    // Add dumb robots. 20 per level (Move to other function)
    for i in 0 .. game_state.level * 20 {

        let mut occupied = true;
        let mut p_x = 0;
        let mut p_y = 0;

        while occupied {
            p_x = rng.gen_range(1..BOARD_WIDTH);
            p_y = rng.gen_range(1..BOARD_HEIGHT);

            occupied = false;
            for robot in &dumb_robots {
                if robot.pos_x == p_x && robot.pos_y == p_y {
                    occupied = true;
                }
            }
        }

        dumb_robots.push(Dumb_Robot { pos_x: p_x, pos_y: p_y, is_scrap: false, id: i, kind: 1 });
    }

    // Validate the board making sure that the setup is correct
    validate_board(&mut player, &mut dumb_robots);

    //dumb_robots.push(Dumb_Robot { pos_x: rng.gen_range(1..BOARD_WIDTH), pos_y: rng.gen_range(1..BOARD_HEIGHT), is_scrap: false, id: 1 });
    // dumb_robots.push(Dumb_Robot { pos_x: rng.gen_range(1..BOARD_WIDTH), pos_y: rng.gen_range(1..BOARD_HEIGHT), is_scrap: false, id: 2 });

    while player.is_alive {
        draw_boundaries(&player);
        draw_active_objects(&player, &dumb_robots, &junk_heaps);
        player_input(&mut player, &dumb_robots);
        update_robot_positions(&player, &mut dumb_robots);
        evaluate_state(&mut player, &mut dumb_robots, &mut junk_heaps);
    }

    draw_boundaries(&player);
    draw_active_objects(&player, &dumb_robots, &junk_heaps);

    // add_dummy_score(&args.username, &args.path)
}
