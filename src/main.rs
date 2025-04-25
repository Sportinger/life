use std::time::{SystemTime,UNIX_EPOCH};
use std::path::Path;

extern crate piston_window;
use piston_window::*;

mod life;
use life::{World, Loc, CellType};

const BLACK: [f32;4] = [0.0, 0.0, 0.0, 1.0];
const WHITE: [f32;4] = [1.0; 4];
const RED: [f32;4] = [1.0, 0.0, 0.0, 1.0];
const BLUE: [f32;4] = [0.0, 0.0, 1.0, 1.0];
const SQUARE_SIZE: f64 = 5.0;
const WINDOW_SIZE: u32 = 1024;
const GFX_CONTEXT_OFFSET: f64 = (WINDOW_SIZE / 2) as f64;
const MILLIS_PER_FRAME: u128 = 10;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: life CONFIGURATION")
    } else {
        let opengl = OpenGL::V3_2;
        let mut window: PistonWindow = WindowSettings::new("Life", [WINDOW_SIZE; 2])
            .exit_on_esc(true)
            .graphics_api(opengl)
            .build()
            .unwrap();

        let configuration_path = String::from("./src/configurations/") + &args[1] + ".txt";
        let mut world = World::from_configuration(&std::fs::read_to_string(Path::new(&configuration_path)).unwrap(), '.', '*').unwrap();
        world.swap_buffers_and_clear();

        let mut previous_update = UNIX_EPOCH;
        let mut last_mouse_pos = [0.0, 0.0]; // Variable to store mouse position
        let mut is_left_mouse_down = false; // Track if left mouse button is held
        let mut is_right_mouse_down = false; // Track if right mouse button is held

        while let Some(e) = window.next() {
            // --- Store Mouse Position ---
            if let Some(pos) = e.mouse_cursor_args() {
                last_mouse_pos = pos;
            }
            // --- End Store Mouse Position ---

            // --- Handle Mouse Button State ---
            if let Event::Input(Input::Button(ButtonArgs {
                state, // Capture the state (Press or Release)
                button: Button::Mouse(button),
                scancode: _, 
            }), _timestamp) = e
            {
                match button {
                    MouseButton::Left => {
                        is_left_mouse_down = state == ButtonState::Press;
                        println!("Left mouse button: {}", if is_left_mouse_down { "pressed" } else { "released" });
                    },
                    MouseButton::Right => {
                        is_right_mouse_down = state == ButtonState::Press;
                        println!("Right mouse button: {}", if is_right_mouse_down { "pressed" } else { "released" });
                    },
                    _ => {}
                }
            }
            // --- End Mouse Button State ---

            // --- Continuous Drawing while Mouse Down ---
            if is_left_mouse_down || is_right_mouse_down {
                // Use the stored mouse position
                let pos = last_mouse_pos;

                // Convert window coordinates [x, y] to world coordinates Loc { row, col }
                let world_x = pos[0] - GFX_CONTEXT_OFFSET;
                let world_y = pos[1] - GFX_CONTEXT_OFFSET;

                // Divide by square size and floor to get cell coordinates
                let col = (world_x / SQUARE_SIZE).floor() as i64;
                let row = (world_y / SQUARE_SIZE).floor() as i64;

                // Set cell type based on which mouse button is pressed
                let cell_type = if is_left_mouse_down && is_right_mouse_down {
                    // If both buttons are pressed, prioritize left (Red)
                    CellType::Red
                } else if is_left_mouse_down {
                    CellType::Red
                } else if is_right_mouse_down {
                    CellType::Blue
                } else {
                    // This shouldn't happen given our if condition above, but just in case
                    CellType::Red
                };

                // Set a 2x2 block of cells with the appropriate type
                println!("Setting cells at ({}, {}) with type: {:?}", row, col, cell_type);
                world.set_cell_now(&Loc::new(row, col), cell_type);
                world.set_cell_now(&Loc::new(row + 1, col), cell_type);
                world.set_cell_now(&Loc::new(row, col + 1), cell_type);
                world.set_cell_now(&Loc::new(row + 1, col + 1), cell_type);
            }
            // --- End Continuous Drawing ---

            if previous_update.elapsed().map(|d| d.as_millis()).unwrap_or(0) > MILLIS_PER_FRAME {
                // NOTE: Uncomment for timing info
                // let step_start = SystemTime::now();
                world.step();
                // println!("Step took: {}ms", step_start.elapsed().map(|d| d.as_micros()).unwrap_or(0) as f32 / 1000.0);
                previous_update = SystemTime::now();
            }
            
            window.draw_2d(&e, |context, graphics, _| {
                clear(BLACK, graphics);

                // Translate by 1/2 the window size, to center 0,0 in the middle of the window
                let context = context.trans(GFX_CONTEXT_OFFSET, GFX_CONTEXT_OFFSET);
                
                // Use iter() to get key and value directly, avoiding extra get() lookup
                for (loc, cell_type) in world.current_buffer().iter() {
                    if cell_type.is_alive() {
                        let color = match cell_type {
                            CellType::Red => RED,
                            CellType::Blue => BLUE,
                            CellType::Dead => WHITE, // Should never happen due to is_alive() check
                        };
                        rectangle(color, [loc.col as f64 * SQUARE_SIZE, loc.row as f64 * SQUARE_SIZE, SQUARE_SIZE, SQUARE_SIZE], context.transform, graphics);
                    }
                }
            });
        }
    }
}