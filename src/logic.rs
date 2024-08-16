use crate::display::*;
use crate::highscore::*;
/// This module contains the logic for the game.
///
/// It includes functions for player input, moving the player, handling game ticks,
/// and updating the game state based on player actions.
/// The module also includes a function for quitting the game and a function for
/// handling the bombing action.
///
/// The `player_input` function enables raw mode for reading player input events.
/// It processes the key events and updates the player's position accordingly.
/// The function returns a tuple indicating whether the move was legal and if the player wants to quit the game.
///
/// The `move_player` function updates the player's position based on the given direction.
/// It checks if the new position is within the game board boundaries and if it is a valid move.
/// If the move is not valid, the function returns false.
///
/// The `game_tick` function handles the game tick, which is called periodically.
/// It clears the game board, adds junk heaps to the board, and checks if the player wants to bomb.
/// If the player wants to bomb, it calculates the bomb coordinates and updates the game state accordingly.
/// The function also moves the dumb robots towards the player and handles collisions with junk heaps and the player.
/// It updates the game board and the player's score accordingly.
use crate::structs::*;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use rand::Rng;
use std::io;

fn quit_now() {
    // Just a clean up function
    execute!(io::stdout(), Show).unwrap();
    // Exit the application
    println!("");
    std::process::exit(0);
}

fn player_input(
    player: &mut Player, robots: &Vec<DumbRobot>, gamestate: &mut GameState,
    game_board_data: &Vec<Vec<i32>>,
) -> (bool, bool) {
    enable_raw_mode().expect("Failed to enable raw mode");

    let mut legal_move = false;
    let mut quit = false;

    match read().expect("Failed to read event") {
        Event::Key(event) => {
            match event.code {
                KeyCode::Char(c) => {
                    match c {
                        'y' => legal_move = move_player(player, -1, -1, game_board_data), // Move diagonally up and left
                        'k' => legal_move = move_player(player, 0, -1, game_board_data),  // Move up
                        'u' => legal_move = move_player(player, 1, -1, game_board_data), // Move diagonally up and right
                        'h' => legal_move = move_player(player, -1, 0, game_board_data), // Move left
                        'l' => legal_move = move_player(player, 1, 0, game_board_data), // Move right,
                        'b' => legal_move = move_player(player, -1, 1, game_board_data), // Move diagonally down and left
                        'j' => legal_move = move_player(player, 0, 1, game_board_data), // Nove down
                        'n' => legal_move = move_player(player, 1, 1, game_board_data), // Move diagonally down and right
                        'q' => {
                            player.is_alive = false;
                            legal_move = false;
                            quit = true;
                        } // Quit the game (needs function)
                        's' => {
                            teleport_player(true, player, robots.clone());
                            legal_move = true;
                        } // Safe teleport
                        't' => {
                            teleport_player(false, player, robots.clone());
                            legal_move = true;
                        } // Teleport
                        'a' => {
                            gamestate.bomb_away = true;
                            legal_move = true;
                        } // Bomb
                        'w' => {
                            gamestate.wait_for_end = true;
                            legal_move = true;
                        } // Wait until robots are gone, or player is dead
                        '.' => legal_move = true,                                       // Wait
                        _ => legal_move = false, // Do nothing
                    }
                }
                _ => (),
            }
        }
        _ => {}
    }

    disable_raw_mode().expect("Failed to disable raw mode");
    gamestate.turn += 1;
    (legal_move, quit)
}

fn move_player(
    player: &mut Player, d_pos_x: i32, d_pos_y: i32, game_board_data: &Vec<Vec<i32>>,
) -> bool {
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

    if game_board_data[player.pos_y as usize - 1][player.pos_x as usize - 1] != 0 {
        player.pos_x -= d_pos_x;
        player.pos_y -= d_pos_y;
        return false;
    }

    true
}

fn game_tick(
    player: &mut Player, dumb_robots: &mut Vec<DumbRobot>, junk_heaps: &mut Vec<JunkHeap>,
    game_board_data: &mut Vec<Vec<i32>>, item: &mut Item, game_state: &mut GameState,
) {
    // Clear the game board..
    game_board_data
        .iter_mut()
        .for_each(|row| row.iter_mut().for_each(|cell| *cell = 0));

    // Add the junk heaps to the board as 2
    for junk in junk_heaps.clone() {
        game_board_data[junk.pos_y as usize - 1][junk.pos_x as usize - 1] = 2;
    }

    // Check if we should bomb away
    if game_state.bomb_away {
        game_state.bomb_away = false;
        // Here we should create a vector with all coordinates based on the player position in the following form
        // ..B..   Space two steps up
        // .BBB.   Space diagonal up and left, up and right, and up
        // BB@BB   Two spaces left, one space left, two spaces right, one space right
        // .BBB.   Space diagonal down and left, down and right, and down
        // ..B..   Space two steps down
        let bomb_coordinates = vec![
            (player.pos_x, player.pos_y - 2),
            (player.pos_x - 1, player.pos_y - 1),
            (player.pos_x + 1, player.pos_y - 1),
            (player.pos_x, player.pos_y - 1),
            (player.pos_x - 2, player.pos_y),
            (player.pos_x - 1, player.pos_y),
            (player.pos_x + 1, player.pos_y),
            (player.pos_x + 2, player.pos_y),
            (player.pos_x - 1, player.pos_y + 1),
            (player.pos_x + 1, player.pos_y + 1),
            (player.pos_x, player.pos_y + 1),
            (player.pos_x, player.pos_y + 2),
        ];

        // For all these coordinates, we should move to the position, as long as is in the playing field
        // and draw a {
        for coordinate in bomb_coordinates {
            if coordinate.0 > 0
                && coordinate.0 <= BOARD_WIDTH
                && coordinate.1 > 0
                && coordinate.1 <= BOARD_HEIGHT
            {
                // Check if there is a robot at this position
                for robot in &mut *dumb_robots {
                    if robot.pos_x == coordinate.0 && robot.pos_y == coordinate.1 {
                        robot.is_scrap = true;
                        player.score += 1;

                        // Add a junk heap the heaps array
                        junk_heaps.push(JunkHeap {
                            pos_x: robot.pos_x,
                            pos_y: robot.pos_y,
                        });
                    }
                }

                // Move the cursor to the specific position and print a {
                move_cursor_padded(coordinate.0, coordinate.1);
                println!("{{");
            }
        }
        // Wait for 500 ms
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // All dumb_robots should move towards the player in a straight line
    for robot in &mut *dumb_robots {
        if robot.kind == 1 {
            if !robot.is_scrap {
                // First just make sure that this robot is not standing on a junk pile.
                if game_board_data[robot.pos_y as usize - 1][robot.pos_x as usize - 1] == 2 {
                    robot.is_scrap = true;
                    player.score += 1;
                    continue;
                }

                let old_x = robot.pos_x.clone();
                let old_y = robot.pos_y.clone();

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
                if robot.pos_y == player.pos_y && robot.pos_x == player.pos_x {
                    if player.invincible {
                        //robot.is_scrap = true;
                        robot.pos_x = old_x;
                        robot.pos_y = old_y;
                        junk_heaps.push(JunkHeap {
                            pos_x: robot.pos_x,
                            pos_y: robot.pos_y,
                        });
                        player.score += 1;
                        player.invincible = false;
                    } else {
                        player.is_alive = false;
                    }
                }

                // Add this robot to the game_board if it is a free slot, otherwise turn into scrap
                if game_board_data[robot.pos_y as usize - 1][robot.pos_x as usize - 1] == 0 {
                    game_board_data[robot.pos_y as usize - 1][robot.pos_x as usize - 1] = 1;
                } else if game_board_data[robot.pos_y as usize - 1][robot.pos_x as usize - 1] == 1 {
                    robot.is_scrap = true;
                    player.score += 2;

                    // Add a junk heap the heaps array
                    junk_heaps.push(JunkHeap {
                        pos_x: robot.pos_x,
                        pos_y: robot.pos_y,
                    });
                } else if game_board_data[robot.pos_y as usize - 1][robot.pos_x as usize - 1] == 2 {
                    robot.is_scrap = true;
                    player.score += 1;
                }
            }
        } else if robot.kind == 2 {
            if !robot.is_scrap {
                // First just make sure that this robot is not standing on a junk pile.
                if game_board_data[robot.pos_y as usize - 1][robot.pos_x as usize - 1] == 2 {
                    robot.is_scrap = true;
                    player.score += 1;
                    continue;
                }

                // The horse robot can move two steps forward and one to the side
                let moves: Vec<(i32, i32)> = vec![
                    (2, 1),
                    (2, -1),
                    (-2, 1),
                    (-2, -1),
                    (1, 2),
                    (1, -2),
                    (-1, 2),
                    (-1, -2),
                ];

                // Iterate over the moves vector and calculate the euclidian distance to the player using the euclidian distance
                let mut shortest_distance = 1000;
                let mut shortest_move = (0, 0);

                for candidate_move in moves {
                    let new_x = robot.pos_x + candidate_move.0;
                    let new_y = robot.pos_y + candidate_move.1;

                    // Continue if the new position is outside the board
                    if new_x < 1 || new_x > BOARD_WIDTH || new_y < 1 || new_y > BOARD_HEIGHT {
                        continue;
                    }

                    // Calculate the distance to the player based on the current move. If it is the shortest move, update shortest move
                    let distance = eucledian_distance(
                        player,
                        &DumbRobot {
                            pos_x: new_x,
                            pos_y: new_y,
                            is_scrap: false,
                            kind: 3,
                        },
                    );

                    if distance <= shortest_distance {
                        shortest_move = candidate_move;
                        shortest_distance = distance;
                    }
                }

                // Move the robot to the shortest move
                robot.pos_x += shortest_move.0;
                robot.pos_y += shortest_move.1;

                if robot.pos_y == player.pos_y && robot.pos_x == player.pos_x {
                    if player.invincible {
                        // For horses we will just randomize a direction for x and y where we should put the pile
                        let mut rng = rand::thread_rng();
                        let pile_x = rng.gen_range(-1..2);
                        let pile_y = rng.gen_range(-1..2);
                        junk_heaps.push(JunkHeap {
                            pos_x: player.pos_x + pile_x,
                            pos_y: robot.pos_y + pile_y,
                        });
                        player.score += 1;
                        player.invincible = false;
                    } else {
                        player.is_alive = false;
                    }
                }

                // Add this robot to the game_board if it is a free slot, otherwise turn into scrap
                if game_board_data[robot.pos_y as usize - 1][robot.pos_x as usize - 1] == 0 {
                    game_board_data[robot.pos_y as usize - 1][robot.pos_x as usize - 1] = 1;
                } else if game_board_data[robot.pos_y as usize - 1][robot.pos_x as usize - 1] == 1 {
                    robot.is_scrap = true;
                    player.score += 2;

                    // Add a junk heap the heaps array
                    junk_heaps.push(JunkHeap {
                        pos_x: robot.pos_x,
                        pos_y: robot.pos_y,
                    });
                } else if game_board_data[robot.pos_y as usize - 1][robot.pos_x as usize - 1] == 2 {
                    robot.is_scrap = true;
                    player.score += 1;
                }
            }
        } else if robot.kind == 3 {
            // This robot moves like a queen in chess. It should try to reduce the distance to the player with every move
            if !robot.is_scrap {
                // First just make sure that this robot is not standing on a junk pile.
                if game_board_data[robot.pos_y as usize - 1][robot.pos_x as usize - 1] == 2 {
                    robot.is_scrap = true;
                    player.score += 1;
                    continue;
                }

                let moves: Vec<(i32, i32)> = vec![
                    (0, 1),
                    (0, -1),
                    (1, 0),
                    (-1, 0),
                    (1, 1),
                    (-1, 1),
                    (1, -1),
                    (-1, -1),
                ];

                // Iterate over the moves vector and calculate the euclidian distance to the player using the euclidian distance
                let mut shortest_distance = 1000;
                let mut shortest_move = (0, 0);

                for candidate_move in moves {
                    let new_x = robot.pos_x + candidate_move.0;
                    let new_y = robot.pos_y + candidate_move.1;

                    // Continue if the new position is outside the board
                    if new_x < 1 || new_x > BOARD_WIDTH || new_y < 1 || new_y > BOARD_HEIGHT {
                        continue;
                    }

                    // Calculate the distance to the player based on the current move. If it is the shortest move, update shortest move
                    let distance = eucledian_distance(
                        player,
                        &DumbRobot {
                            pos_x: new_x,
                            pos_y: new_y,
                            is_scrap: false,
                            kind: 3,
                        },
                    );

                    if distance <= shortest_distance {
                        shortest_move = candidate_move;
                        shortest_distance = distance;
                    }
                }

                // We now have a unit vector in which direction to move. Loop this move until you either hit the
                // players x_position and/or y_position
                let mut new_x = robot.pos_x;
                let mut new_y = robot.pos_y;

                let mut keep_moving = true;

                while keep_moving {
                    new_x += shortest_move.0;
                    new_y += shortest_move.1;

                    // Check that new new position is withint the board
                    if new_x < 1 || new_x > BOARD_WIDTH || new_y < 1 || new_y > BOARD_HEIGHT {
                        break;
                    }

                    if new_x == player.pos_x && new_y == player.pos_y {
                        if player.invincible {
                            //robot.is_scrap = true;
                            junk_heaps.push(JunkHeap {
                                pos_x: robot.pos_x,
                                pos_y: robot.pos_y,
                            });
                            player.score += 1;
                            player.invincible = false;
                        } else {
                            robot.pos_x = new_x;
                            robot.pos_y = new_y;
                            player.is_alive = false;
                        }
                    }

                    // Check the board for this position to make sure that it is a free spot
                    if game_board_data[new_y as usize - 1][new_x as usize - 1] == 0 {
                        robot.pos_x = new_x;
                        robot.pos_y = new_y;
                    } else if game_board_data[new_y as usize - 1][new_x as usize - 1] == 1 {
                        robot.is_scrap = true;
                        player.score += 2;

                        // Add a junk heap the heaps array
                        junk_heaps.push(JunkHeap {
                            pos_x: robot.pos_x,
                            pos_y: robot.pos_y,
                        });
                    } else if game_board_data[new_y as usize - 1][new_x as usize - 1] == 2 {
                        robot.is_scrap = true;
                        player.score += 1;
                    }

                    if player.pos_x == robot.pos_x || player.pos_y == robot.pos_y && player.is_alive
                    {
                        keep_moving = false;
                    }
                }
            }
        }
    }

    // See whether we should show the current level item
    // It should be a 2 percent chance of showing the item
    let mut rng = rand::thread_rng();
    let show_item = rng.gen_range(1..100);
    if show_item <= 5 && !item.visible {
        item.visible = true;
    }

    // Also make sure that the player is not standing on a newly created junk pile..
    if game_board_data[player.pos_y as usize - 1][player.pos_x as usize - 1] != 0 {
        player.is_alive = false;
    }

    // Check if the player is standing on the item
    if player.pos_x == item.pos_x && player.pos_y == item.pos_y && item.visible && !item.picked_up {
        item.picked_up = true;
        if item.kind == 1 {
            player.invincible = true;
        } else if item.kind == 2 {
            player.bombs += 1;
        }
    }
}

fn eucledian_distance(player: &Player, robot: &DumbRobot) -> i32 {
    // Calculate the Eucledian distance between the player and the robot
    // ((player.pos_x as f64 - robot.pos_x as f64).powi(2) + (player.pos_y as f64 - robot.pos_y as f64).powi(2)).sqrt() as i32
    (player.pos_x - robot.pos_x).abs() + (player.pos_y - robot.pos_y).abs()
}

fn teleport_player(try_safe: bool, player: &mut Player, dumb_robots: Vec<DumbRobot>) {
    // Because Andreas said so.. We need a prompt to tell people that they are teleporting..

    let mut safe_teleport = false;
    // Check if the player has any safe teleports left
    if player.safe_teleports > 0 && try_safe {
        safe_teleport = true;
        player.safe_teleports -= 1;
    }

    execute!(
        io::stdout(),
        MoveTo(
            BOARD_WIDTH as u16 + 7,
            PADDING_TOP as u16 + BOARD_HEIGHT as u16
        )
    )
    .unwrap();
    if safe_teleport {
        print!("Teleporting (safe)...");
    } else {
        print!("Teleporting...");
    }

    println!("");

    // Sleep for 200ms
    std::thread::sleep(std::time::Duration::from_millis(500));

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
    } else {
        player.pos_x = new_x;
        player.pos_y = new_y;
    }
    enable_raw_mode().expect("Failed to enable raw mode");
}

fn any_robots_left(robots: &Vec<DumbRobot>) -> bool {
    // Iterate over the robots and check whether at least one is alive
    for robot in robots {
        if !robot.is_scrap {
            return true;
        }
    }

    false
}

fn no_of_dumb_robots(level: i32) -> i32 {
    if level < 2 {
        20
    } else {
        let robots = 20 + (level - 2) * 5;
        robots
    }
}

fn no_of_super_robots(level: i32) -> i32 {
    if level < 5 {
        0
    } else {
        let robots = 1 + (level - 4) * 2;
        robots
    }
}

fn no_of_killer_robots(level: i32) -> i32 {
    if level < 9 {
        0
    } else {
        let robots = 1 + ((level - 10) * 2) + 1;
        robots
    }
}

// Generate level
fn generate_level(
    gamestate: &GameState, game_board_data: &mut Vec<Vec<i32>>, dumb_robots: &mut Vec<DumbRobot>,
    junk_heaps: &mut Vec<JunkHeap>, player: &mut Player, item: &mut Item,
) {
    let mut rng = rand::thread_rng();

    // Clear the game board..
    game_board_data
        .iter_mut()
        .for_each(|row| row.iter_mut().for_each(|cell| *cell = 0));

    // Add one additional safe teleport per level
    player.safe_teleports += 1;
    player.invincible = false;

    // Clear the old junk piles vector and the dumb_robots one
    dumb_robots.clear();
    junk_heaps.clear();

    // Update the level drop item
    item.pos_x = rng.gen_range(1..BOARD_WIDTH);
    item.pos_y = rng.gen_range(1..BOARD_HEIGHT);
    item.level = gamestate.level;
    item.kind = rng.gen_range(1..3);
    item.visible = false;
    item.picked_up = false;

    // Add dumb robots.
    for _ in 0..no_of_dumb_robots(gamestate.level) {
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

        dumb_robots.push(DumbRobot {
            pos_x: p_x,
            pos_y: p_y,
            is_scrap: false,
            kind: 1,
        });
        // Add the robot to the game board array
        game_board_data[p_y as usize - 1][p_x as usize - 1] = 1;
    }

    // Add super robots.
    for _ in 0..no_of_super_robots(gamestate.level) {
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

        dumb_robots.push(DumbRobot {
            pos_x: p_x,
            pos_y: p_y,
            is_scrap: false,
            kind: 2,
        });
        // Add the robot to the game board array
        game_board_data[p_y as usize - 1][p_x as usize - 1] = 1;
    }

    // Add killer robots.
    for _ in 0..no_of_killer_robots(gamestate.level) {
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

        dumb_robots.push(DumbRobot {
            pos_x: p_x,
            pos_y: p_y,
            is_scrap: false,
            kind: 3,
        });
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

    player.pos_x = p_x;
    player.pos_y = p_y;
}

pub fn run_game(args: &Args) {
    execute!(io::stdout(), Hide).unwrap();
    handle_highscore(&args);
    // Show the splash
    splash_screen();
    game_loop(args);
}

fn game_loop(args: &Args) {

    let mut game_board_data: Vec<Vec<i32>> =
        vec![vec![0; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];

    let mut dumb_robots: Vec<DumbRobot> = Vec::new();
    let mut junk_heaps: Vec<JunkHeap> = Vec::new();


    // Basic setup
    let mut gamestate = GameState {
        turn: 0,
        level: 1,
        wait_for_end: false,
        bomb_away: false,
    };

    let mut player = Player {
        username: args.username.to_string(),
        score: 0,
        is_alive: true,
        pos_x: 0,
        pos_y: 0,
        safe_teleports: 2,
        invincible: false,
        bombs: 0,
    };

    let mut item = Item {
        pos_x: 0,
        pos_y: 0,
        level: 0,
        kind: 0,
        visible: false,
        picked_up: false,
    };

    while player.is_alive {
        // Generate level should generate robots based on the level, and randomize the player position
        generate_level(
            &gamestate,
            &mut game_board_data,
            &mut dumb_robots,
            &mut junk_heaps,
            &mut player,
            &mut item,
        );

        while player.is_alive && any_robots_left(&dumb_robots) {
            draw_boundaries(&player, &gamestate, &junk_heaps, &dumb_robots);
            draw_active_objects(&player, &dumb_robots, &junk_heaps, &item);
            if !gamestate.wait_for_end {
                let (legal_move, quit) =
                    player_input(&mut player, &dumb_robots, &mut gamestate, &game_board_data);
                if legal_move {
                    game_tick(
                        &mut player,
                        &mut dumb_robots,
                        &mut junk_heaps,
                        &mut game_board_data,
                        &mut item,
                        &mut gamestate,
                    );
                }
                if quit {
                    quit_now();
                }
            } else {
                game_tick(
                    &mut player,
                    &mut dumb_robots,
                    &mut junk_heaps,
                    &mut game_board_data,
                    &mut item,
                    &mut gamestate,
                );
                // Sleep for 75ms
                std::thread::sleep(std::time::Duration::from_millis(75));
            }

            if !any_robots_left(&dumb_robots) {
                gamestate.wait_for_end = false;

                // Increase the level (and perhaps write something)
                gamestate.level += 1;
                generate_level(
                    &gamestate,
                    &mut game_board_data,
                    &mut dumb_robots,
                    &mut junk_heaps,
                    &mut player,
                    &mut item,
                );
            }
        }
    }

    // All is over.. Present the retry prompt..
    game_tick(
        &mut player,
        &mut dumb_robots,
        &mut junk_heaps,
        &mut game_board_data,
        &mut item,
        &mut gamestate,
    );
    draw_boundaries(&player, &gamestate, &junk_heaps, &dumb_robots);
    draw_active_objects(&player, &dumb_robots, &junk_heaps, &item);

    move_cursor_padded(4, 0);
    println!("[You did not make it. You were caught by the robots..]");
    add_highscore(&args, &player, &gamestate);

    // Sleep for 1000ms
    std::thread::sleep(std::time::Duration::from_millis(1000));

    if retry_query() {
        game_loop(args);
    } else {
        show_highscore(&args.path, &player, &gamestate);
        quit_now();
        std::process::exit(0);
    }
}
