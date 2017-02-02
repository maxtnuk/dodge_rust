use std::f64::consts::PI;
use std::f64;
use rand::{thread_rng, Rng};
use piston_window::types::Color;
use piston_window::{RenderArgs, UpdateArgs, Button, Key};
pub enum Speed {
    Go { scala: f64 },
    None,
}
pub enum Side {
    None,
    Center,
    Up,
    Down,
    Right,
    Left,
}
pub struct Arrow {
    theta: f64,
    sin: f64,
    cos: f64,
}
impl Arrow {
    fn new(theta: f64) -> Self {
        Arrow {
            theta: theta,
            sin: (PI * theta).sin(),
            cos: (PI * theta).cos(),
        }
    }
    pub fn show_theta(&self) -> f64 {
        self.theta
    }
}
pub struct Object {
    background: (u32, u32),
    current_speed: Speed,
    pub current_state: (f64, f64),
    pub size: (f64, f64),
    spawn: Side,
    pub arrow: Arrow,
    pub color: Color,
}
impl Object {
    pub fn new(width: f64, height: f64) -> Self {
        Object {
            background: (0, 0),
            current_state: (0.0, 0.0),
            current_speed: Speed::None,
            size: (width, height),
            spawn: Side::None,
            arrow: Arrow::new(0.0),
            color: [0.0, 0.0, 0.0, 0.0],
        }
    }
    pub fn set_place(&mut self, r: &RenderArgs) {
        self.background = (r.width, r.height);
    }
    pub fn set_speed(&mut self, scala: f64) {
        self.current_speed = Speed::Go { scala: scala };
    }
    pub fn set_pos(&mut self, r: &RenderArgs, pos: Side) {
        let position_x = thread_rng().gen_range(0.0, r.width as f64 - self.size.0);
        let position_y = thread_rng().gen_range(0.0, r.height as f64 - self.size.1);
        self.current_state = match pos {
            Side::Up => (position_x, 0.0),
            Side::Right => (r.width as f64 - self.size.0, position_y),
            Side::Down => (position_x, r.height as f64 - self.size.1),
            Side::Left => (0.0, position_y),
            Side::Center => (r.width as f64 / 2.0, r.height as f64 / 2.0),
            _ => panic!("system error"),
        };
        self.spawn = pos;
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    fn inner_set_pos(&mut self, pos: (f64, f64)) {
        self.current_state = pos;
    }
    pub fn arrow_set(&mut self, theta: f64) {
        self.arrow = Arrow::new(theta);
    }
    pub fn random_arrow_set(&mut self) {
        let theta = match self.spawn {
            Side::Up => thread_rng().gen_range(0.0, 1.0),
            Side::Right => thread_rng().gen_range(0.5, 1.5),
            Side::Down => thread_rng().gen_range(1.0, 2.0),
            Side::Left => thread_rng().gen_range(-0.5, 0.5),
            Side::Center => thread_rng().gen_range(0.0, 2.0),
            _ => panic!("system error"),
        };
        self.arrow = Arrow::new(theta);
    }
    pub fn update(&mut self, args: &UpdateArgs) {
        let before_pos = self.current_state;
        match self.current_speed {
            Speed::Go { scala } => {
                self.current_state.0 += scala * self.arrow.cos * args.dt;
                self.current_state.1 += scala * self.arrow.sin * args.dt;
            }
            Speed::None => {}
        };
        if !self.is_wall() {
            self.collide();
            self.inner_set_pos(before_pos);
        }
    }
    pub fn is_wall(&mut self) -> bool {
        let available_x = self.background.0 as f64 - self.size.0;
        let available_y = self.background.1 as f64 - self.size.1;
        between(0.0, available_x, self.current_state.0) &&
        between(0.0, available_y, self.current_state.1)
    }
    pub fn collide(&mut self) {
        let available_x = self.background.0 as f64 - self.size.0;
        let available_y = self.background.1 as f64 - self.size.1;
        let before_theta = self.arrow.theta;
        let plane_vec = match (between(0.0, available_x, self.current_state.0),
                               between(0.0, available_y, self.current_state.1)) {
            (false, true) => 1.0,
            (true, false) => 2.0,
            (false, false) => 2.0 * before_theta,
            (true, true) => {
                panic!("system error");
            }
        };
        let mut result = plane_vec - before_theta;
        loop {
            if result < 0.0 {
                result += 2.0;
                continue;
            } else if result > 2.0 {
                result -= 2.0;
                continue;
            } else {
                break;
            }
        }
        self.arrow = Arrow::new(result);
    }
    pub fn is_hit(&self, other: &Object) -> bool {
        let (result_x, result_y) = (self.current_state.0 - other.current_state.0,
                                    self.current_state.1 - other.current_state.1);
        match (result_x < 0.0, result_y < 0.0) {
            (false, false) => result_x <= other.size.0 && result_y <= other.size.1,
            (true, false) => result_x.abs() <= self.size.0 && result_y <= other.size.1,
            (false, true) => result_x <= other.size.0 && result_y.abs() <= self.size.1,
            (true, true) => result_x.abs() <= self.size.0 && result_y.abs() <= self.size.1,
        }
    }
    // manage the key control
    pub fn move_it(&mut self, control: &Button) {
        let theta = match *control {
            Button::Keyboard(key) => {
                match key {
                    Key::Right => 0.0,
                    Key::Left => 1.0,
                    Key::Up => 1.5,
                    Key::Down => 0.5,
                    _ => self.arrow.theta,
                }
            }
            _ => self.arrow.theta,
        };
        self.arrow = Arrow::new(theta);
    }
}
fn between(x: f64, y: f64, target: f64) -> bool {
    x <= target && target <= y
}
