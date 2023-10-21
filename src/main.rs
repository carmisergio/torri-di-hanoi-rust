extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use ggez::winit::window;
use glutin_window::GlutinWindow as Window;
use graphics::ImageSize;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{
    Button, MouseButton, MouseCursorEvent, PressEvent, ReleaseEvent, RenderArgs, RenderEvent,
    UpdateArgs, UpdateEvent,
};
use piston::window::WindowSettings;

// Constants
static WINDOW_TITLE: &str = "Torri di Hanoi";
const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const DISC_WIDTH: f64 = 200.0;
const DISC_HEIGHT: f64 = 60.0;
const ROD_WIDTH: f64 = 20.0;
const ROD_HEIGHT: f64 = 500.0;
const ROD_BASE: f64 = WINDOW_HEIGHT as f64 - 100.0;
const ROD_TOP: f64 = ROD_BASE - ROD_HEIGHT;
const ROD_A_CENTER: f64 = 350.0 + ROD_WIDTH / 2.0;
const ROD_B_CENTER: f64 = WINDOW_WIDTH as f64 - 350.0 - ROD_WIDTH / 2.0;
// const RECT_WIDTH_HALF: f32 = RECT_WIDTH / 2;
// const RECT_HEIGHT_HALF: f32 = RECT_HEIGHT / 2;
// const PERNO_WIDTH: f32 = 20.0;
// const PERNO_HEIGHT: f32 = 500.0;
// const PERNO_WIDTH_HALF: f32 = PERNO_WIDTH / 2.0;
// const PERNO_HEIGHT_HALF: f32 = PERNO_HEIGHT / 2.0;
// const PERNO_A_X: f32 = WINDOW_WIDTH / 4.0;
// const PERNO_B_X: f32 = WINDOW_WIDTH - WINDOW_WIDTH / 4.0;
const COLOR_BACKGROUND: [f32; 4] = [0.1, 0.1, 0.1, 1.0];
const COLOR_DISC: [f32; 4] = [0.8, 0.0, 0.8, 1.0];
const COLOR_ROD: [f32; 4] = [0.27, 0.18, 0.05, 1.0];
const COLOR_ROD_HIGHLIGHT: [f32; 4] = [0.94, 0.93, 0.68, 1.0];

fn clamp_rect_position(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    screen_width: f64,
    screen_height: f64,
) -> (f64, f64) {
    let mut new_pos_x = x;
    let mut new_pos_y = y;

    if new_pos_x < 0.0 {
        new_pos_x = 0.0;
    } else if new_pos_x > screen_width - width {
        new_pos_x = screen_width - width;
    }

    if new_pos_y < 0.0 {
        new_pos_y = 0.0;
    } else if new_pos_y > screen_height - height {
        new_pos_y = screen_height - height;
    }

    (new_pos_x, new_pos_y)
}

struct Rod {
    width: f64,
    height: f64,
    pos_x: f64,
    pos_y: f64,
    highlited: bool,
}

impl Rod {
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        // let rect = graphics::rectangle::square(self.pos_x as f64, self.pos_y as f64, 20.0);
        let rect = [self.pos_x, self.pos_y, self.width, self.height];

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            graphics::rectangle(
                if self.highlited {
                    COLOR_ROD_HIGHLIGHT
                } else {
                    COLOR_ROD
                },
                rect,
                transform,
                gl,
            );
        })
    }
}

struct Disc {
    width: f64,
    height: f64,
    pos_x: f64,
    pos_y: f64,
    texture: opengl_graphics::Texture,
}

impl Disc {
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        // let rect = graphics::rectangle::square(self.pos_x as f64, self.pos_y as f64, 20.0);
        // let rect = [self.pos_x, self.pos_y, self.width, self.height];
        let img = graphics::Image::new().rect([self.pos_x, self.pos_y, self.width, self.height]);

        gl.draw(args.viewport(), |c, gl| {
            // let transform = c.transform;

            // graphics::rectangle(COLOR_DISC, rect, transform, gl);
            img.draw(
                &self.texture,
                &graphics::DrawState::default(),
                c.transform,
                gl,
            );
        })
    }

    fn pos_in(&self, x: f64, y: f64) -> bool {
        (x >= self.pos_x && x <= self.pos_x + self.width)
            && (y >= self.pos_y && y <= self.pos_y + self.height)
    }

    fn set_pos(&mut self, x: f64, y: f64) {
        (self.pos_x, self.pos_y) = clamp_rect_position(
            x,
            y,
            self.width,
            self.height,
            WINDOW_WIDTH as f64,
            WINDOW_HEIGHT as f64,
        );
    }
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.

    // Elements
    disc: Disc,
    rod_a: Rod,
    rod_b: Rod,

    // Mouse position
    mouse_pos_x: f64,
    mouse_pos_y: f64,
    //
    // Movement variables
    moving_disc: bool,
    mov_ofst_x: f64,
    mov_ofst_y: f64,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            graphics::clear(COLOR_BACKGROUND, gl);

            // Render
            self.rod_a.render(gl, args);
            self.rod_b.render(gl, args);
            self.disc.render(gl, args);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {}

    fn mouse_moved(&mut self, pos: &[f64; 2]) {
        // Save mouse position
        self.mouse_pos_x = pos[0];
        self.mouse_pos_y = pos[1];

        // Mov disc
        if self.moving_disc {
            self.disc
                .set_pos(pos[0] + self.mov_ofst_x, pos[1] + self.mov_ofst_y);

            if self.disc.pos_x <= WINDOW_WIDTH as f64 / 2.0 - DISC_WIDTH / 2.0 {
                self.rod_a.highlited = true;
                self.rod_b.highlited = false;
            } else {
                self.rod_a.highlited = false;
                self.rod_b.highlited = true;
            }
        }
    }

    fn mouse_button_pressed(&mut self, button: &MouseButton) {
        if *button == MouseButton::Left {
            // Check if mouse has been clicked inside disc
            if self.disc.pos_in(self.mouse_pos_x, self.mouse_pos_y) {
                self.moving_disc = true;

                // Calculate movement offset
                self.mov_ofst_x = self.disc.pos_x - self.mouse_pos_x;
                self.mov_ofst_y = self.disc.pos_y - self.mouse_pos_y;
            }
        }
    }
    fn mouse_button_released(&mut self, button: &MouseButton) {
        if *button == MouseButton::Left {
            if self.moving_disc {
                self.moving_disc = false;

                self.rod_a.highlited = false;
                self.rod_b.highlited = false;

                if self.disc.pos_x <= WINDOW_WIDTH as f64 / 2.0 - DISC_WIDTH / 2.0 {
                    self.disc.set_pos(
                        ROD_A_CENTER - DISC_WIDTH / 2.0,
                        ROD_BASE - DISC_HEIGHT / 2.0,
                    );
                } else {
                    self.disc.set_pos(
                        ROD_B_CENTER - DISC_WIDTH / 2.0,
                        ROD_BASE - DISC_HEIGHT / 2.0,
                    );
                }
            }
        }
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new(WINDOW_TITLE, [WINDOW_WIDTH, WINDOW_HEIGHT])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();

    // Load image

    let texture = opengl_graphics::Texture::from_path(
        std::path::Path::new("./assets/block1.png"),
        &opengl_graphics::TextureSettings::new().mag(opengl_graphics::Filter::Nearest),
    )
    .expect("Could not load brick.");

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        disc: Disc {
            width: DISC_WIDTH,
            height: DISC_HEIGHT,
            pos_x: ROD_A_CENTER - DISC_WIDTH / 2.0,
            pos_y: ROD_BASE - DISC_HEIGHT,
            texture,
        },
        rod_a: Rod {
            width: ROD_WIDTH,
            height: ROD_HEIGHT,
            pos_x: ROD_A_CENTER - ROD_WIDTH / 2.0,
            pos_y: ROD_TOP,
            highlited: false,
        },
        rod_b: Rod {
            width: ROD_WIDTH,
            height: ROD_HEIGHT,
            pos_x: ROD_B_CENTER - ROD_WIDTH / 2.0,
            pos_y: ROD_TOP,
            highlited: false,
        },
        moving_disc: false,
        mov_ofst_x: 0.0,
        mov_ofst_y: 0.0,
        mouse_pos_x: 0.0,
        mouse_pos_y: 0.0,
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }

        // Mouse button events
        if let Some(Button::Mouse(button)) = e.press_args() {
            println!("Pressed mouse button '{:?}'", button);
            app.mouse_button_pressed(&button)
        }
        if let Some(Button::Mouse(button)) = e.release_args() {
            println!("Released mouse button '{:?}'", button);
            app.mouse_button_released(&button)
        }

        // Mouse movement events
        e.mouse_cursor(|pos| {
            // println!("Mouse moved '{} {}'", pos[0], pos[1]);
            app.mouse_moved(&pos);
        });
    }
}
