extern crate sdl2;
extern crate rand;

use crate::sdl2::event::Event;
use crate::sdl2::keyboard::Keycode;
use crate::sdl2::pixels::Color;
use crate::sdl2::render::Canvas;
use crate::sdl2::rect::Rect;
use crate::sdl2::render::WindowCanvas;
use crate::sdl2::gfx::framerate::FPSManager;
use crate::rand::prelude::*;

const WELL_HEIGHT : usize = 22;
const WELL_WIDTH : usize = 10;
const FRAMERATE_HZ : u32 = 25;

macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {
        Color::RGB($r, $g, $b)
    }
}

#[derive(PartialEq)]
enum GameState {
    Playing,
    ClearingRows(f32)
}

struct State {
    cells: [[u8; WELL_WIDTH]; WELL_HEIGHT],
    score: u32,
    lines: u16,
    level: u16,
    // TODO: Next piece
    // TODO: Current piece info (position)
    current_piece_x: u32,
    current_piece_y: u32,
    current_piece: [[u8; 4]; 4], // 4x4 should be enough room for the current piece.
    next_piece: [[u8; 4]; 4],
    step_time: f32,
    dropping: bool, // FIXME: this needs a better idea...
    status: GameState
}

fn is_pivot_cell(cell: u8) -> bool {
    cell & 0x80 != 0
}

fn find_pivot_offset(piece: &[[u8; 4]; 4]) -> (u32, u32) {
    for (cy, row) in piece.iter().enumerate() {
        for (cx, cell) in row.iter().enumerate() {
            if *cell > 0 && is_pivot_cell(*cell) {
                return (cx as u32, cy as u32);
            }
        }
    }
    (0, 0) // FIXME: should crash...
}

fn piece_will_land(state: &State) -> bool {
    // looking for a situation where if the piece goes down one more, it will
    // intersect a tile.
    // if this returns true, just write the pieces to the storage where it already is.
    let (pivot_x, pivot_y) = find_pivot_offset(&state.current_piece);

    for (cy, row) in state.current_piece.iter().enumerate() {
        for (cx, cell) in row.iter().enumerate() {
            if *cell > 0 {
                let x : i32 = state.current_piece_x as i32 - pivot_x as i32 + cx as i32;
                if x < 0 { continue; }
                // Test for one deeper
                let y : i32 = (state.current_piece_y + 1) as i32 - pivot_y as i32 + cy as i32;
                if y < 0 { continue; } // bail out on this one if the cell is off screen

                if y >= (WELL_HEIGHT as i32) {
                    return true; // landed on bottom of screen
                }

                if x < WELL_WIDTH as i32 {
                    // check this cell
                    if state.cells[y as usize][x as usize] > 0 {
                        return true; // there's already some landed garbage here
                    }
                }
            }
        }
    }

    false
}

fn can_move_piece(state: &State, piece: &[[u8; 4]; 4], dx: i32, dy: i32) -> bool { // FIXME: state's a bit heavy of a thing to move around here
    // FIXME: de-duplicate...
    // ...also clean up
    let (pivot_x, pivot_y) = find_pivot_offset(piece);

    for (cy, row) in state.current_piece.iter().enumerate() {
        for (cx, cell) in row.iter().enumerate() {
            if *cell > 0 {
                let x : i32 = (state.current_piece_x as i32 + dx) - pivot_x as i32 + cx as i32;
                if x < 0 { return false; }
                // Test for one deeper
                let y : i32 = (state.current_piece_y as i32 + dy) - pivot_y as i32 + cy as i32;
                if y < 0 { continue; } // bail out on this one if the cell is off screen

                if y >= (WELL_HEIGHT as i32) {
                    return false; // landed on bottom of screen
                }

                if x < WELL_WIDTH as i32 {
                    // check this cell
                    if state.cells[y as usize][x as usize] > 0 {
                        return false; // cell is occupied already
                    }
                }

                if x >= WELL_WIDTH as i32 {
                    return false; // can't move this cell outside of the map right side
                }
            }
        }
    }

    true
}

fn can_move_left(state: &State) -> bool { // FIXME: state's a bit heavy of a thing to move around here
    can_move_piece(&state, &state.current_piece, -1, 0)
}

fn can_move_right(state: &State) -> bool { // FIXME: state's a bit heavy of a thing to move around here
    can_move_piece(&state, &state.current_piece, 1, 0)
}

fn render_cells<T : sdl2::render::RenderTarget>(state: &State, width: u32, height: u32, canvas: &mut Canvas<T>) {
    assert!(width > 0);
    assert!(height > 0);

    // FIXME: Don't alloc this every time, make it global
    let palette = vec!
        [ rgb!(240, 232, 205)
        , rgb!(252, 169, 133)
        // yellows
        , rgb!(255,250,129)
        // greens
        , rgb!(224,243,176)
        // blues
        , rgb!(179,226,221)
        , rgb!(111,183,214)
        // purples
        , rgb!(117,139,191)
        // pinks
        , rgb!(249, 140, 182)
        ];

    let tile_size = height / (WELL_HEIGHT as u32);
    let well_x = (width - (WELL_WIDTH as u32 * tile_size)) / 2;
    let well_y = (height - (WELL_HEIGHT as u32 * tile_size)) / 2;

    // Now centre it and draw the well
    // add a cool border.
    canvas.set_draw_color(rgb!(200, 200, 200));
    canvas.fill_rect(Rect::new(well_x as i32 - 2, well_y as i32 - 2, tile_size * WELL_WIDTH as u32 + 4, tile_size * WELL_HEIGHT as u32 + 4)).unwrap();

    // draw the inner well
    canvas.set_draw_color(rgb!(255, 0, 255));
    canvas.fill_rect(Rect::new(well_x as i32, well_y as i32, tile_size * WELL_WIDTH as u32, tile_size * WELL_HEIGHT as u32)).unwrap();

    // FIXME: Remove all this ugly duplicated code...
    // all we're doing is shifting the palette!!!

    match state.status {
        GameState::Playing => {
            for (y, row) in state.cells.iter().enumerate() {
                for (x, cell) in row.iter().enumerate() {
                    if *cell > 0 {
                        let cell_colour = palette[(*cell as usize - 1) % palette.len()];
                        canvas.set_draw_color(cell_colour);
                        canvas.fill_rect(
                            Rect::new(well_x as i32 + (x as u32 * tile_size) as i32, well_y as i32 + (y as u32 * tile_size) as i32, tile_size, tile_size)
                        ).unwrap();
                    }
                }
            }
        },
        GameState::ClearingRows(t) => {
            for (y, row) in state.cells.iter().enumerate() {
                for (x, cell) in row.iter().enumerate() {
                    if *cell > 0 {
                        let mut cell_colour = palette[(*cell as usize - 1) % palette.len()];
                        if row.iter().all(|&c| c > 0) {
                            // this is a clearing row, it should twinkle...
                            // TODO: a better, time based twinkle
                            cell_colour = palette[ rand::thread_rng().next_u32() as usize % palette.len() ]
                        }

                        canvas.set_draw_color(cell_colour);
                        canvas.fill_rect(
                            Rect::new(well_x as i32 + (x as u32 * tile_size) as i32, well_y as i32 + (y as u32 * tile_size) as i32, tile_size, tile_size)
                        ).unwrap();
                    }
                }
            }
        }
    }

    // draw the actively moving sprite
    let (pivot_x, pivot_y) = find_pivot_offset(&state.current_piece);

    for (cy, row) in state.current_piece.iter().enumerate() {
        for (cx, cell) in row.iter().enumerate() {
            if *cell > 0 {
                let x : i32 = state.current_piece_x as i32 - pivot_x as i32 + cx as i32;
                if x < 0 { continue; }
                let y : i32 = state.current_piece_y as i32 - pivot_y as i32 + cy as i32;
                if y < 0 { continue; } // bail out on this one if the cell is off screen

                let x = ((x as u32) * tile_size) + well_x;
                let y = ((y as u32) * tile_size) + well_y;
                let cell_colour = palette[((*cell & 0x7f) as usize - 1) % palette.len()];
                canvas.set_draw_color(cell_colour);
                canvas.fill_rect(
                    Rect::new(x as i32, y as i32, tile_size, tile_size)
                ).unwrap();

                // TODO: remove this duplicate code somehow, it'd be nice...
            }
        }
    }

    // draw the pivot point for debugging (DEBUG)
    let x = (state.current_piece_x * tile_size) + well_x;
    let y = (state.current_piece_y * tile_size) + well_y;
    canvas.set_draw_color(rgb!(255,255,255));
    canvas.fill_rect(
        Rect::new(x as i32 + (tile_size as i32 / 2 - 2), y as i32 + (tile_size as i32 / 2 - 2), 4, 4)
    ).unwrap();

    // done drawing, reset colour state
    canvas.set_draw_color(rgb!(0, 0, 0));
}

// rotate 90 degrees clockwise: (-y, x)
// rotate 90 degrees counter-clockwise: (y, -x)

fn rotated_cw(piece: [[u8; 4]; 4]) -> [[u8; 4]; 4] {
    let mut result = [[0; 4]; 4];
    for y in 0..=3 {
        for x in 0..=3 {
            result[y][x] = piece[3-x][y]; // hmmm
        }
    }
    result
}

fn can_rotate_cw(state: &State) -> bool {
    // 1. try rotating it cw?
    let rotated = rotated_cw(state.current_piece);
    // 2. when you put it on the pivot offset after rotating,
    //    does it hit a wall or another block?
    // 3. if not, we're good.
    return can_move_piece(&state, &rotated, 0, 0);
}

fn render_text(x: i32, y: i32, text: String, font: &sdl2::ttf::Font, canvas: &mut WindowCanvas) { // FIXME
    let surface = font.render(text.as_str())
                        .solid(rgb!(255,255,255))
                        .unwrap();
    let src = surface.rect();
    let creator = canvas.texture_creator(); // TODO: Figure out how to not have to use WindowCanvas so I can still get texture_creator
    let t = creator.create_texture_from_surface(surface).unwrap();

    canvas.copy(&t, src, Rect::new( x, y, src.width(), src.height() )).unwrap(); // TODO: is 0 right?
}

fn land_piece(state: &mut State) {
    // TODO: need to reduplicate all of this code - some kind of special iterator
    let (pivot_x, pivot_y) = find_pivot_offset(&state.current_piece);

    for (cy, row) in state.current_piece.iter().enumerate() {
        for (cx, cell) in row.iter().enumerate() {
            if *cell > 0 {
                let x : i32 = state.current_piece_x as i32 - pivot_x as i32 + cx as i32;
                if x < 0 { continue; }
                let y : i32 = state.current_piece_y as i32 - pivot_y as i32 + cy as i32;
                if y < 0 { continue; } // bail out on this one if the cell is off screen

                state.cells[y as usize][x as usize] = *cell;
            }
        }
    }
}

fn random_piece() -> [[u8; 4]; 4] {
    let mut rng = rand::thread_rng();
    // pick a piece at random from our repertoire
    // store the geometry as 1 except for the pivot which is 128 + 1
    // mul the 'base' value of the piece (pay attention to pivots) by a palette value
    // install the piece with a pivot
    let pieces = [
        [ // J
            [ 1, 0, 0, 0 ],
            [ 1, 129, 1, 0 ],
            [ 0, 0, 0, 0 ],
            [ 0, 0, 0, 0 ]
        ],
        [ // L
            [ 0, 0, 0, 0 ],
            [ 1, 129, 1, 0 ],
            [ 1, 0, 0, 0 ],
            [ 0, 0, 0, 0 ]
        ],
        [ // T
            [ 0, 0, 0, 0 ],
            [ 0, 1, 129, 1 ],
            [ 0, 0, 1, 0 ],
            [ 0, 0, 0, 0 ]
        ],
        [ // O
            [ 0, 0, 0, 0 ],
            [ 0, 1, 129, 0 ],
            [ 0, 1, 1, 0 ],
            [ 0, 0, 0, 0 ]
        ],
        [ // I
            [ 0, 0, 1, 0 ],
            [ 0, 0, 129, 0 ],
            [ 0, 0, 1, 0 ],
            [ 0, 0, 1, 0 ]
        ],
        [ // Z
            [ 0, 0, 0, 1 ],
            [ 0, 0, 129, 1 ],
            [ 0, 0, 1, 0 ],
            [ 0, 0, 0, 0 ]
        ],
        [ // S
            [ 0, 0, 1, 0 ],
            [ 0, 0, 129, 1 ],
            [ 0, 0, 0, 1 ],
            [ 0, 0, 0, 0 ]
        ],
    ];

    let i = (rng.next_u32() as usize) % pieces.len();
    let src = pieces[i];

    let mut result : [[u8; 4]; 4] = [ [0,0,0,0], [0,0,0,0], [0,0,0,0], [0,0,0,0] ]; // FIXME: shorthand?

    let colour = 1 + ((rng.next_u32() as usize) % 8) as u8; // HACK - get the palette global in here for exact length...

    for y in 0..4 {
        for x in 0..4 {
            if src[y][x] > 128 {
                result[y][x] = (src[y][x] - 128) * colour + 128;
            } else {
                result[y][x] = src[y][x] * colour;
            }
        }
    }

    result
}

fn rows_complete(state: &State) -> u32 {
    let mut count = 0;
    for row_idx in 0..state.cells.len() {
        if state.cells[row_idx].iter().all(|&c| c > 0) {
            count += 1; // this row is filled
        }
    }
    count
}

fn clear_completed_rows(state: &mut State) {
    // start from the top
    for row_idx in 0..state.cells.len() {
        if state.cells[row_idx].iter().all(|&c| c > 0) {
            // zero out this row
            for i in 0..state.cells[row_idx].len() {
                state.cells[row_idx][i] = 0;
            }

            // if you see a full row, just copy all the rows above it down
            if row_idx > 0 {
                for copy_row_idx in (row_idx - 1)..0 {
                    state.cells[copy_row_idx + 1] = state.cells[copy_row_idx];
                }
            }
        }
    }
    // FIXME: this feels really bad
}

fn on_piece_landed(state: &mut State) {
    // detect scoring (1, 2, 3, 4, etc)
    let rows_completed = rows_complete(state);
    if rows_completed > 0 {
        // switch to scoring animations if any scores were made
        state.status = GameState::ClearingRows(10.0);
        // 500 points per row
        state.score += rows_completed * (state.level as u32 + 1) * 500;
    }

    // set up the next piece
    //  - swap next piece into new piece
    state.current_piece = state.next_piece;
    //  - compute next piece
    state.next_piece = random_piece();
    //  - reset cursor position
    state.current_piece_y = 0;
    state.current_piece_x = 4;
}

fn step_piece(state: &mut State) {
    if piece_will_land(&state) {
        // TODO: detect losing (piece wrote outside of screen boundaries)
        // write the piece to the state
        land_piece(state);
        on_piece_landed(state);
    } else {
        // drop the piece
        state.current_piece_y += 1; // HACK
    }
}

// TODO: Make an iterator that does a 2D iteration over the current piece
// and hits a callback for each valid square with (cx, cy, cell)?
// Reduce the code everywhere.

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let font = ttf_context.load_font("Enigma_2i.TTF", 22).unwrap();

    let window = video_subsystem
        .window("tetris", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let (width, height) = window.size();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut framerate = FPSManager::new();
    framerate.set_framerate(FRAMERATE_HZ).unwrap(); // set fixed framerate at 25hz

    let mut state = State {
        cells: [[0; WELL_WIDTH]; WELL_HEIGHT],
        score: 0,
        lines: 0,
        level: 0,
        current_piece_x: 4,
        current_piece_y: 0, // for now
        current_piece: random_piece(),
        next_piece: random_piece(),
        step_time: 0.0,
        dropping: false,
        status: GameState::Playing
    };

    let mut event_pump = sdl_context.event_pump().unwrap();

    'main: loop {
        canvas.clear();

        render_cells(&state, width, height, &mut canvas);

        render_text(10, 10, format!("Score: {}", state.score), &font, &mut canvas);
        render_text(10, 35, format!("Lines: {}", state.lines), &font, &mut canvas);
        render_text(10, 60, format!("Level: {}", state.level), &font, &mut canvas);

        canvas.present();

        match state.status {
            GameState::Playing => {
                // only allow input when not clearing rows
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit {..} => break 'main,
                        Event::KeyDown {
                            keycode: Some(Keycode::Escape), ..
                        } => break 'main,
                        Event::KeyDown {
                            keycode: Some(key), ..
                        } => {
                            match key {
                                Keycode::Space => {
                                    // TODO: block can rotate
                                    if can_rotate_cw(&state) {
                                        // TODO: wallkicks?
                                        state.current_piece = rotated_cw(state.current_piece);
                                    }
                                },
                                Keycode::Left => {
                                    if can_move_left(&state) {
                                        state.current_piece_x -= 1;
                                    }
                                },
                                Keycode::Right => {
                                    if can_move_right(&state) {
                                        state.current_piece_x += 1;
                                    }
                                },
                                Keycode::Down => {
                                    state.dropping = true;
                                },
                                _ => {}
                            }
                        },
                        Event::KeyUp {
                            keycode: Some(key), ..
                        } => {
                            match key {
                                Keycode::Down => {
                                    state.dropping = false;
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                }

                let mut step_tick = 2.5;

                if state.dropping {
                    step_tick *= 10.0; // drop faster when DOWN is held
                }

                state.step_time += step_tick;

                // TODO: adjust this 'speed' based on the level
                while state.step_time >= 50.0 { // ehh, i don't like this while
                    state.step_time -= 50.0;
                    step_piece(&mut state);
                }
            },
            GameState::ClearingRows(mut timer) => {
                timer -= 0.55;
                if timer <= 0.0 {
                    // clearing complete, return to game
                    state.status = GameState::Playing;

                    // delete the cleared rows!!!
                    clear_completed_rows(&mut state);
                }
                else {
                    // still clearing, step the timer down
                    state.status = GameState::ClearingRows(timer);
                }

                // stub event pump, just to keep the OS happy
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit {..} => break 'main,
                        Event::KeyDown {
                            keycode: Some(Keycode::Escape), ..
                        } => break 'main,
                        // FIXME: reset dropping state if key up here
                        _ => {}
                    }
                }
            }
        }

        framerate.delay();
    }
}
