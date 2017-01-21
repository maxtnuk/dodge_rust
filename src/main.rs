extern crate piston_window;
extern crate rand;
extern crate stopwatch;
use piston_window::*;
use piston_window::types::Color;
use std::f64::consts::PI;
use std::f64;
use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use rand::{thread_rng, Rng};
use stopwatch::Stopwatch;
// speed of object
enum Speed {
    Go { scala: f64 },
    None,
}
// object spawn side
enum Side {
    None,
    Center,
    Up,
    Down,
    Right,
    Left,
}
// direction of the object
struct Arrow {
    theta: f64,
    sin: f64,
    cos: f64,
}
impl Arrow {
    fn new(theta: f64) -> Self {
        // caculate the sin,cos of the angle
        Arrow {
            theta: theta,
            sin: (PI * theta).sin(),
            cos: (PI * theta).cos(),
        }
    }
}
// object info
pub struct Object {
    // evironment sixe
    background: (u32, u32),
    current_speed: Speed,
    // object's position
    current_state: (f64, f64),
    size: (f64, f64),
    spawn: Side,
    arrow: Arrow,
    color: Color,
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
    // set the environment size
    fn set_place(&mut self, r: &RenderArgs) {
        self.background = (r.width, r.height);
    }
    // set object speed
    pub fn set_speed(&mut self, scala: f64) {
        self.current_speed = Speed::Go { scala: scala };
    }
    // this is for setting the spawn postion
    fn set_pos(&mut self, r: &RenderArgs, pos: Side) {
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
    // set the object color
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    // set position for some other function
    fn inner_set_pos(&mut self, pos: (f64, f64)) {
        self.current_state = pos;
    }//set the direction according to angle (theta: 0.0~2.0)
    pub fn arrow_set(&mut self, theta: f64) {
        self.arrow = Arrow::new(theta);
    }
    // set direction for ramdome
    fn random_arrow_set(&mut self) {
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
    // update the current state
    fn update(&mut self, args: &UpdateArgs) {
        let before_pos = self.current_state;
        match self.current_speed {
            Speed::Go { scala } => {
                self.current_state.0 += scala * self.arrow.cos * args.dt;
                self.current_state.1 += scala * self.arrow.sin * args.dt;
            }
            Speed::None => {}
        };
        // if object hit the wall then reflect
        if !self.is_wall() {
            // this is the reflect function
            self.collide();
            // and set the before position
            self.inner_set_pos(before_pos);
        }
    }
    // manage the key control
    fn move_it(&mut self, control: &Button) {
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
    // if object goes outside the window then it turns the false
    fn is_wall(&mut self) -> bool {
        let available_x = self.background.0 as f64 - self.size.0;
        let available_y = self.background.1 as f64 - self.size.1;
        between(0.0, available_x, self.current_state.0) &&
        between(0.0, available_y, self.current_state.1)
    }
    // set the new direction according to current position
    fn collide(&mut self) {
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
        // set theta range in 0.0~2.0
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
    // if object hit with other object then return true
    fn is_hit(&self, other: &Object) -> bool {
        let (result_x, result_y) = (self.current_state.0 - other.current_state.0,
                                    self.current_state.1 - other.current_state.1);
        match (result_x < 0.0, result_y < 0.0) {
            (false, false) => result_x <= other.size.0 && result_y <= other.size.1,
            (true, false) => result_x.abs() <= self.size.0 && result_y <= other.size.1,
            (false, true) => result_x <= other.size.0 && result_y.abs() <= self.size.1,
            (true, true) => result_x.abs() <= self.size.0 && result_y.abs() <= self.size.1,
        }
    }
}
// return treu if target is in the range
fn between(x: f64, y: f64, target: f64) -> bool {
    x <= target && target <= y
}
// MAXINUM number of other particles
const MAXINUM: u32 = 100;
// time limit of channel
const TIME_LIMIT: u64 = 2000;
fn main() {
    let opengls = OpenGL::V4_5;
    // set piston window
    let mut window: PistonWindow = WindowSettings::new("machine_dodge", [400, 400])
        .opengl(opengls)
        .exit_on_esc(true)
        .build()
        .unwrap();
    let (tx, rx) = mpsc::channel();
    let mut start: bool = true;
    let mut game_end: bool = false;
    let mut count = 0;
    // create main object
    let mut machine = Object::new(20.0, 20.0);
    // create other objects
    let mut obstacles: Vec<Object> = Vec::new();
    let time_limit = Duration::from_millis(TIME_LIMIT);
    // start stopwatch
    let mut sw = Stopwatch::start_new();
    // make the eventloop of window
    while let Some(e) = window.next() {
        // rnader the window
        if let Some(r) = e.render_args() {
            if start {
                // setting the main object
                machine.set_pos(&r, Side::Center);
                machine.set_speed(50.0);
                machine.set_color([0.0, 1.0, 0.0, 1.0]);
                start = false;
            }
            // if game is over then run this code and restart
            if game_end {
                start = true;
                game_end = false;
                count = 0;
                obstacles.clear();
                let result = sw.elapsed_ms();
                let (t_x, b_x) = (result / 1000, result % 1000);
                // print the surviving time
                println!("THE END \ntime:{}.{}", t_x, b_x);
                sw.restart();
                continue;
            }
            if count < MAXINUM {
                let rng = thread_rng().gen_range(1, 4);
                count += rng;
                // spawn randome number of obstacles
                for _ in 0..rng {
                    let tx = tx.clone();
                    thread::spawn(move || {
                        let mut obstacle = Object::new(10.0, 10.0);
                        let position = match thread_rng().gen_range(0, 4) {
                            0 => Side::Up,
                            1 => Side::Right,
                            2 => Side::Down,
                            3 => Side::Left,
                            _ => {
                                panic!("system error");
                            }
                        };
                        obstacle.set_pos(&r, position);
                        obstacle.set_speed(30.0);
                        obstacle.set_color([0.0, 0.0, 1.0, 1.0]);
                        obstacle.random_arrow_set();
                        tx.send(obstacle).unwrap();
                    });
                    let temp = match rx.recv_timeout(time_limit) {
                        Ok(result) => result,
                        Err(_) => {
                            panic!("send fail");
                        }
                    };
                    obstacles.push(temp);
                }
            }
            // draw the object
            window.draw_2d(&e, |c, g| {
                // set postion of main object
                let transform = c.transform.trans(machine.current_state.0, machine.current_state.1);
                machine.set_place(&r);
                // clear the background
                clear([0.0, 0.0, 0.0, 1.0], g);
                // draw the rectangle
                Rectangle::new(machine.color).draw([0.0, 0.0, machine.size.0, machine.size.1],
                                                   &c.draw_state,
                                                   transform,
                                                   g);
                for obstacle in obstacles.iter_mut() {
                    // set postion of obstacles
                    let transform = c.transform
                        .trans(obstacle.current_state.0, obstacle.current_state.1);
                    obstacle.set_place(&r);
                    // draw the rectangle
                    Rectangle::new(obstacle.color)
                        .draw([0.0, 0.0, obstacle.size.0, obstacle.size.1],
                              &c.draw_state,
                              transform,
                              g);
                }
            });
        }
        // if there is the press event then run this code
        if let Some(b) = e.press_args() {
            machine.move_it(&b);
        }
        // if there is the update event then run this code
        if let Some(u) = e.update_args() {
            machine.update(&u);
            for obstacle in obstacles.iter_mut() {
                {
                    // compare withe main and obstacles
                    game_end = machine.is_hit(&obstacle);
                    if game_end {
                        break;
                    }
                }
                obstacle.update(&u);
            }
        }
    }
}
