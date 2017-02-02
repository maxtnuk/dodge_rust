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
use self::object::*;

pub mod object;
// MAXINUM number of other particles
const MAXINUM: u32 = 200;
// time limit of channel
const TIME_LIMIT: u64 = 2000;
fn main() {
    let opengls = OpenGL::V4_5;
    // set piston window
    let mut window: PistonWindow = WindowSettings::new("dodge_game", [800, 800])
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
