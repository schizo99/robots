use crate::structs::*;
use crossterm::{
    cursor::MoveTo,
    event::{read, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
/// This module contains functions related to displaying the game interface and graphics.
///
/// The `display` module provides functions for drawing various game elements on the screen,
/// such as the player, robots, junk heaps, and boundaries. It also includes a splash screen
/// function for displaying the game's introduction and a retry query function for prompting
/// the user to try again.
use std::io::{self, prelude::*};

pub fn move_cursor_padded(x: i32, y: i32) {
    execute!(
        io::stdout(),
        MoveTo(
            x as u16 + PADDING_LEFT as u16,
            y as u16 + PADDING_TOP as u16
        )
    )
    .unwrap();
}

pub fn draw_active_objects(
    player: &Player, dumb_robots: &Vec<DumbRobot>, junk_heaps: &Vec<JunkHeap>, item: &Item,
) {
    // Draw the item, if it is visible and not picked up
    if item.visible && !item.picked_up {
        move_cursor_padded(item.pos_x, item.pos_y);
        if item.kind == 1 {
            print!("S");
        } else if item.kind == 2 {
            print!("B");
        }
    }
    // Draw the player
    move_cursor_padded(player.pos_x, player.pos_y);
    print!("@");

    // Draw the robots
    for robot in dumb_robots {
        move_cursor_padded(robot.pos_x, robot.pos_y);
        if !robot.is_scrap {
            // Separate the robots by kind
            if robot.kind == 1 {
                print!("+");
            } else if robot.kind == 2 {
                print!("&");
            } else if robot.kind == 3 {
                print!("N");
            }
        }
    }

    // Draw the junk heaps
    for junk in junk_heaps {
        move_cursor_padded(junk.pos_x, junk.pos_y);
        print!("#");
    }

    // Draw the player
    move_cursor_padded(player.pos_x, player.pos_y);
    if player.is_alive {
        print!("@")
    } else {
        print!("%");
    }

    execute!(
        io::stdout(),
        MoveTo(BOARD_WIDTH as u16 + 4, BOARD_HEIGHT as u16 + 4)
    )
    .unwrap();
}

fn alive_robots(robots: &Vec<DumbRobot>) -> i32 {
    let mut alive = 0;
    for robot in robots {
        if !robot.is_scrap {
            alive += 1;
        }
    }
    alive
}

// A very busy redraw function. However. This is the final version!
pub fn draw_boundaries(
    player: &Player, gamestate: &GameState, junk_heaps: &Vec<JunkHeap>,
    dumb_robots: &Vec<DumbRobot>,
) {
    execute!(io::stdout(), Clear(ClearType::All)).expect("Failed to clear screen");
    execute!(io::stdout(), MoveTo(0, 0)).expect("Failed to move cursor");
    let score_str = format!("Score:  {}", player.score);
    let safe_teleports_str = format!("s:  safe teleport ({})", player.safe_teleports);
    let level_str = format!("Level:  {}", gamestate.level);
    let bomb_str = format!("a:  bomb ({})", player.bombs);
    let player_str;
    if player.invincible {
        player_str = format!("@:  you (invincible)");
    } else {
        player_str = format!("@:  you");
    }
    let alive_robots_str = format!("Robots:  {}", alive_robots(dumb_robots));
    let junk_piles_str = format!("Junk piles:  {}", junk_heaps.len());
    let menu = vec![
        "Directions:  y k u",
        "              \\|/",
        "             h- -l",
        "              /|\\",
        "             b j n",
        "Commands:",
        "",
        "w:  wait for end",
        "t:  teleport (unsafe)",
        safe_teleports_str.as_str(),
        bomb_str.as_str(),
        ".:  wait one turn",
        "q:  quit",
        "",
        "Legend:",
        "",
        "+:  robot",
        "&:  super robot",
        "N:  killer robot",
        "#:  junk heap",
        player_str.as_str(),
        "",
        "",
        "",
        score_str.as_str(),
    ];
    for (line, i) in menu.iter().zip(0..) {
        move_cursor_padded(BOARD_WIDTH + 4, i);
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
    println!(
        "\t{}\t  {}\t{}",
        level_str, junk_piles_str, alive_robots_str
    );
}

pub fn splash_screen() {
    let splash_screen = vec![
        "/------------------------------------\\", 
        "|                                    |   Welcome to the game!",
        "|               ROBOTS               |",
        "|                                    |   The game is simple. You are the player, represented by the @ symbol",
        "|      .--.      .--.      .--.      |   You are surrounded by robots, represented by +, &, and N.",
        "|     /    \\    /    \\    /    \\     |   The robots will try to catch you. If they do, you lose.",
        "|    |  []  |  |  []  |  |  []  |    |",
        "|    |      |  |      |  |      |    |   You can move in the following directions:",
        "|    |______|  |______|  |______|    |",
        "|                                    |   y k u",
        "|        .----.       / \\            |    \\|/          (You can also pick up objects, represented by S and B.",
        "|       /      \\     /   \\           |   h- -l         S will make you invincible for a short (?) period of time.",
        "|      |  O  O  |   |  O  |          |    /|\\          B will give you an extra bomb.)",
        "|      |   \\/   |   |     |          |   b j n",
        "|       \\      /     \\___/           |",
        "|        `----'                      |   You can teleport (t), safe teleport (s) if charged,",
        "|                                    |   use any of your bombs (a), wait for the level end (w),",
        "|                                    |   or quit the game (q).",
        "\\---------- ASCII art by: Chat-GPT --/",
        ];

    // Print the vector with padding...

    // Clear the screen
    execute!(io::stdout(), Clear(ClearType::All)).unwrap();

    // Set the cursor to the top left corner
    execute!(io::stdout(), MoveTo(0, 0)).unwrap();

    print!("{}", "\n".repeat(PADDING_TOP as usize));

    for (line, _) in splash_screen.iter().zip(0..) {
        print!("{}", " ".repeat(PADDING_LEFT as usize));
        println!("{}", line);
    }

    // Sleep for 5000ms
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Set the cursor to the top left corner
    move_cursor_padded(74, 18);

    println!(" (Press any key to continue...)");

    // Wait for any key input
    enable_raw_mode().expect("Failed to enable raw mode");
    let _ = read().expect("Failed to read event");
    disable_raw_mode().expect("Failed to disable raw mode");
}

// Retry function takes either a y/n input and returns a boolean
pub fn retry_query() -> bool {
    execute!(
        io::stdout(),
        MoveTo(
            BOARD_WIDTH as u16 + 7,
            PADDING_TOP as u16 + BOARD_HEIGHT as u16 - 2
        )
    )
    .unwrap();
    print!("Do you want to try again? (y/n) ");

    // Sleep for a 1000ms

    let mut try_again = false;
    io::stdout().flush().unwrap();

    enable_raw_mode().expect("Failed to enable raw mode");

    match read().expect("Failed to read event") {
        Event::Key(event) => {
            match event.code {
                KeyCode::Char(c) => {
                    match c {
                        'y' => try_again = true,  // We want to retry
                        'Y' => try_again = true, // We want to retry (should caps lock be initiated)
                        'n' => try_again = false, // We do not want to retry
                        'N' => try_again = false, // We do not want to retry (should caps lock be initiated)
                        _ => {
                            try_again = false;
                            retry_query();
                        }
                    }
                }
                _ => (),
            }
        }
        _ => try_again = false,
    }

    disable_raw_mode().expect("Failed to disable raw mode");

    execute!(
        io::stdout(),
        MoveTo(BOARD_WIDTH as u16 + 4, BOARD_HEIGHT as u16 + 4)
    )
    .unwrap();

    try_again
}
