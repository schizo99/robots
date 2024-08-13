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

const BOARD_WIDTH: i32 = 60;
const BOARD_HEIGHT : i32 = 24;

struct Game_State {
    turn: i32,
    level: i32,
    wait_for_end: bool,
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

#[derive(Clone)]
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

fn quit_or_die() {
    // Just a clean up function
    execute!(io::stdout(), Show).unwrap();
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

    // Draw the player
    execute!(io::stdout(), MoveTo(player.pos_x as u16 + PADDING_LEFT as u16, player.pos_y as u16 + PADDING_TOP as u16)).unwrap();
    if player.is_alive { print!("@") } else { print!("%"); }

    // Draw the score
    execute!(io::stdout(), MoveTo(BOARD_WIDTH as u16 + 7, PADDING_TOP as u16 + BOARD_HEIGHT as u16)).unwrap();
    print!("Score: {}    ", player.score);

    // Draw safe teleports
    execute!(io::stdout(), MoveTo(BOARD_WIDTH as u16 + 11, PADDING_TOP as u16 + 12)).unwrap();
    print!("safe teleport! ({})    ", player.safe_teleports);

    execute!(io::stdout(), MoveTo(BOARD_WIDTH as u16 + 4, BOARD_HEIGHT as u16 + 4)).unwrap();
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
        "t:  teleport (unsafe)",
        "s:  safe teleport! (3)",
        ".:  wait one turn",
        "q:  quit",
        "",
        "Legend:",
        "",
        "+:  robot",
        "&:  super robot",
        "N:  killer robot",
        "#   junk heap",
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

fn player_input(player: &mut Player, robots: &Vec<Dumb_Robot>, game_state: &mut Game_State) -> bool {
    enable_raw_mode().expect("Failed to enable raw mode");

    let mut legal_move = false;

    match read().expect("Failed to read event") {
        Event::Key(event) => {
            match event.code {
                KeyCode::Char(c) => {
                    match c {
                        'y' => { move_player(player, -1, -1); legal_move = true; },     // Move diagonally up and left 
                        'k' => { move_player(player, 0, -1); legal_move = true; },      // Move up
                        'u' => { move_player(player, 1, -1); legal_move = true; },      // Move diagonally up and right
                        'h' => { move_player(player, -1, 0); legal_move = true; },      // Move left
                        'l' => { move_player(player, 1, 0); legal_move = true; },       // Move right,
                        'b' => { move_player(player, -1, 1); legal_move = true; },      // Move diagonally down and left
                        'j' => { move_player(player, 0, 1); legal_move = true; },       // Nove down
                        'n' => { move_player(player, 1, 1); legal_move = true; },       // Move diagonally down and right
                        'q' => { player.is_alive = false; legal_move = false; },        // Quit the game (needs function)
                        's' => { teleport_player(true, player, robots.clone()); legal_move = true; },   // Safe teleport
                        't' => { teleport_player(false, player, robots.clone()); legal_move = true; }   // Teleport
                        'w' => { game_state.wait_for_end = true; legal_move = true; },  // Wait until robots are gone, or player is dead
                        '.' => legal_move = true,   // Wait
                        _ => legal_move = false     // Do nothing
                    }
                },
                _ => ()
            }
        },
        _ => {}
    }

    disable_raw_mode().expect("Failed to disable raw mode");    
    legal_move
}

fn move_player(player: &mut Player, d_pos_x: i32, d_pos_y: i32) {
    player.pos_x += d_pos_x;
    player.pos_y += d_pos_y;
    if player.pos_x < 1 {
        player.pos_x = 1;
    }
    if player.pos_x >= BOARD_WIDTH {
        player.pos_x = BOARD_WIDTH;
    }
    if player.pos_y < 1 {
        player.pos_y = 1;
    }
    if player.pos_y >= BOARD_HEIGHT {
        player.pos_y = BOARD_HEIGHT;
    }
}

fn game_tick(player: &mut Player, dumb_robots: &mut Vec<Dumb_Robot>, junk_heaps: &mut Vec<Junk_Heap>, game_board_data: &mut Vec<Vec<i32>>) {

    // Clear the game board..
    game_board_data.iter_mut().for_each(|row| row.iter_mut().for_each(|cell| *cell = 0));

    // Add the junk heaps to the board as 2
    for junk in junk_heaps.clone() {
        game_board_data[junk.pos_y as usize - 1][junk.pos_x as usize - 1] = 2;
    }

    // All dumb_robots should move towards the player in a straight line
    for robot in dumb_robots {
        if !robot.is_scrap {
            // First just make sure that this robot is not standing on a junk pile
            if game_board_data[robot.pos_y as usize - 1][robot.pos_x as usize - 1] == 2 {
                robot.is_scrap = true;
                player.score += 1;
                continue;
            }

            if robot.pos_x < player.pos_x {
                robot.pos_x += 1;
            } else if robot.pos_x > player.pos_x {
                robot.pos_x -= 1;
            }
            if robot.pos_y < player.pos_y {
                robot.pos_y += 1;
            } else if robot.pos_y > player.pos_y {
                robot.pos_y -= 1;
            } else if robot.pos_y == player.pos_y && robot.pos_x == player.pos_x {
                player.is_alive = false;
            }

            // Add this robot to the game_board if it is a free slot, otherwise turn into scrap
            if game_board_data[robot.pos_y as usize - 1][robot.pos_x as usize - 1] == 0 {
                game_board_data[robot.pos_y as usize - 1][robot.pos_x as usize - 1] = 1;
            }
            else if game_board_data[robot.pos_y as usize - 1][robot.pos_x as usize - 1] == 1 {
                robot.is_scrap = true;
                player.score += 2;

                // Add a junk heap the heaps array
                junk_heaps.push(Junk_Heap { pos_x: robot.pos_x, pos_y: robot.pos_y });
            }
            else if game_board_data[robot.pos_y as usize - 1][robot.pos_x as usize - 1] == 2 {
                robot.is_scrap = true;
                player.score += 1;
            }
        }
    }
}

fn teleport_player(try_safe: bool, player: &mut Player, dumb_robots: Vec<Dumb_Robot>) {

    let mut safe_teleport = false;
    // Check if the player has any safe teleports left
    if player.safe_teleports > 0 && try_safe{
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

fn any_robots_left(robots: &Vec<Dumb_Robot>) -> bool {
    // Iterate over the robots and check whether at least one is alive
    for robot in robots {
        if !robot.is_scrap {
            return true;
        }
    }

    false
}

fn main() {
    execute!(io::stdout(), Hide).unwrap();

    // let args = Args::parse();
    // println!("{:?}", args);
    // handle_highscore(&args);

    // We need a playing ground..
    let mut rng = rand::thread_rng();

    let mut game_board_data: Vec<Vec<i32>> = vec![vec![0; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];

    let mut dumb_robots: Vec<Dumb_Robot> = Vec::new();
    let mut junk_heaps: Vec<Junk_Heap> = Vec::new();

    // Basic setup
    let mut game_state = Game_State {
        turn: 0,
        level: 1,
        wait_for_end: false,
    };

    // Add dumb robots. 20 per level (Move to other function)
    for i in 0 .. game_state.level * 20 {

        let mut occupied = true;
        let mut p_x = 0;
        let mut p_y = 0;

        while occupied {
            p_x = rng.gen_range(1..BOARD_WIDTH);
            p_y = rng.gen_range(1..BOARD_HEIGHT);

            if game_board_data[p_y as usize - 1][p_x as usize - 1] == 0 {
                occupied = false;
            }
        }

        dumb_robots.push(Dumb_Robot { pos_x: p_x, pos_y: p_y, is_scrap: false, id: i, kind: 1 });
        // Add the robot to the game board array
        game_board_data[p_y as usize - 1][p_x as usize - 1] = 1;
    }

    // Setup the player
    let mut found_starting_spot = false;
    let mut p_x = 0;
    let mut p_y = 0;

    while !found_starting_spot {
        p_x = rng.gen_range(1..BOARD_WIDTH);
        p_y = rng.gen_range(1..BOARD_HEIGHT);

        if game_board_data[p_y as usize - 1][p_x as usize - 1] == 0 {
            found_starting_spot = true;
        }
    }

    // Setup the player
    let mut player = Player {
        username: "Kalle".to_string(),
        score: 0,
        is_alive: true,
        pos_x: p_x,
        pos_y: p_y,
        safe_teleports: 3,
    };


    while player.is_alive {
        draw_boundaries(&player);
        draw_active_objects(&player, &dumb_robots, &junk_heaps);
        if !game_state.wait_for_end {
            if player_input(&mut player, &dumb_robots, &mut game_state) {
                game_tick(&mut player, &mut dumb_robots, &mut junk_heaps, &mut game_board_data);
            }
        }
        else {
            game_tick(&mut player, &mut dumb_robots, &mut junk_heaps, &mut game_board_data);
            // Sleep for 75ms
            std::thread::sleep(std::time::Duration::from_millis(75));
        }

        if !any_robots_left(&dumb_robots) {
            game_state.wait_for_end = false;

            // Generate new level....
        }

    }

    draw_boundaries(&player);
    draw_active_objects(&player, &dumb_robots, &junk_heaps);

    // Just to clean up stuff..
    execute!(io::stdout(), Show).unwrap();
    println!("");

    // add_dummy_score(&args.username, &args.path)
}
