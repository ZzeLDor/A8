use macroquad::prelude::*;
use std::net::TcpStream;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};

const SQUARES: i16 = 16;
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
    position: (i16, i16),
    power: i16,
    moved: bool,
}

#[derive(Serialize, Deserialize)]
struct Blocker {
    blocked_squares: Vec<(i16, i16)>,
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

fn send_action(action: &str, x: i16, y: i16) -> Game {
    let mut stream = TcpStream::connect("127.0.0.1:25567").unwrap();
    let action = format!("{} {} {}", action, x, y);
    stream.write_all(action.as_bytes()).unwrap();

    let mut buffer = [0; 512];
    let n = stream.read(&mut buffer).unwrap();

    let response = String::from_utf8_lossy(&buffer[..n]);
    let tresponse = response.trim_end_matches(char::from(0));
    serde_json::from_str(&tresponse).unwrap()
}

fn is_mouse_over_button(mouse_pos: Vec2, button_pos: Vec2, button_size: Vec2) -> bool {
    mouse_pos.x >= button_pos.x && mouse_pos.x <= button_pos.x + button_size.x &&
        mouse_pos.y >= button_pos.y && mouse_pos.y <= button_pos.y + button_size.y
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

fn is_within_power(start: Point, end: Point, power: i16) -> bool {
    let dx = (start.0 - end.0).abs();
    let dy = (start.1 - end.1).abs();
    dx <= power && dy <= power && !(dx == 0 && dy == 0)
}

#[macroquad::main("Runner Problem")]
async fn main() {
    let mut game = send_action("init", 0, 0);

    loop {
        match game.game_state {
            GameState::Menu => {
                clear_background(LIGHTGRAY);
                let text = "Start Game";
                let font_size = 30.;
                let text_size = measure_text(text, None, font_size as _, 1.0);
                let button_pos = Vec2::new(screen_width() / 2. - text_size.width / 2., screen_height() / 2. - text_size.height / 2.);
                let button_size = Vec2::new(text_size.width, text_size.height);

                draw_text(
                    text,
                    button_pos.x,
                    button_pos.y + text_size.height,
                    font_size,
                    DARKGRAY,
                );

                if is_mouse_button_pressed(MouseButton::Left) {
                    let mouse_pos = mouse_position();
                    if is_mouse_over_button(Vec2::new(mouse_pos.0, mouse_pos.1), button_pos, button_size) {
                        game = send_action("start_game", 0, 0);
                    }
                }
            }
            GameState::Playing => {
                clear_background(LIGHTGRAY);

                let game_size = screen_width().min(screen_height());
                let offset_x = (screen_width() - game_size) / 2. + 10.;
                let offset_y = (screen_height() - game_size) / 2. + 10.;
                let sq_size = (screen_height() - offset_y * 2.) / game.squares as f32;

                let mouse_pos = mouse_position();
                let hover_pos = get_grid_pos(Vec2::new(mouse_pos.0, mouse_pos.1), offset_x, offset_y, sq_size, game.squares);

                if is_mouse_button_pressed(MouseButton::Left) {
                    if let Some(grid_pos) = hover_pos {
                        if !game.runner.moved {
                            if is_within_power(game.runner.position, grid_pos, game.power)
                                && !game.blocker.blocked_squares.contains(&grid_pos) {
                                game = send_action("move_runner", grid_pos.0, grid_pos.1);
                            }
                        } else if !game.blocker.moved {
                            if grid_pos != game.runner.position && !game.blocker.blocked_squares.contains(&grid_pos) {
                                game = send_action("move_blocker", grid_pos.0, grid_pos.1);
                            }
                        }
                    }
                }

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
                    draw_rectangle(
                        offset_x + i as f32 * sq_size,
                        offset_y,
                        sq_size,
                        sq_size,
                        BLUE,
                    );
                    draw_rectangle(
                        offset_x + i as f32 * sq_size,
                        offset_y + (game.squares - 1) as f32 * sq_size,
                        sq_size,
                        sq_size,
                        BLUE,
                    );
                    draw_rectangle(
                        offset_x,
                        offset_y + i as f32 * sq_size,
                        sq_size,
                        sq_size,
                        BLUE,
                    );
                    draw_rectangle(
                        offset_x + (game.squares - 1) as f32 * sq_size,
                        offset_y + i as f32 * sq_size,
                        sq_size,
                        sq_size,
                        BLUE,
                    );
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

                if let Some(pos) = hover_pos {
                    if !game.runner.moved {
                        if is_within_power(game.runner.position, pos, game.power)
                            && !game.blocker.blocked_squares.contains(&pos) {
                            draw_rectangle(
                                offset_x + pos.0 as f32 * sq_size,
                                offset_y + pos.1 as f32 * sq_size,
                                sq_size,
                                sq_size,
                                Color::new(0.0, 1.0, 0.0, 0.3),
                            );
                        }
                    } else if !game.blocker.moved {
                        if pos != game.runner.position && !game.blocker.blocked_squares.contains(&pos) {
                            draw_rectangle(
                                offset_x + pos.0 as f32 * sq_size,
                                offset_y + pos.1 as f32 * sq_size,
                                sq_size,
                                sq_size,
                                Color::new(1.0, 0.0, 0.0, 0.3),
                            );
                        }
                    }
                }

                draw_text(format!("TURN: {}", game.turn_count).as_str(), 10., 45., 20., DARKGRAY);

                let turn_text = if !game.runner.moved {
                    "Runner's Turn"
                } else if !game.blocker.moved {
                    "Blocker's Turn"
                } else {
                    "Processing..."
                };
                draw_text(turn_text, 10., 70., 20., DARKGRAY);

                if game.game_over {
                    game = send_action("game_over", 0, 0);
                } else if game.won {
                    game = send_action("game_won", 0, 0);
                }
            }
            GameState::GameOver(player_won) => {
                clear_background(LIGHTGRAY);
                let text = if player_won {
                    "Runner won! Return to Menu"
                } else {
                    "Blocker won! Return to Menu"
                };
                let font_size = 30.;
                let text_size = measure_text(text, None, font_size as _, 1.0);
                let button_pos = Vec2::new(screen_width() / 2. - text_size.width / 2., screen_height() / 2. - text_size.height / 2.);
                let button_size = Vec2::new(text_size.width, text_size.height);

                draw_text(
                    text,
                    button_pos.x,
                    button_pos.y + text_size.height,
                    font_size,
                    if player_won { SKYBLUE } else { DARKGRAY },
                );

                if is_mouse_button_pressed(MouseButton::Left) {
                    let mouse_pos = mouse_position();
                    if is_mouse_over_button(Vec2::new(mouse_pos.0, mouse_pos.1), button_pos, button_size) {
                        game = send_action("return_to_menu", 0, 0);
                    }
                }
            }
        }

        next_frame().await;
    }
}