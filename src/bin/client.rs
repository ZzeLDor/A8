use macroquad::prelude::*;
use std::net::TcpStream;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};

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
    position: (i16, i16),
    power: i16,
    moved: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct Blocker {
    blocked_squares: Vec<(i16, i16)>,
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

fn send_action(action: &str) -> (Game, String, bool) {
    let mut stream = TcpStream::connect("127.0.0.1:25567").unwrap();
    stream.write_all(action.as_bytes()).unwrap();

    let mut buffer = [0; 4096];
    let n = stream.read(&mut buffer).unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);
    serde_json::from_str(&response).unwrap()
}

struct GameClient {
    stream: Option<TcpStream>,
    max_retries: u32,
}

impl GameClient {
    fn new(max_retries: u32) -> Self {
        Self {
            stream: None,
            max_retries,
        }
    }

    fn connect(&mut self) -> Result<(), std::io::Error> {
        self.stream = Some(TcpStream::connect("127.0.0.1:25567")?);
        Ok(())
    }

    fn ensure_connected(&mut self) -> Result<(), std::io::Error> {
        if self.stream.is_none() {
            self.connect()?;
        }
        Ok(())
    }

    fn send_action(&mut self, action: &str) -> Result<(Game, String, bool), std::io::Error> {
        let mut retries = 0;
        loop {
            if retries >= self.max_retries {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Max retries reached"
                ));
            }

            match self.ensure_connected() {
                Ok(()) => {
                    let stream = self.stream.as_mut().unwrap();
                    match stream.write_all(action.as_bytes()) {
                        Ok(()) => {
                            let mut buffer = [0; 4096];
                            match stream.read(&mut buffer) {
                                Ok(n) if n > 0 => {
                                    let response = String::from_utf8_lossy(&buffer[..n]);
                                    return Ok(serde_json::from_str(&response).unwrap());
                                }
                                Ok(_) | Err(_) => {
                                    self.stream = None;
                                    retries += 1;
                                    std::thread::sleep(std::time::Duration::from_millis(500));
                                    continue;
                                }
                            }
                        }
                        Err(_) => {
                            self.stream = None;
                            retries += 1;
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            continue;
                        }
                    }
                }
                Err(_) => {
                    retries += 1;
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    continue;
                }
            }
        }
    }
}

fn is_within_power(start: Point, end: Point, power: i16) -> bool {
    let dx = (start.0 - end.0).abs();
    let dy = (start.1 - end.1).abs();
    dx <= power && dy <= power && !(dx == 0 && dy == 0)
}

fn get_grid_pos(mouse_pos: Vec2, offset_x: f32, offset_y: f32, sq_size: f32, sent_sq: i16) -> Option<Point> {
    let grid_x = ((mouse_pos.x - offset_x) / sq_size) as i16;
    let grid_y = ((mouse_pos.y - offset_y) / sq_size) as i16;

    if grid_x >= 0 && grid_x < sent_sq && grid_y >= 0 && grid_y < sent_sq {
        Some((grid_x, grid_y))
    } else {
        None
    }
}

#[macroquad::main("Angel Problem - Multiplayer")]
async fn main() {
    let mut client = GameClient::new(3);
    let (mut game, mut player_type, _) = match client.send_action("init") {
        Ok(response) => response,
        Err(e) => {
            println!("Failed to connect to server: {}", e);
            return;
        }
    };
    let mut last_update = get_time();

    loop {
        clear_background(LIGHTGRAY);
        if matches!(game.game_state, GameState::WaitingForPlayers) && is_key_pressed(KeyCode::Space) {
            match client.send_action("activate_cpu") {
                Ok((new_game, new_player_type, _)) => {
                    game = new_game;
                    player_type = new_player_type;
                }
                Err(e) => {
                    draw_text(
                        &format!("Failed to activate CPU mode: {}. Retrying...", e),
                        10.,
                        30.,
                        20.,
                        RED,
                    );
                }
            }
        }

        match game.game_state {
            GameState::WaitingForPlayers => {
                let text = "Press SPACE to play against CPU or wait for other player...";
                let font_size = 30.;
                let text_size = measure_text(text, None, font_size as _, 1.0);
                draw_text(
                    text,
                    screen_width() / 2. - text_size.width / 2.,
                    screen_height() / 2.,
                    font_size,
                    DARKGRAY,
                );
            }
            GameState::CPUMode | GameState::Playing => {
                let game_size = screen_width().min(screen_height());
                let offset_x = (screen_width() - game_size) / 2. + 10.;
                let offset_y = (screen_height() - game_size) / 2. + 10.;
                let sq_size = (screen_height() - offset_y * 2.) / game.squares as f32;

                draw_rectangle(offset_x, offset_y, game_size - 20., game_size - 20., WHITE);
                for i in 1..game.squares {
                    draw_line(
                        offset_x,
                        offset_y + sq_size * i as f32,
                        screen_width() - offset_x,
                        offset_y + sq_size * i as f32,
                        2.,
                        LIGHTGRAY,
                    );
                    draw_line(
                        offset_x + sq_size * i as f32,
                        offset_y,
                        offset_x + sq_size * i as f32,
                        screen_height() - offset_y,
                        2.,
                        LIGHTGRAY,
                    );
                }

                for i in 0..game.squares {
                    for (x, y) in [(i, 0), (i, game.squares-1), (0, i), (game.squares-1, i)] {
                        draw_rectangle(
                            offset_x + x as f32 * sq_size,
                            offset_y + y as f32 * sq_size,
                            sq_size,
                            sq_size,
                            BLUE,
                        );
                    }
                }

                for pos in &game.blocker.blocked_squares {
                    draw_rectangle(
                        offset_x + pos.0 as f32 * sq_size,
                        offset_y + pos.1 as f32 * sq_size,
                        sq_size,
                        sq_size,
                        RED,
                    );
                }

                draw_rectangle(
                    offset_x + game.runner.position.0 as f32 * sq_size,
                    offset_y + game.runner.position.1 as f32 * sq_size,
                    sq_size,
                    sq_size,
                    GOLD,
                );

                let mouse_pos = mouse_position();
                let hover_pos = get_grid_pos(Vec2::new(mouse_pos.0, mouse_pos.1), offset_x, offset_y, sq_size, game.squares);

                if let Some(pos) = hover_pos {
                    if game.current_player == player_type {
                        let (color, valid_move) = match player_type.as_str() {
                            "runner" => (
                                Color::new(0.0, 1.0, 0.0, 0.3),
                                is_within_power(game.runner.position, pos, game.power)
                                    && !game.blocker.blocked_squares.contains(&pos)
                            ),
                            "blocker" => (
                                Color::new(1.0, 0.0, 0.0, 0.3),
                                pos != game.runner.position
                                    && !game.blocker.blocked_squares.contains(&pos)
                            ),
                            _ => (Color::new(0.0, 0.0, 0.0, 0.0), false),
                        };

                        if valid_move {
                            draw_rectangle(
                                offset_x + pos.0 as f32 * sq_size,
                                offset_y + pos.1 as f32 * sq_size,
                                sq_size,
                                sq_size,
                                color,
                            );
                        }
                    }
                }

                if is_mouse_button_pressed(MouseButton::Left) && game.current_player == player_type {
                    if let Some(grid_pos) = hover_pos {
                        let action = match player_type.as_str() {
                            "runner" if is_within_power(game.runner.position, grid_pos, game.power)
                                && !game.blocker.blocked_squares.contains(&grid_pos) => {
                                Some(format!("move_runner {} {}", grid_pos.0, grid_pos.1))
                            }
                            "blocker" if grid_pos != game.runner.position
                                && !game.blocker.blocked_squares.contains(&grid_pos) => {
                                Some(format!("move_blocker {} {}", grid_pos.0, grid_pos.1))
                            }
                            _ => None,
                        };

                        if let Some(action) = action {
                            let (new_game, new_player_type, _) = match client.send_action(&action){
                                Ok(response) => response,
                                Err(e) => {
                                    println!("Failed to send action: {}", e);
                                    break;
                                }
                            };
                            game = new_game;
                            player_type = new_player_type;
                        }
                    }
                }

                draw_text(
                    &format!("Turn: {} | You are: {}", game.turn_count, player_type),
                    10.,
                    30.,
                    20.,
                    DARKGRAY,
                );
                draw_text(
                    &format!("Current turn: {}", game.current_player),
                    10.,
                    60.,
                    20.,
                    if game.current_player == player_type { GREEN } else { DARKGRAY },
                );
            }
            GameState::GameOver(runner_won) => {
                let text = if runner_won {
                    if player_type == "runner" {
                        "You won! The runner escaped!"
                    } else {
                        "You lost! The runner escaped!"
                    }
                } else {
                    if player_type == "blocker" {
                        "You won! The runner is trapped!"
                    } else {
                        "You lost! You are trapped!"
                    }
                };

                let font_size = 30.;
                let text_size = measure_text(text, None, font_size as _, 1.0);
                draw_text(
                    text,
                    screen_width() / 2. - text_size.width / 2.,
                    screen_height() / 2.,
                    font_size,
                    if runner_won == (player_type == "runner") { GREEN } else { RED },
                );
            }
        }
        if game.current_player != player_type && get_time() - last_update >= 0.1 {
            let (new_game, new_player_type, _) = match client.send_action("poll") {
                Ok(response) => response,
                Err(e) => {
                    println!("Failed to poll for updates: {}", e);
                    break;
                }
            };
            game = new_game;
            player_type = new_player_type;
            last_update = get_time();
        }

        next_frame().await;
    }
}