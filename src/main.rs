extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;

use std::time::Duration;

struct GameBoard {
    cell_size: u32,
    width: u32,
    height: u32,
    mouse_down: bool,
    paused: bool,
    step_divisor: u8,

    data: Vec<bool>,
}

impl GameBoard {
    fn create(size: (u32, u32), cell_size: u32) -> Self {
        let width = size.0 / cell_size;
        let height = size.1 / cell_size;

        let mut vector = Vec::new();
        vector.reserve_exact((width * height).try_into().unwrap());

        let mut i = 0;
        loop {
            if i >= width * height {
                break;
            }

            vector.push(false);
            i += 1;
        }

        Self {
            cell_size,
            width,
            height,

            paused: true,
            mouse_down: false,
            step_divisor: 20,

            data: vector,
        }
    }

    fn reset(&mut self) {
        self.paused = true;
        self.mouse_down = false;

        for i in 0..self.total_squares() {
            self.data[i as usize] = false;
        }
    }

    fn total_squares(&self) -> u32 {
        self.width * self.height
    }

    fn get_neighbor_cells(&self, index: u32) -> Vec<u32> {
        let mut output: Vec<u32> = Vec::new();

        let left_edge = index % self.width == 0;
        let right_edge = (index + 1) % self.width == 0;
        let top_edge = index < self.width;
        let bottom_edge = self.total_squares() - self.width < index;

        // println!("I{index} L{left_edge} R{right_edge} T{top_edge} B{bottom_edge}");

        if !left_edge {
            output.push(index - 1);
        }

        if !right_edge {
            output.push(index + 1);
        }

        if !top_edge {
            output.push(index - self.width);

            if !left_edge {
                output.push(index - self.width - 1);
            }
            if !right_edge {
                output.push(index - self.width + 1);
            }
        }

        if !bottom_edge {
            output.push(index + self.width);

            if !left_edge {
                output.push(index + self.width - 1);
            }

            if !right_edge {
                output.push(index + self.width + 1);
            }
        }

        output
    }

    fn alive_neighbors(&self, index: u32) -> u32 {
        let mut count = 0;
        for cell in self.get_neighbor_cells(index) {
            if cell >= self.total_squares() {
                continue;
            }

            if self.data[cell as usize] {
                count += 1;
            }
        }

        count
    }

    fn step_simulation(&mut self) {
        if self.paused {
            return;
        }

        let mut changes: Vec<(u32, bool)> = Vec::new();

        let mut i = 0;
        while i < self.total_squares() {
            let alive_neighbors = self.alive_neighbors(i);
            let alive = self.data[i as usize];
            let mut change = alive;

            if alive {
                if alive_neighbors < 2 || alive_neighbors > 3 {
                    change = false;
                }
            } else if alive_neighbors == 3 {
                change = true;
            }

            if change != alive {
                changes.push((i, change));
            }

            i += 1;
        }

        for change in changes {
            self.data[change.0 as usize] = change.1;
        }
    }

    fn render(&mut self, canvas: &mut Canvas<Window>, mouse_x: &i32, mouse_y: &i32) {
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        let mut y = 0;
        while y < self.height {
            let mut x = 0;
            while x < self.width {
                let index = y * self.width + x;
                let alive = self.data[index as usize];
                let rect = Rect::new(
                    (x * self.cell_size) as i32,
                    (y * self.cell_size) as i32,
                    self.cell_size,
                    self.cell_size,
                );
                if self.mouse_down && rect.contains_point(Point::new(*mouse_x, *mouse_y)) {
                    self.data[index as usize] = true;
                }

                canvas
                    .draw_point(rect.top_left())
                    .expect("There was an error rendering a point");
                if alive {
                    canvas
                        .fill_rect(rect)
                        .expect("There was an error rendering a cell");
                }

                // println!("{index}:{x}x{y} {alive}");

                x += 1;
            }

            y += 1;
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().expect("Could not get an SDL2 context");
    let video_subsystem = sdl_context.video().expect("Could not get a video context");

    let window_size = (1920, 1080);

    let window = video_subsystem
        .window("Game of Life", window_size.0, window_size.1)
        .position_centered()
        .build()
        .expect("Could not create the window");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("Could not get a canvas for the window");

    clear_canvas(&mut canvas);

    let mut event_pump = sdl_context
        .event_pump()
        .expect("Could not get an event pump");

    let mut game_board = GameBoard::create(window_size, 20);

    'main: loop {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,

                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    game_board.paused = !game_board.paused;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    game_board.step_divisor += 1;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    game_board.step_divisor -= 1;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    game_board.reset();
                }

                Event::MouseButtonDown { mouse_btn, .. } => {
                    println!("{}", game_board.paused);
                    if game_board.paused {
                        match mouse_btn {
                            MouseButton::Left => {
                                game_board.mouse_down = true;
                            }

                            _ => {}
                        }
                    }
                }

                Event::MouseButtonUp { mouse_btn, .. } => match mouse_btn {
                    MouseButton::Left => {
                        game_board.mouse_down = false;
                    }

                    _ => {}
                },

                // Other events
                _ => {}
            }
        }

        let mut mouse_x = 0;
        let mut mouse_y = 0;
        unsafe {
            sdl2::sys::SDL_GetMouseState(&mut mouse_x, &mut mouse_y);
        }

        // Step the game simulation
        game_board.step_simulation();

        // Render the final result
        game_board.render(&mut canvas, &mouse_x, &mouse_y);

        // Show the rendered frame
        canvas.present();

        let mut step_divisor = game_board.step_divisor;
        if game_board.paused {
            step_divisor = 60;
        }
        println!("{step_divisor}");

        // Sleep so we don't have a seizure
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / step_divisor as u32));
    }
}

fn clear_canvas(canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();
}
