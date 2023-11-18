extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
mod textures;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{
    Button, MouseButton, MouseCursorEvent, PressEvent, ReleaseEvent, RenderArgs, RenderEvent,
    UpdateArgs, UpdateEvent,
};
use piston::window::WindowSettings;
// use piston::Window;

use textures::{
    compute_disc_color, load_disc_texture_color, load_rod_texture, DiscTexture, RodTexture,
};

// Constants
static WINDOW_TITLE: &str = "Torri di Hanoi";
const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const DISC_WIDTH_MIN: f64 = 100.0;
const DISC_WIDTH_MAX: f64 = 350.0;
const DISC_HEIGHT: f64 = 60.0;
const ROD_WIDTH: f64 = 20.0;
const ROD_HEIGHT: f64 = 500.0;
const ROD_BASE: f64 = WINDOW_HEIGHT as f64 - 100.0;
const ROD_TOP: f64 = ROD_BASE - ROD_HEIGHT;
const N_DISCS: u32 = 5;
const N_RODS: u32 = 3;

const COLOR_BACKGROUND: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const COLOR_PLAY_AREA_BACKGROUND: [f32; 4] = [0.1, 0.1, 0.1, 1.0];

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
    center: f64,
    highlighted: bool,
    dropbox_start: f64,
    dropbox_end: f64,
    texture: RodTexture,
}

impl Rod {
    fn render(
        &self,
        c: graphics::Context,
        gl: &mut opengl_graphics::GlGraphics,
        play_area_render_info: &PlayAreaRenderInfo,
    ) {
        let image = graphics::Image::new().rect([
            self.pos_x + play_area_render_info.x,
            self.pos_y + play_area_render_info.y,
            self.width,
            self.height,
        ]);

        image.draw(
            if self.highlighted {
                &self.texture.highlight
            } else {
                &self.texture.normal
            },
            &graphics::DrawState::default(),
            c.transform,
            gl,
        );
    }

    fn pos_in_dropbox(&self, x: f64, _y: f64, pari: &PlayAreaRenderInfo) -> bool {
        x >= self.dropbox_start + pari.x && x <= self.dropbox_end + pari.y
    }
}

struct Disc {
    width: f64,
    value: u32,
    highlighted: bool,
    texture: DiscTexture,
}

impl Disc {
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs, center_x: f64, y: f64) {
        // Calculate actual x based on center x and width
        let x = center_x - self.width / 2.0;

        let pixel_size = DISC_HEIGHT / 12.0;

        let img_left = graphics::Image::new().rect([x, y, pixel_size, DISC_HEIGHT]);
        let img_middle = graphics::Image::new().rect([
            x + pixel_size,
            y,
            self.width - pixel_size * 2.0,
            DISC_HEIGHT,
        ]);
        let img_right =
            graphics::Image::new().rect([x + self.width - pixel_size, y, pixel_size, DISC_HEIGHT]);

        gl.draw(args.viewport(), |c, gl| {
            img_left.draw(
                if self.highlighted {
                    &self.texture.left_highlight
                } else {
                    &self.texture.left
                },
                &graphics::DrawState::default(),
                c.transform,
                gl,
            );

            // Draw middle
            img_middle.draw(
                if self.highlighted {
                    &self.texture.middle_highlight
                } else {
                    &self.texture.middle
                },
                &graphics::DrawState::default(),
                c.transform,
                gl,
            );

            img_right.draw(
                if self.highlighted {
                    &self.texture.right_highlight
                } else {
                    &self.texture.right
                },
                &graphics::DrawState::default(),
                c.transform,
                gl,
            );
        })
    }

    fn pos_in(&self, x: f64, y: f64, pos_center_x: f64, pos_y: f64) -> bool {
        let pos_x = pos_center_x - self.width / 2.0;
        (x >= pos_x && x <= pos_x + self.width) && (y >= pos_y && y <= pos_y + DISC_HEIGHT)
    }

    fn clamped_pos(&self, x: f64, y: f64, pari: &PlayAreaRenderInfo) -> (f64, f64) {
        // TODO remove unnecessary calculation from centre to absolute to centre again
        let (clamped_x, clamped_y) = clamp_rect_position(
            x - self.width / 2.0,
            y,
            self.width,
            DISC_HEIGHT,
            pari.width,
            pari.height,
        );
        (clamped_x + self.width / 2.0 + pari.x, clamped_y + pari.y)
    }

    fn calc_movement_offset(
        &self,
        x: f64,
        y: f64,
        pos_center_x: f64,
        pos_y: f64,
        pari: &PlayAreaRenderInfo,
    ) -> (f64, f64) {
        let offset_x: f64 = pos_center_x - x;
        let offset_y: f64 = pos_y - y;

        (offset_x - pari.x, offset_y - pari.y)
    }
}

fn calc_stacked_y(stack: u32) -> f64 {
    ROD_BASE - DISC_HEIGHT * (stack + 1) as f64
}

fn calc_rod_value(rod: &Vec<Disc>) -> u32 {
    if rod.len() < 1 {
        return u32::MAX;
    } else {
        return rod.last().unwrap().value;
    }
}

#[derive(Copy, Clone)]
struct PlayAreaRenderInfo {
    width: f64,
    height: f64,
    x: f64,
    y: f64,
}

pub struct PlayArea {
    gl: GlGraphics, // OpenGL drawing backend.

    // Elements
    discs: Vec<Vec<Disc>>,
    rods: Vec<Rod>,

    // Mouse position
    mouse_pos_x: f64,
    mouse_pos_y: f64,

    // Movement variables
    moving_disc: Option<Disc>,
    mov_ofst_x: f64,
    mov_ofst_y: f64,
    start_rod: usize,

    last_pari: PlayAreaRenderInfo,
}

impl PlayArea {
    fn render(&mut self, args: &RenderArgs, play_area_render_info: PlayAreaRenderInfo) {
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            // graphics::clear(COLOR_PLAY_AREA_BACKGROUND, gl);

            self.last_pari = play_area_render_info.clone();

            let window_size = args.window_size;

            graphics::rectangle(
                COLOR_PLAY_AREA_BACKGROUND,
                [
                    play_area_render_info.x,
                    play_area_render_info.y,
                    play_area_render_info.width,
                    play_area_render_info.height,
                ],
                c.transform,
                gl,
            );

            // Render all rods
            for rod in self.rods.iter() {
                rod.render(c, gl, &play_area_render_info);
            }

            // Render all discs
            for (i_rod, rod) in self.discs.iter().enumerate() {
                for (i, disc) in rod.iter().enumerate() {
                    disc.render(
                        gl,
                        args,
                        self.rods[i_rod].center + play_area_render_info.x,
                        calc_stacked_y(i as u32) + play_area_render_info.y,
                    );
                }
            }

            // Render moving disc
            if !matches!(self.moving_disc, None) {
                let (clamped_x, clamped_y) = self.moving_disc.as_ref().unwrap().clamped_pos(
                    self.mouse_pos_x + self.mov_ofst_x,
                    self.mouse_pos_y + self.mov_ofst_y,
                    &play_area_render_info,
                );

                self.moving_disc
                    .as_ref()
                    .unwrap()
                    .render(gl, args, clamped_x, clamped_y)
            }
        });
    }

    fn update(&mut self, _args: &UpdateArgs) {}

    fn mouse_moved(&mut self, pos: &[f64; 2]) {
        // Save mouse position
        self.mouse_pos_x = pos[0];
        self.mouse_pos_y = pos[1];

        // Higlight rods
        if !matches!(self.moving_disc, None) {
            for (i_rod, rod) in self.rods.iter_mut().enumerate() {
                rod.highlighted = false;
                if rod.pos_in_dropbox(self.mouse_pos_x, self.mouse_pos_y, &self.last_pari) {
                    if calc_rod_value(&self.discs[i_rod])
                        >= self.moving_disc.as_mut().unwrap().value
                    {
                        rod.highlighted = true;
                    }
                }
            }
        } else {
            // Highlight discs
            for (i_rod, rod) in self.discs.iter_mut().enumerate() {
                if !rod.is_empty() {
                    let last_disc_i = rod.len() - 1;

                    rod[last_disc_i].highlighted = rod[last_disc_i].pos_in(
                        self.mouse_pos_x,
                        self.mouse_pos_y,
                        self.rods[i_rod].center + self.last_pari.x,
                        calc_stacked_y(last_disc_i as u32) + self.last_pari.y,
                    )
                }
            }
        }
    }

    fn mouse_button_pressed(&mut self, button: &MouseButton) {
        if *button == MouseButton::Left {
            // Check if mouse has been clicked inside disc

            for (i_rod, rod) in self.discs.iter_mut().enumerate() {
                if !rod.is_empty() {
                    let last_disc = rod.last().unwrap();
                    let last_disc_i = rod.len() - 1;

                    // If mouse was clicked on a disc
                    if last_disc.pos_in(
                        self.mouse_pos_x,
                        self.mouse_pos_y,
                        self.rods[i_rod].center + self.last_pari.x,
                        calc_stacked_y(last_disc_i as u32) + self.last_pari.y,
                    ) {
                        (self.mov_ofst_x, self.mov_ofst_y) = last_disc.calc_movement_offset(
                            self.mouse_pos_x,
                            self.mouse_pos_y,
                            self.rods[i_rod].center + self.last_pari.x,
                            calc_stacked_y(last_disc_i as u32) + self.last_pari.y,
                            &self.last_pari,
                        );

                        // Remove disc and save to current disc
                        // rod[last_disc_i].highlighted = false;
                        self.moving_disc = Some(rod.remove(last_disc_i));
                        self.start_rod = i_rod as usize;
                    }
                }
            }
        }
    }
    fn mouse_button_released(&mut self, button: &MouseButton) {
        if *button == MouseButton::Left {
            if !matches!(self.moving_disc, None) {
                let mut currently_moving: Disc = std::mem::take(&mut self.moving_disc).unwrap();

                let mut drop: Option<usize> = None;

                for (i_rod, rod) in self.rods.iter_mut().enumerate() {
                    if rod.pos_in_dropbox(self.mouse_pos_x, self.mouse_pos_y, &self.last_pari)
                        && calc_rod_value(&self.discs[i_rod]) >= currently_moving.value
                    {
                        drop = Some(i_rod);
                        break;
                    }
                }

                currently_moving.highlighted = false;

                if matches!(drop, None) {
                    self.discs[self.start_rod].push(currently_moving);
                } else {
                    self.discs[drop.unwrap()].push(currently_moving);
                }

                self.moving_disc = None;

                for rod in self.rods.iter_mut() {
                    rod.highlighted = false;
                }
            }
        }
    }
}

fn place_play_area(window_width: f64, window_height: f64) -> PlayAreaRenderInfo {
    PlayAreaRenderInfo {
        width: WINDOW_WIDTH as f64,
        height: WINDOW_HEIGHT as f64,
        x: window_width / 2.0 - WINDOW_WIDTH as f64 / 2.0,
        y: window_height / 2.0 - WINDOW_HEIGHT as f64 / 2.0,
    }
}

struct App {
    gl: GlGraphics,
    play_area: PlayArea,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            graphics::clear(COLOR_BACKGROUND, gl);

            self.play_area.render(
                args,
                place_play_area(args.window_size[0], args.window_size[1]),
            );
        });
    }

    fn update(&mut self, args: &UpdateArgs) {}

    fn mouse_moved(&mut self, pos: &[f64; 2]) {
        self.play_area.mouse_moved(pos);
    }

    fn mouse_button_pressed(&mut self, button: &MouseButton) {
        self.play_area.mouse_button_pressed(button);
    }

    fn mouse_button_released(&mut self, button: &MouseButton) {
        self.play_area.mouse_button_released(button);
    }
}

fn init_discs(n_discs: u32, n_rods: u32) -> Vec<Vec<Disc>> {
    let mut discs: Vec<Vec<Disc>> = vec![];
    let mut internal: Vec<Disc> = vec![];

    let width_step = (DISC_WIDTH_MAX - DISC_WIDTH_MIN) / (n_discs - 1) as f64;

    // Add discs to first rod
    for n in (0..n_discs).rev() {
        internal.push(Disc {
            width: DISC_WIDTH_MIN + width_step * n as f64,
            value: n,
            highlighted: false,
            texture: load_disc_texture_color(compute_disc_color(n, n_discs)),
        })
    }
    discs.push(internal);

    // Fill remaining rods
    for _n in 1..n_rods {
        discs.push(vec![]);
    }

    discs
}

fn init_rods(n_rods: u32) -> Vec<Rod> {
    let mut rods: Vec<Rod> = vec![];

    let screen_divs = WINDOW_WIDTH as f64 / n_rods as f64;

    for n in 0..n_rods {
        rods.push(Rod {
            width: ROD_WIDTH,
            height: ROD_HEIGHT,
            pos_x: screen_divs / 2.0 + screen_divs * n as f64 - ROD_WIDTH / 2.0,
            pos_y: ROD_TOP,
            center: screen_divs / 2.0 + screen_divs * n as f64,
            highlighted: false,
            dropbox_start: screen_divs * n as f64,
            dropbox_end: screen_divs * (n + 1) as f64,
            texture: load_rod_texture(),
        })
    }

    rods
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new(WINDOW_TITLE, [WINDOW_WIDTH, WINDOW_HEIGHT])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .resizable(true)
        .build()
        .unwrap();

    // Initialize discs
    let discs = init_discs(N_DISCS, N_RODS);
    let rods = init_rods(N_RODS);

    let mut play_area = PlayArea {
        gl: GlGraphics::new(opengl),
        discs,
        rods,
        moving_disc: None,
        mov_ofst_x: 0.0,
        mov_ofst_y: 0.0,
        mouse_pos_x: 0.0,
        mouse_pos_y: 0.0,
        start_rod: 0,
        last_pari: PlayAreaRenderInfo {
            width: 0.0,
            height: 0.0,
            x: 0.0,
            y: 0.0,
        },
    };

    let mut app = App {
        gl: GlGraphics::new(opengl),
        play_area,
    };

    // Create a new game and run it.

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
            app.mouse_button_pressed(&button)
        }
        if let Some(Button::Mouse(button)) = e.release_args() {
            app.mouse_button_released(&button)
        }

        // Mouse movement events
        e.mouse_cursor(|pos| {
            app.mouse_moved(&pos);
        });
    }
}
