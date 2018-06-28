extern crate pxl;
extern crate rand;

use pxl::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::f64::consts::PI;
use rand::prelude::*;



const TARGET_ASTEROID_COUNT: u32 = 8;
const SCREEN_SIZE: usize = 256;

fn clamp(n: f32) -> f32 {
    if n < 0.0 {
        0.0
    } else if n > 1.0 {
        1.0
    } else {
        n
    }
}


struct Entity {
    shape: Shape,
}


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
                let self_x= self.position.x.saturating_sub(width / 2);
                let self_y= self.position.y.saturating_sub(height / 2);
                let other_x= other.position.x.saturating_sub(other_width / 2);
                let other_y= other.position.y.saturating_sub(other_height / 2);
                return self_x < other_x.saturating_add(other_width) &&
                    self_x.saturating_add(width) > other_x &&
                    self_y < other_y.saturating_add(other_height) &&
                    height.saturating_add(self_y) > other_y
            }
        }
    }

    fn draw(&self, pixels: &mut [Pixel]) {
        match self.kind {
            ShapeKind::Rect{width, height} => {
                for dy in 0..height as i32 {
                    for dx in 0..width as i32{
                        if let Some(coordinate) = self.position.add_delta(dx - width as i32 / 2, dy - height as i32 / 2) {
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
        self.y as usize * SCREEN_SIZE as usize + self.x as usize
    }

    fn add_delta(self, x: i32, y: i32) -> Option<Coordinate>{
        let new_x = self.x as i32 + x;
        let new_y = self.y as i32 + y;
        if new_x < 0 || new_x >= SCREEN_SIZE as i32 || new_y < 0 || new_y >= SCREEN_SIZE as i32 {
            None
        } else {
            Some(Coordinate{x: new_x as u8, y: new_y as u8})
        }
    }
}

struct Game {
    background_color: Pixel,
    game_title: String,
    game_timer: u32,
    lives: u8,
    player: Shape,
    rest_until: Option<u32>,
    buttons_state: HashMap<Button, ButtonState>,
    // TODO maybe combine these two into one vector
    asteroids: Vec<Shape>,
    mushrooms: Vec<Entity>,
    crystals: Vec<Entity>,
    collisions_count: u32,
    audio: Arc<Mutex<Audio>>,
}


impl Game {
    fn pressed(&self, button: Button) -> bool {
        self.buttons_state.get(&button) == Some(&ButtonState::Pressed)
    }
}

impl Program for Game {
    fn synthesizer(&self) -> Option<Arc<Mutex<Synthesizer>>> {
        Some(self.audio.clone())
    }

    fn dimensions(&self) -> (usize, usize){
        (SCREEN_SIZE, SCREEN_SIZE)
    }
    
    fn new() -> Game {
        Game {
            background_color: Pixel{alpha: 1.0, red: 1.0, green: 1.0, blue: 1.0},
            game_timer: 0,
            lives: 8,
            game_title: String::new(),
            player: Shape{
                position: Coordinate{x: 127, y: (SCREEN_SIZE - 5) as u8},
                color: Pixel{alpha: 1.0, red: 1.0, green: 0.0, blue: 0.5},
                kind: ShapeKind::Rect {width: 10, height: 10},
                speed: 1,
                is_alive: true
            },
            rest_until: None,
            buttons_state: HashMap::new(),
            asteroids: Vec::new(),
            mushrooms: Vec::new(),
            crystals: Vec:: new(),
            collisions_count: 0,
            audio: Arc::new(Mutex::new(Audio::new())),
        }
    }
    fn title(&self) -> &str {
        &self.game_title.as_str()
    }
    fn tick(&mut self, events: &[Event]) {
        if self.lives == 0 {
            self.game_title = format!("Game over! Score:{} ", (self.game_timer / 60).to_string());
            return
        }
        self.game_title = format!("Asteroids! Score:{} Lives:{}", (self.game_timer / 60).to_string(), self.lives);
        self.game_timer += 1;

        let resting = match self.rest_until {
            Some(rest_until) => if rest_until > self.game_timer {
                true
            } else {
                self.rest_until = None;
                false
            },
            None => false
        };

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
        for mushroom in self.mushrooms.iter_mut() {
            if !mushroom.shape.is_alive{
                continue;
            }
            if mushroom.shape.collides(&self.player) {
                self.lives += 1;
                self.audio.lock().unwrap().beep(0.5);
                mushroom.shape.is_alive = false;
            }
        }

        for crystal in self.crystals.iter_mut() {
            if !crystal.shape.is_alive{
                continue;
            }
            if crystal.shape.collides(&self.player) {
                self.rest_until = Some(self.game_timer + 180);
                // Remove all asteroids
                for asteroid in self.asteroids.iter_mut() {
                    asteroid.is_alive = false;
                }
                crystal.shape.is_alive = false;
            }
        }

        for asteroid in self.asteroids.iter_mut() {
            if !asteroid.is_alive {
                continue;
            }

            if asteroid.position.y < asteroid.speed {
                asteroid.is_alive = true;
            }
            // asteroids move towards us
            asteroid.position.y = asteroid.position.y.saturating_add(asteroid.speed);

            // background gets darker with each colission
            if asteroid.collides(&self.player) {
                self.audio.lock().unwrap().beep(0.1);
                self.collisions_count += 1;
                self.background_color.green = clamp(self.background_color.green - 0.04);
                self.background_color.red = clamp(self.background_color.red - 0.04);
                self.background_color.blue = clamp(self.background_color.blue - 0.04);

                // remove asteroid from vec
                asteroid.is_alive = false;

                // subtract life
                self.lives = self.lives.saturating_sub(1);
                // TODO if lives as zero do something
            }
        }

        // add a new mushroom
        if self.game_timer % rand::thread_rng().gen_range(1000, 1200) == 0{
            let shape = Shape{
                speed: 0,
                position: Coordinate{x: random() , y: random()},
                color: Pixel{alpha: 1.0, red: 0.65, green: 0.33, blue: 0.07},
                kind: ShapeKind::Rect {width: 16, height: 4},
                is_alive: true
            };
            self.mushrooms.push(Entity{shape})
        }


        // add a new crystal
        if self.game_timer % 1200 == 0{
            let shape = Shape{
                speed: 0,
                position: Coordinate{x: random() , y: random()},
                color: Pixel{alpha: 1.0, red: 0.41, green: 1.0, blue: 0.99},
                kind: ShapeKind::Rect {width: 17, height: 17}, // TODO maybe give them random sizes
                is_alive: true
            };
            self.crystals.push(Entity{shape})
        }

        let additional_asteroids = self.game_timer / 700;
        if self.asteroids.len() < (TARGET_ASTEROID_COUNT + additional_asteroids) as usize  && !resting {
            self.asteroids.push(Shape{
                speed: rand::thread_rng().gen_range(1, 5),
                position: Coordinate{x: random() , y:0},
                color: Pixel{alpha: 1.0, red: random(), green: random(), blue: 0.0},
                kind: ShapeKind::Rect {width: 4, height: 4},
                is_alive: true
            })
        }

        // clean out asteroids
        self.asteroids.retain(|asteroid|asteroid.position.y < 255 && asteroid.is_alive);
        self.mushrooms.retain(|mushroom|mushroom.shape.position.y < 255 && mushroom.shape.is_alive);
        self.crystals.retain(|crystal|crystal.shape.position.y < 255 && crystal.shape.is_alive);

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

        // mushroom
        for mushroom in &self.mushrooms {
            if mushroom.shape.is_alive {
                mushroom.shape.draw(pixels);
            }
        }

        // mushroom
        for crystal in &self.crystals {
            if crystal.shape.is_alive {
                crystal.shape.draw(pixels);
            }
        }

    }
}

struct Voice {
    volume: f32,
    kind: VoiceKind,
    end_time: f64,
}

impl Voice {
    fn sample(&mut, time: f64) -> Sample {
        unimplemented!();
    }
}

enum VoiceKind {
    Sin, // sine wave // like original NES synth, fixed body of 6 voices
    Square, // square wave
    Sawtooth, // sawtooth wave
    Noise, // white noise (random values between -1 and 1)
}

struct Audio {
    current_time: f64,
    end_time: f64,
    voices: Vec<Voice>,
}

impl Audio {
    fn new() -> Audio {
        Audio{current_time: 0.0, end_time: 0.0, voices: Vec::new()}
    }

    // in game:
    // audio.lock().unwrap().play(0.5, 0.25, VoiceKind::Sin);

    fn play(&mut self, duration: f64, volume: f64, kind: VoiceKind) {
        self.voices.push(Voice{volume, kind, end_time: self.curent_time + duration});
    }

    fn beep(&mut self, duration: f64){
        self.end_time = self.current_time + duration;
    }
}

impl Synthesizer for Audio {
    // variable length contiguous slice &[]
    // samples size is number of samples played
    // samples_played as f64 / SAMPLES_PER_SECOND as f64 -> seconds passed

    fn synthesize(&mut self, samples_played: u64, samples: &mut [Sample]) {


        // Polyphonic synthesizer

        // go over voices, find the active voices that are still playing (t < voice.end_time)
        // for all active voices, add the voice's sample to the output sample

        // write a value to every every sample in the sample buffer

        let mut t = samples_played as f64 / SAMPLES_PER_SECOND as f64;

        for s in samples {
            // left and right speakers
            let frequency = 440.0; // hz, i.e. samples per second
            let radians = t * frequency * 2.0 * PI;
            let volume = 0.5;
            if t < self.end_time {
                let amplitude_of_waveform = (radians * frequency).sin() as f32 * volume; // -1 to 1
                s.left = amplitude_of_waveform; // becomes a big number if there are many voices
                s.right = amplitude_of_waveform;
            } else {
                s.left = 0.0;
                s.right = 0.0;
            }

            t += 1.0 / SAMPLES_PER_SECOND as f64;

        }

        // remove all voices that are done playing, i.e t >= voice.end_time using retain

        self.current_time = t;
    }
}

fn main() {
    run::<Game>();
}
