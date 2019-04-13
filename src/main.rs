extern crate sdl2;

use crate::sdl2::event::Event;
use crate::sdl2::keyboard::Keycode;
use crate::sdl2::pixels::Color;
use crate::sdl2::render::Canvas;
use crate::sdl2::rect::Rect;
use crate::sdl2::video::Window;

const WELL_HEIGHT : usize = 22;
const WELL_WIDTH : usize = 10;

struct State {
    cells: [[u8; WELL_WIDTH]; WELL_HEIGHT],
    score: u16,
    lines: u16,
    level: u16,
    // TODO: Next piece
    // TODO: Current piece info (position)
    // TODO: Step timer
}

fn render_cells(state: &State, canvas: &mut Canvas<Window>) {
    // I guess figure out what size the tiles have to be for the height.
    let (width, height) = canvas.logical_size();
    assert!(width > 0);
    assert!(height > 0);

    let tile_size = height / (WELL_HEIGHT as u32);
    let well_x = (width - (WELL_WIDTH as u32 * tile_size)) / 2;
    let well_y = (height - (WELL_HEIGHT as u32 * tile_size)) / 2;

    // Now centre it
    // TODO: add a cool border.
    canvas.set_draw_color(Color::RGB(255, 0, 255));
    canvas.fill_rect(Rect::new(well_x as i32, well_y as i32, tile_size * WELL_WIDTH as u32, tile_size * WELL_HEIGHT as u32));
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

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut state = State {
        cells: [[0; WELL_WIDTH]; WELL_HEIGHT],
        score: 0,
        lines: 0,
        level: 0
    };

    let mut event_pump = sdl_context.event_pump().unwrap();

    'main: loop {
        canvas.clear();

        render_cells(&state, &mut canvas);

        canvas.present();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown {..} => break 'main,
                _ => {}
            }
        }
    }
}
