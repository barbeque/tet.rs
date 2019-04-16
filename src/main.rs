extern crate sdl2;

use crate::sdl2::event::Event;
use crate::sdl2::keyboard::Keycode;
use crate::sdl2::pixels::Color;
use crate::sdl2::render::Canvas;
use crate::sdl2::rect::Rect;
use crate::sdl2::render::WindowCanvas;

const WELL_HEIGHT : usize = 22;
const WELL_WIDTH : usize = 10;

macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {
        Color::RGB($r, $g, $b)
    }
}

struct State {
    cells: [[u8; WELL_WIDTH]; WELL_HEIGHT],
    score: u16,
    lines: u16,
    level: u16,
    // TODO: Next piece
    // TODO: Current piece info (position)
    current_piece_x: u32,
    current_piece_y: u32,
    current_piece: [[u8; 4]; 4] // 4x4 should be enough room for the current piece.
    // TODO: Step timer
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

    for (y, row) in state.cells.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if *cell > 0 {
                let cell_colour = palette[*cell as usize % palette.len()];
                canvas.set_draw_color(cell_colour);
                canvas.fill_rect(
                    Rect::new(well_x as i32 + (x as u32 * tile_size) as i32, well_y as i32 + (y as u32 * tile_size) as i32, tile_size, tile_size)
                ).unwrap();
            }
        }
    }

    // draw the actively moving sprite
    for (cy, row) in state.current_piece.iter().enumerate() {
        for (cx, cell) in row.iter().enumerate() {
            if *cell > 0 {
                let x = ((state.current_piece_x + cx as u32) * tile_size) + well_x;
                let y = ((state.current_piece_y + cy as u32) * tile_size) + well_y;
                let cell_colour = palette[*cell as usize % palette.len()];
                canvas.set_draw_color(cell_colour);
                canvas.fill_rect(
                    Rect::new(x as i32, y as i32, tile_size, tile_size)
                ).unwrap();

                // TODO: remove this duplicate code somehow, it'd be nice...
            }
        }
    }

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
    // FIXME: how should we move the origin back to an actual tile?
    result
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

    let mut state = State {
        cells: [[0; WELL_WIDTH]; WELL_HEIGHT],
        score: 0,
        lines: 0,
        level: 0,
        current_piece_x: 4,
        current_piece_y: 0, // for now
        current_piece:
            [
                [ 4, 0, 0, 0 ],
                [ 4, 4, 4, 0 ],
                [ 0, 0, 0, 0 ],
                [ 0, 0, 0, 0 ]
            ]
    };

    state.cells[21][5] = 6;

    let mut event_pump = sdl_context.event_pump().unwrap();

    'main: loop {
        canvas.clear();

        render_cells(&state, width, height, &mut canvas);

        render_text(10, 10, format!("Score: {}", state.score), &font, &mut canvas);
        render_text(10, 35, format!("Lines: {}", state.lines), &font, &mut canvas);
        render_text(10, 60, format!("Level: {}", state.level), &font, &mut canvas);

        canvas.present();

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
                            state.current_piece = rotated_cw(state.current_piece);
                        }
                        _ => {}
                    }
                },
                // TODO: wire up arrow keys to move piece
                // TODO: wire up rotate to move piece
                // TODO: write can_move and can_rotate
                _ => {}
            }
        }
    }
}
