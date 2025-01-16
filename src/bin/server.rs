use std::net::TcpListener;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};
use rand::Rng;
use rand::seq::SliceRandom;
use std::sync::{Arc, Mutex};
use std::thread;

const SQUARES: i16 = 21;
const POWER: i16 = 1;

type Point = (i16, i16);

#[derive(Serialize, Deserialize, Clone)]
enum GameState {
    WaitingForPlayers,
    Playing,
    GameOver(bool),
    CPUMode,
}

#[derive(Serialize, Deserialize, Clone)]
struct Runner {
    position: Point,
    power: i16,
    moved: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct Blocker {
    blocked_squares: Vec<Point>,
    moved: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct Game {
    runner: Runner,
    blocker: Blocker,
    game_over: bool,
    won: bool,
    turn_count: i32,
    game_state: GameState,
    squares: i16,
    power: i16,
    current_player: String,
}

impl Game {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut blocked_squares = Vec::new();
        for _ in 0..SQUARES * 2 {
            blocked_squares.push((rng.gen_range(1..SQUARES-1), rng.gen_range(1..SQUARES-1)));
        }

        Game {
            runner: Runner {
                position: (SQUARES / 2, SQUARES / 2),
                power: POWER,
                moved: false,
            },
            blocker: Blocker {
                blocked_squares,
                moved: false,
            },
            game_over: false,
            won: false,
            turn_count: 0,
            game_state: GameState::WaitingForPlayers,
            squares: SQUARES,
            power: POWER,
            current_player: String::from("runner"),
        }
    }

    fn update(&mut self, action: &str, player_type: &str) -> bool {
        if player_type != self.current_player {
            return false;
        }

        let action: Vec<&str> = action.trim().split_whitespace().collect();
        match action[0] {
            "move_runner" => {
                if player_type == "runner" && !self.runner.moved && action.len() == 3 {
                    let x: i16 = action[1].parse().unwrap();
                    let y: i16 = action[2].parse().unwrap();
                    self.runner.position = (x, y);
                    self.runner.moved = true;
                    self.current_player = String::from("blocker");

                    if x == 0 || x == SQUARES - 1 || y == 0 || y == SQUARES - 1 {
                        self.won = true;
                        self.game_state = GameState::GameOver(true);
                    }
                }
            }
            "move_blocker" => {
                if player_type == "blocker" && !self.blocker.moved && action.len() == 3 {
                    let x: i16 = action[1].parse().unwrap();
                    let y: i16 = action[2].parse().unwrap();
                    let new_block = (x, y);
                    if !self.blocker.blocked_squares.contains(&new_block) {
                        self.blocker.blocked_squares.push(new_block);
                        self.blocker.moved = true;
                        self.current_player = String::from("runner");

                        let mut can_move = false;
                        for dx in -self.runner.power..=self.runner.power {
                            for dy in -self.runner.power..=self.runner.power {
                                let test_pos = (self.runner.position.0 + dx, self.runner.position.1 + dy);
                                if test_pos.0 >= 0 && test_pos.0 < SQUARES &&
                                    test_pos.1 >= 0 && test_pos.1 < SQUARES &&
                                    !self.blocker.blocked_squares.contains(&test_pos) {
                                    can_move = true;
                                    break;
                                }
                            }
                        }
                        if !can_move {
                            self.game_over = true;
                            self.game_state = GameState::GameOver(false);
                        }
                    }
                }
            }
            _ => {}
        }

        if self.runner.moved && self.blocker.moved {
            self.runner.moved = false;
            self.blocker.moved = false;
            self.turn_count += 1;
        }

        true
    }
}

fn simulate_cpu_runner(game: &mut Game) {
    let mut rng = rand::thread_rng();
    let mut possible_moves = Vec::new();

    for dx in -game.runner.power..=game.runner.power {
        for dy in -game.runner.power..=game.runner.power {
            let new_pos = (game.runner.position.0 + dx, game.runner.position.1 + dy);

            if new_pos.0 >= 0 && new_pos.0 < SQUARES &&
                new_pos.1 >= 0 && new_pos.1 < SQUARES &&
                !game.blocker.blocked_squares.contains(&new_pos) {

                if new_pos.0 == 0 || new_pos.0 == SQUARES - 1 ||
                    new_pos.1 == 0 || new_pos.1 == SQUARES - 1 {
                    for _ in 0..3 {
                        possible_moves.push(new_pos);
                    }
                } else {
                    possible_moves.push(new_pos);
                }
            }
        }
    }

    if let Some(&new_pos) = possible_moves.choose(&mut rng) {
        game.runner.position = new_pos;
        game.runner.moved = true;
        game.current_player = String::from("blocker");

        if new_pos.0 == 0 || new_pos.0 == SQUARES - 1 ||
            new_pos.1 == 0 || new_pos.1 == SQUARES - 1 {
            game.won = true;
            game.game_state = GameState::GameOver(true);
        }
    }
}

fn handle_client(mut stream: std::net::TcpStream, game: Arc<Mutex<Game>>, player_type: String) {
    loop {
        let mut buffer = [0; 512];
        match stream.read(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                let action = String::from_utf8_lossy(&buffer[..n]);
                let mut game = game.lock().unwrap();

                if action.trim() == "activate_cpu" && matches!(game.game_state, GameState::WaitingForPlayers) {
                    game.game_state = GameState::CPUMode;
                    if game.current_player == "runner" {
                        simulate_cpu_runner(&mut game);
                    }
                    let response = serde_json::to_string(&(game.clone(), player_type.clone(), true)).unwrap();
                    if let Err(_) = stream.write_all(response.as_bytes()) {
                        break;
                    }
                    continue;
                }
                if action.trim() == "poll" && matches!(game.game_state, GameState::CPUMode) {
                    if game.current_player == "runner" {
                        simulate_cpu_runner(&mut game);
                    }
                    let response = serde_json::to_string(&(game.clone(), player_type.clone(), true)).unwrap();
                    if let Err(_) = stream.write_all(response.as_bytes()) {
                        break;
                    }
                    continue;
                }

                let success = game.update(&action, &player_type);

                if success && matches!(game.game_state, GameState::CPUMode) {
                    if game.current_player == "runner" {
                        simulate_cpu_runner(&mut game);
                    }
                }

                let response = serde_json::to_string(&(game.clone(), player_type.clone(), success)).unwrap();
                if let Err(_) = stream.write_all(response.as_bytes()) {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}
fn main() {
    let listener = TcpListener::bind("127.0.0.1:25567").unwrap();
    let game = Arc::new(Mutex::new(Game::new()));
    let mut player_count = 0;
    let active_connections = Arc::new(Mutex::new(Vec::new()));

    println!("Server started, waiting for players...");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut connections = active_connections.lock().unwrap();

                if player_count >= 2 {
                    let response = serde_json::to_string(&("Game full".to_string())).unwrap();
                    let _ = stream.write_all(response.as_bytes());
                    continue;
                }

                let game = Arc::clone(&game);
                player_count += 1;

                let player_type = if player_count == 1 {
                    println!("Blocker connected!");
                    String::from("blocker")
                } else {
                    println!("Runner connected!");
                    {
                        let mut game = game.lock().unwrap();
                        game.game_state = GameState::Playing;
                    }
                    String::from("runner")
                };

                connections.push(stream.try_clone().unwrap());

                thread::spawn(move || {
                    handle_client(stream, game, player_type);
                });

                if player_count == 2 {
                    println!("Game started!");
                }
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}

