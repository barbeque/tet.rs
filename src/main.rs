extern crate sdl2;

use crate::sdl2::event::Event;
use crate::sdl2::keyboard::Keycode;
use crate::sdl2::pixels::Color;
use crate::sdl2::render::Canvas;
use crate::sdl2::rect::Rect;
use crate::sdl2::video::Window;

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
    // TODO: Step timer
}

fn render_cells(state: &State, width: u32, height: u32, canvas: &mut Canvas<Window>) {
    // I guess figure out what size the tiles have to be for the height.
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
    canvas.fill_rect(Rect::new(well_x as i32 - 2, well_y as i32 - 2, tile_size * WELL_WIDTH as u32 + 4, tile_size * WELL_HEIGHT as u32 + 4));

    // draw the inner well
    canvas.set_draw_color(rgb!(255, 0, 255));
    canvas.fill_rect(Rect::new(well_x as i32, well_y as i32, tile_size * WELL_WIDTH as u32, tile_size * WELL_HEIGHT as u32));

    for (y, row) in state.cells.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if *cell > 0 {
                // TODO: index into palette based on cell value
                let cell_colour = palette[*cell as usize % palette.len()];
                canvas.set_draw_color(cell_colour);
                canvas.fill_rect(
                    Rect::new(well_x as i32 + (x as u32 * tile_size) as i32, well_y as i32 + (y as u32 * tile_size) as i32, tile_size, tile_size)
                );
            }
        }
    }

    // done drawing, reset colour state
    canvas.set_draw_color(rgb!(0, 0, 0));
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

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
        level: 0
    };

    state.cells[21][5] = 6;

    let mut event_pump = sdl_context.event_pump().unwrap();

    'main: loop {
        canvas.clear();

        render_cells(&state, width, height, &mut canvas);

        canvas.present();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown {..} => break 'main,
                _ => {}
            }
        }
    }
}
