use graphics::*;
use opengl_graphics::{GlGraphics, GlyphCache};
use piston::input::*;
use crate::screens::{Screen, ScreenState};
use super::popup::Popup;

const GRID_MIN: i32 = -5;
const GRID_MAX: i32 = 5;
const POINT_SIZE: f64 = 5.0;
const GRID_LINE_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];  // White
const PLAYER_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];     // Red
const TRIANGLE_SIZE: f64 = POINT_SIZE * 1.8;
const TRIANGLE_INSET: f64 = POINT_SIZE * 0.2;
const TEXT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const TEXT_POS_X: f64 = 20.0;
const TEXT_POS_Y: f64 = 30.0;

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn to_string(&self) -> &'static str {
        match self {
            Direction::Up => "FACING: UP",
            Direction::Down => "FACING: DOWN",
            Direction::Left => "FACING: LEFT",
            Direction::Right => "FACING: RIGHT",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Point {
    x: i32,
    y: i32,
    movable: bool,
    facing: Option<Direction>,
}

pub struct GameScreen {
    player: Point,
    obstacles: Vec<Point>,
    grid_scale: f64,
    popups: Vec<Popup>,
}

impl GameScreen {
    pub fn new() -> Self {
        let player = Point { 
            x: 0, 
            y: 0, 
            movable: true,
            facing: Some(Direction::Right),
        };
        
        let mut obstacles = Vec::new();
        
        // Add boundary points
        for x in GRID_MIN..=GRID_MAX {
            obstacles.push(Point { x, y: GRID_MIN, movable: false, facing: None });
            obstacles.push(Point { x, y: GRID_MAX, movable: false, facing: None });
        }
        for y in GRID_MIN..=GRID_MAX {
            obstacles.push(Point { x: GRID_MIN, y, movable: false, facing: None });
            obstacles.push(Point { x: GRID_MAX, y, movable: false, facing: None });
        }

        // Add other obstacles
        obstacles.extend(vec![
            Point { x: 2, y: 2, movable: false, facing: None },
            Point { x: -2, y: -2, movable: false, facing: None },
            Point { x: -2, y: 2, movable: false, facing: None },
            Point { x: 2, y: -2, movable: false, facing: None },
        ]);

        GameScreen {
            player,
            obstacles,
            grid_scale: 30.0,
            popups: Vec::new(),
        }
    }

    fn try_move_player(&mut self, dx: i32, dy: i32) {
        // Update facing direction based on movement attempt
        self.player.facing = Some(match (dx, dy) {
            (0, 1) => Direction::Up,
            (0, -1) => Direction::Down,
            (-1, 0) => Direction::Left,
            (1, 0) => Direction::Right,
            _ => self.player.facing.unwrap_or(Direction::Right),
        });

        let new_x = self.player.x + dx;
        let new_y = self.player.y + dy;

        // Check if new position would be on a boundary
        if new_x <= GRID_MIN || new_x >= GRID_MAX || new_y <= GRID_MIN || new_y >= GRID_MAX {
            self.show_boundary_message();
            return;
        }

        // Create the new position
        let new_pos = Point { 
            x: new_x, 
            y: new_y, 
            movable: true,
            facing: None,
        };

        // Check if the new position is occupied by any obstacle
        if !self.obstacles.iter().any(|obstacle| obstacle.x == new_pos.x && obstacle.y == new_pos.y) {
            self.player.x = new_x;
            self.player.y = new_y;
        }
    }

    fn show_boundary_message(&mut self) {
        self.popups.push(Popup::new_text_box(
            "Boundary in the way".to_string(),
            2.0  // Display for 2 seconds
        ));
    }

    fn update_popups(&mut self) {
        self.popups.retain_mut(|popup| {
            popup.update();
            popup.active
        });
    }

    fn grid_to_screen(&self, x: i32, y: i32, window_size: [f64; 2]) -> [f64; 2] {
        let center_x = window_size[0] / 2.0;
        let center_y = window_size[1] / 2.0;
        
        [
            center_x + (x as f64 * self.grid_scale),
            center_y - (y as f64 * self.grid_scale),
        ]
    }

    fn draw_grid(&self, c: &Context, g: &mut GlGraphics, window_size: [f64; 2]) {
        for x in GRID_MIN..=GRID_MAX {
            let start = self.grid_to_screen(x, GRID_MIN, window_size);
            let end = self.grid_to_screen(x, GRID_MAX, window_size);
            line(GRID_LINE_COLOR, 0.5, [start[0], start[1], end[0], end[1]], c.transform, g);
        }

        for y in GRID_MIN..=GRID_MAX {
            let start = self.grid_to_screen(GRID_MIN, y, window_size);
            let end = self.grid_to_screen(GRID_MAX, y, window_size);
            line(GRID_LINE_COLOR, 0.5, [start[0], start[1], end[0], end[1]], c.transform, g);
        }
    }

    fn draw_point(&self, point: &Point, color: [f32; 4], c: &Context, g: &mut GlGraphics, window_size: [f64; 2]) {
        let pos = self.grid_to_screen(point.x, point.y, window_size);
        
        if point.movable && point.facing.is_some() {
            // For the player, draw a directional triangle
            let (sin, cos) = match point.facing.unwrap() {
                Direction::Right => (0.0, 1.0),    // Point right
                Direction::Up => (-1.0, 0.0),      // Point up
                Direction::Left => (0.0, -1.0),    // Point left
                Direction::Down => (1.0, 0.0),     // Point down
            };

            let tip_x = pos[0] + cos * TRIANGLE_SIZE;
            let tip_y = pos[1] + sin * TRIANGLE_SIZE;
            
            let base_x = pos[0] - cos * TRIANGLE_INSET;
            let base_y = pos[1] - sin * TRIANGLE_INSET;
            
            let half_base = TRIANGLE_SIZE * 0.5;
            let base1_x = base_x - sin * half_base;
            let base1_y = base_y + cos * half_base;
            let base2_x = base_x + sin * half_base;
            let base2_y = base_y - cos * half_base;

            let triangle = [
                [tip_x, tip_y],
                [base1_x, base1_y],
                [base2_x, base2_y],
            ];

            polygon(color, &triangle, c.transform, g);
        } else {
            ellipse(
                color,
                [pos[0] - POINT_SIZE, pos[1] - POINT_SIZE, POINT_SIZE * 2.0, POINT_SIZE * 2.0],
                c.transform,
                g
            );
        }
    }

    fn draw_direction_text(&self, c: &Context, g: &mut GlGraphics, glyphs: &mut GlyphCache) {
        if let Some(direction) = self.player.facing {
            text::Text::new_color(TEXT_COLOR, 16)
                .draw(
                    direction.to_string(),
                    glyphs,
                    &c.draw_state,
                    c.transform.trans(TEXT_POS_X, TEXT_POS_Y),
                    g,
                )
                .unwrap_or_else(|e| eprintln!("Error drawing text: {}", e));
        }
    }
}

impl Screen for GameScreen {
    fn draw(&mut self, c: &Context, g: &mut GlGraphics, glyphs: &mut GlyphCache, window_size: [f64; 2]) {
        clear([0.0, 0.0, 0.0, 1.0], g);
        self.draw_grid(c, g, window_size);
        
        for obstacle in &self.obstacles {
            self.draw_point(obstacle, GRID_LINE_COLOR, c, g, window_size);
        }
        
        self.draw_point(&self.player, PLAYER_COLOR, c, g, window_size);
        self.draw_direction_text(c, g, glyphs);

        for popup in &self.popups {
            popup.draw(c, g, glyphs, window_size);
        }
    }

    fn update(&mut self) -> Option<ScreenState> {
        self.update_popups();
        None
    }

    fn handle_input(&mut self, input: &Input) -> Option<ScreenState> {
        match input {
            Input::Button(ButtonArgs {state: ButtonState::Press, button: Button::Keyboard(key), ..}) => {
                match key {
                    Key::W => self.try_move_player(0, 1),
                    Key::S => self.try_move_player(0, -1),
                    Key::A => self.try_move_player(-1, 0),
                    Key::D => self.try_move_player(1, 0),
                    Key::Escape => return Some(ScreenState::Pause),
                    _ => {}
                }
            }
            _ => {}
        }
        None
    }
}
