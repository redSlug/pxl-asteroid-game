extern crate pxl;
extern crate rand;

use pxl::*;
use std::collections::HashMap;
use rand::prelude::*;

const TARGET_ASTEROID_COUNT: usize = 8;
const MAX_ASTEROID_COUNT: usize = 64;

struct Shape {
    position: Coordinate,
    color:    Pixel,
    kind:     ShapeKind,
    speed: u8,
    is_alive: bool,
}

#[derive(Copy, Clone)]
enum ShapeKind {
    Rect{width: u8, height: u8},
}

impl Shape {
    fn collides(&self, other: &Shape) -> bool {
        use ShapeKind::*;

        match (self.kind, other.kind) {
            (Rect{width, height}, Rect{width: other_width, height: other_height}) => {
                return self.position.x < other.position.x.saturating_add(other_width) &&
                    self.position.x.saturating_add(width) > other.position.x &&
                    self.position.y < other.position.y.saturating_add(other_height) &&
                    height.saturating_add(self.position.y) > other.position.y
            }
        }
    }

    fn draw(&self, pixels: &mut [Pixel]) {
        match self.kind {
            ShapeKind::Rect{width, height} => {
                let x = self.position.x as i32;
                let y = self.position.y as i32;
                for dy in 0..height as i32 {
                    for dx in 0..width as i32{
                        if let Some(coordinate) = self.position.add_delta(dx - height as i32 / 2, dy - width as i32 / 2) {
                            pixels[coordinate.pixel_index()] = self.color;
                        }
                    }
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
struct Coordinate {
    x: u8,
    y: u8
}

impl Coordinate {
    fn pixel_index(self) -> usize {
        self.y as usize * DISPLAY_COLUMNS as usize + self.x as usize
    }

    fn add_delta(self, x: i32, y: i32) -> Option<Coordinate>{
        let new_x = self.x as i32 + x;
        let new_y = self.y as i32 + y;
        if new_x < 0 || new_x >= DISPLAY_COLUMNS as i32 || new_y < 0 || new_y >= DISPLAY_ROWS as i32 {
            None
        } else {
            Some(Coordinate{x: new_x as u8, y: new_y as u8})
        }
    }
}

struct Game {
    background_color: Pixel,
    player: Shape,
    buttons_state: HashMap<Button, ButtonState>,
    asteroids: Vec<Shape>,
    collisions_count: usize
}


impl Game {
    fn pressed(&self, button: Button) -> bool {
        self.buttons_state.get(&button) == Some(&ButtonState::Pressed)
    }
}

impl Program for Game {
    fn new() -> Game {
        Game {
            background_color: Pixel{red: 255, green: 255, blue: 255},
            player: Shape{
                position: Coordinate{x: 127, y: (DISPLAY_ROWS - 5) as u8},
                color: Pixel{red: 255, green: 0, blue: 125},
                kind: ShapeKind::Rect {width: 10, height: 10},
                speed: 1,
                is_alive: true,
            },
            buttons_state: HashMap::new(),
            asteroids: Vec::new(),
            collisions_count: 0
        }
    }
    fn title(&self) -> &str {
        "asteroid game!"
    }
    fn tick(&mut self, events: &[Event]) {
        for event in events {
            match event {
                Event::Button {button, state} => {
                    self.buttons_state.insert(*button, *state);
                }
                _ => {}
            }
        }
        if self.pressed(Button::Down) {
            self.player.position.y = self.player.position.y.saturating_add(3);
        }
        if self.pressed(Button::Up) {
            self.player.position.y = self.player.position.y.saturating_sub(3);
        }
        if self.pressed(Button::Left) {
            self.player.position.x = self.player.position.x.saturating_sub(3);
        }
        if self.pressed(Button::Right) {
            self.player.position.x = self.player.position.x.saturating_add(3);
        }
        for asteroid in self.asteroids.iter_mut() {
            if !asteroid.is_alive {
                continue;
            }

            // asteroids move towards us
            asteroid.position.y = asteroid.position.y.saturating_add(asteroid.speed);

            // background gets darker with each colission
            if asteroid.collides(&self.player) {
                self.collisions_count += 1;
                self.background_color.green = self.background_color.green.saturating_sub(1);
                self.background_color.red = self.background_color.red.saturating_sub(1);
                self.background_color.blue = self.background_color.blue.saturating_sub(1);

                // remove asteroid from vec
                asteroid.is_alive = false;
            }
        }

        if self.asteroids.len() < TARGET_ASTEROID_COUNT {
            self.asteroids.push(Shape{
                speed: rand::thread_rng().gen_range(1, 5),
                position: Coordinate{x: random() , y:0},
                color: Pixel{red: random(), green: random(), blue: 0},
                kind: ShapeKind::Rect {width: 4, height: 4},
                is_alive: true
            })
        }
        
        // clean out asteroids
        self.asteroids.retain(|asteroid|asteroid.position.y < 255);

    }
    fn render(&mut self, pixels: &mut [Pixel]) {
        // board
        for p in pixels.iter_mut() {
            *p = self.background_color;
        }

        // player
        self.player.draw(pixels);

        // asteroids
        for asteroid in &self.asteroids {
            if asteroid.is_alive {
                asteroid.draw(pixels);
            }
        }
    }
}


fn main() {
    run::<Game>();
}
