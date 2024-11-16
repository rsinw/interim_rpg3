extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL, GlyphCache, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::*;
use piston::window::WindowSettings;

mod screens;
use screens::{ScreenManager, ScreenState};
use screens::game::GameScreen;

fn main() {
    let opengl = OpenGL::V3_2;
    let window_size = [800, 600];

    let mut window: GlutinWindow = WindowSettings::new("Game", window_size)
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut gl = GlGraphics::new(opengl);

    let font_path = "C:\\Windows\\Fonts\\Arial.ttf";
    let mut glyphs = GlyphCache::new(
        font_path,
        (),
        TextureSettings::new()
    ).expect("Could not load font");

    let mut screen_manager = ScreenManager::new();
    screen_manager.add_screen(ScreenState::Game, Box::new(GameScreen::new()));

    let mut events = Events::new(EventSettings::new());

    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                screen_manager.draw(&c, g, &mut glyphs, [args.window_size[0] as f64, args.window_size[1] as f64]);
            });
        }

        screen_manager.update();

        if let Some(input) = e.press_args() {
            screen_manager.handle_input(&Input::Button(ButtonArgs {
                state: ButtonState::Press,
                button: input,
                scancode: None,
            }));
        }

        if let Some(pos) = e.mouse_cursor_args() {
            screen_manager.handle_input(&Input::Move(Motion::MouseCursor(pos)));
        }
    }
}
