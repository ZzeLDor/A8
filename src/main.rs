use macroquad::prelude::*;

const SQUARES: i16 = 16;

type Point = (i16, i16);

struct Angel {
    position: Point,
    power: i16,
    moved: bool,
}

struct Devil {
    blocked_squares: Vec<Point>,
    moved: bool,
}

fn get_grid_pos(mouse_pos: Vec2, offset_x: f32, offset_y: f32, sq_size: f32) -> Option<Point> {
    let grid_x = ((mouse_pos.x - offset_x) / sq_size) as i16;
    let grid_y = ((mouse_pos.y - offset_y) / sq_size) as i16;

    if grid_x >= 0 && grid_x < SQUARES && grid_y >= 0 && grid_y < SQUARES {
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

#[macroquad::main("Angel Problem")]
async fn main() {
    let mut angel = Angel {
        position: (SQUARES/2, SQUARES-1),
        power: 2,
        moved: false,
    };

    let mut devil = Devil {
        blocked_squares: Vec::new(),
        moved: false,
    };

    let mut rng = rand::gen_range(0, SQUARES);
    rng = rand::gen_range(0, SQUARES);
    for _ in 0..rng {
        devil.blocked_squares.push((rand::gen_range(0, SQUARES), rand::gen_range(0, SQUARES)));
    }

    let mut game_over = false;
    let mut won = false;
    let mut turn_count = 0;
    let mut hover_pos: Option<Point> = None;

    loop {
        if !game_over && !won {
            let game_size = screen_width().min(screen_height());
            let offset_x = (screen_width() - game_size) / 2. + 10.;
            let offset_y = (screen_height() - game_size) / 2. + 10.;
            let sq_size = (screen_height() - offset_y * 2.) / SQUARES as f32;
            let mouse_pos = mouse_position();
            hover_pos = get_grid_pos(Vec2::new(mouse_pos.0, mouse_pos.1), offset_x, offset_y, sq_size);

            if is_mouse_button_pressed(MouseButton::Left) {
                if let Some(grid_pos) = hover_pos {
                    if !angel.moved {
                        if is_within_power(angel.position, grid_pos, angel.power)
                            && !devil.blocked_squares.contains(&grid_pos) {
                            angel.position = grid_pos;
                            angel.moved = true;

                            if angel.position.1 == 0 {
                                won = true;
                            }
                        }
                    } else if !devil.moved {
                        if grid_pos != angel.position && !devil.blocked_squares.contains(&grid_pos) {
                            devil.blocked_squares.push(grid_pos);
                            devil.moved = true;

                            let mut can_move = false;
                            'check: for dx in -angel.power..=angel.power {
                                for dy in -angel.power..=angel.power {
                                    let test_pos = (
                                        angel.position.0 + dx,
                                        angel.position.1 + dy
                                    );
                                    if test_pos.0 >= 0 && test_pos.0 < SQUARES &&
                                        test_pos.1 >= 0 && test_pos.1 < SQUARES &&
                                        !devil.blocked_squares.contains(&test_pos) {
                                        can_move = true;
                                        break 'check;
                                    }
                                }
                            }
                            if !can_move {
                                game_over = true;
                            }
                        }
                    }
                }
            }

            if angel.moved && devil.moved {
                angel.moved = false;
                devil.moved = false;
                turn_count += 1;
            }
        }

        clear_background(LIGHTGRAY);

        let game_size = screen_width().min(screen_height());
        let offset_x = (screen_width() - game_size) / 2. + 10.;
        let offset_y = (screen_height() - game_size) / 2. + 10.;
        let sq_size = (screen_height() - offset_y * 2.) / SQUARES as f32;

        draw_rectangle(offset_x, offset_y, game_size - 20., game_size - 20., WHITE);
        for i in 1..SQUARES {
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

        for i in 0..SQUARES {
            draw_rectangle(
                offset_x + i as f32 * sq_size,
                offset_y,
                sq_size,
                sq_size,
                SKYBLUE,
            );
        }

        for pos in &devil.blocked_squares {
            draw_rectangle(
                offset_x + pos.0 as f32 * sq_size,
                offset_y + pos.1 as f32 * sq_size,
                sq_size,
                sq_size,
                RED,
            );
        }

        draw_rectangle(
            offset_x + angel.position.0 as f32 * sq_size,
            offset_y + angel.position.1 as f32 * sq_size,
            sq_size,
            sq_size,
            GOLD,
        );

        if let Some(pos) = hover_pos {
            if !angel.moved {
                if is_within_power(angel.position, pos, angel.power)
                    && !devil.blocked_squares.contains(&pos) {
                    draw_rectangle(
                        offset_x + pos.0 as f32 * sq_size,
                        offset_y + pos.1 as f32 * sq_size,
                        sq_size,
                        sq_size,
                        Color::new(0.0, 1.0, 0.0, 0.3),
                    );
                }
            } else if !devil.moved {
                if pos != angel.position && !devil.blocked_squares.contains(&pos) {
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

        draw_text(format!("TURN: {turn_count}").as_str(), 10., 45., 20., DARKGRAY);

        let turn_text = if !angel.moved {
            "Angel's Turn"
        } else if !devil.moved {
            "Devil's Turn"
        } else {
            "Processing..."
        };
        draw_text(turn_text, 10., 70., 20., DARKGRAY);

        if game_over || won {
            let text = if won {
                "Press [enter] to play again."
            } else {
                "Press [enter] to play again."
            };
            let font_size = 30.;
            let text_size = measure_text(text, None, font_size as _, 1.0);

            draw_text(
                text,
                screen_width() / 2. - text_size.width / 2.,
                screen_height() / 2. + text_size.height / 2.,
                font_size,
                if won { SKYBLUE } else { DARKGRAY },
            );

            if is_key_down(KeyCode::Enter) {
                angel = Angel {
                    position: (SQUARES/2, SQUARES-1),
                    power: 2,
                    moved: false,
                };
                devil = Devil {
                    blocked_squares: Vec::new(),
                    moved: false,
                };
                turn_count = 0;
                game_over = false;
                won = false;
                rng = rand::gen_range(0, SQUARES);
                for _ in 0..rng {
                    devil.blocked_squares.push((rand::gen_range(0, SQUARES), rand::gen_range(0, SQUARES)));
                }
            }
        }

        next_frame().await;
    }
}