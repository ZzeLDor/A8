
use std::net::TcpListener;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};
use rand::Rng;

const SQUARES: i16 = 21;
const POWER: i16 = 1;

type Point = (i16, i16);

#[derive(Serialize, Deserialize)]
enum GameState {
    Menu,
    Playing,
    GameOver(bool),
}

#[derive(Serialize, Deserialize)]
struct Runner {
    position: Point,
    power: i16,
    moved: bool,
}

#[derive(Serialize, Deserialize)]
struct Blocker {
    blocked_squares: Vec<Point>,
    moved: bool,
}

#[derive(Serialize, Deserialize)]
struct Game {
    runner: Runner,
    blocker: Blocker,
    game_over: bool,
    won: bool,
    turn_count: i32,
    game_state: GameState,
    squares: i16,
    power: i16,
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
            game_state: GameState::Menu,
            squares: SQUARES,
            power: POWER,
        }
    }

    fn update(&mut self, action: &str) {
        let action: Vec<&str> = action.trim().split_whitespace().collect();
        match action[0] {
            "start_game" => {
                self.game_state = GameState::Playing;
            }
            "move_runner" => {
                if !self.runner.moved && action.len() == 3 {
                    let x: i16 = action[1].parse().unwrap();
                    let y: i16 = action[2].parse().unwrap();
                    self.runner.position = (x, y);
                    self.runner.moved = true;

                    if x == 0 || x == SQUARES - 1 || y == 0 || y == SQUARES - 1 {
                        self.won = true;
                        self.game_state = GameState::GameOver(true);
                    }
                }
            }
            "move_blocker" => {
                if !self.blocker.moved && action.len() == 3 {
                    let x: i16 = action[1].parse().unwrap();
                    let y: i16 = action[2].parse().unwrap();
                    let new_block = (x, y);
                    if !self.blocker.blocked_squares.contains(&new_block) {
                        self.blocker.blocked_squares.push(new_block);
                        self.blocker.moved = true;

                        let mut can_move = false;
                        for dx in -self.runner.power..=self.runner.power {
                            for dy in -self.runner.power..=self.runner.power {
                                let test_pos = (self.runner.position.0 + dx, self.runner.position.1 + dy);
                                if test_pos.0 >= 0 && test_pos.0 < SQUARES && test_pos.1 >= 0 && test_pos.1 < SQUARES && !self.blocker.blocked_squares.contains(&test_pos) {
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
            "return_to_menu" => {
                *self = Game::new();
            }
            _ => {}
        }

        if self.runner.moved && self.blocker.moved {
            self.runner.moved = false;
            self.blocker.moved = false;
            self.turn_count += 1;
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:25567").unwrap();
    let mut game = Game::new();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buffer = [0; 512];
        let n = stream.read(&mut buffer).unwrap();

        let action = String::from_utf8_lossy(&buffer[..n]);
        game.update(&action);

        let response = serde_json::to_string(&game).unwrap();
        stream.write_all(response.as_bytes()).unwrap();
    }
}