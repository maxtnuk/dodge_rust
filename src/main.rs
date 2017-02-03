extern crate piston_window;
extern crate carboxyl;
extern crate rand;
extern crate stopwatch;

use piston_window::*;
use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use rand::{thread_rng, Rng};
use stopwatch::Stopwatch;
use carboxyl::{Sink, Signal};

use self::object::*;

pub mod object;
// MAXINUM number of other particles
const MAXINUM: u32 = 200;
// time limit of channel
const TIME_LIMIT: u64 = 2000;

enum Control {
    During(i32),
    End,
}
fn main() {
    let opengls = OpenGL::V4_5;
    // set piston window
    let mut window: PistonWindow = WindowSettings::new("dodge_game", [800, 800])
        .opengl(opengls)
        .exit_on_esc(true)
        .build()
        .unwrap();
    let (tx, rx) = mpsc::channel();
    let mut game_status = Control::During(-1);
    let time_limit = Duration::from_millis(TIME_LIMIT);
    let sink: Sink<f64> = Sink::new();
    let mut stream_vec: Vec<Signal<bool>> = Vec::new();
    // create main object
    let mut machine = Object::new(20.0, 20.0);
    // create other objects
    let mut obstacles: Vec<Object> = Vec::new();
    // start stopwatch
    let mut sw = Stopwatch::start_new();
    // make the eventloop of window
    while let Some(e) = window.next() {
        // rnader the window
        if let Some(r) = e.render_args() {
            match game_status {
                Control::During(ref mut count) => {
                    if *count < 0 {
                        machine.set_pos(&r, Side::Center);
                        machine.set_speed(50.0);
                        machine.set_color([0.0, 1.0, 0.0, 1.0]);
                        *count += 1;
                    } else if *count < MAXINUM as i32 {
                        let rng = thread_rng().gen_range(1, 4);
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
                            stream_vec.push(sink.stream()
                                .fold(false, |_, x| {
                                    let index = *count as usize;
                                    let before_pos = obstacles[index].current_state;
                                    obstacles[index].move_self(x);
                                    if !obstacles[index].is_wall() {
                                        obstacles[index].collide();
                                        obstacles[index].inner_set_pos(before_pos);
                                    }
                                    obstacles[index].is_hit(&machine)
                                }));
                            *count += 1;
                        }
                    }
                }
                Control::End => {
                    game_status = Control::During(-1);
                    obstacles.clear();
                    let result = sw.elapsed_ms();
                    let (t_x, b_x) = (result / 1000, result % 1000);
                    // print the surviving time
                    println!("THE END \ntime:{}.{}", t_x, b_x);
                    sw.restart();
                    continue;
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
            sink.send(u.dt);
            for obs in stream_vec.iter() {
                {
                    // compare withe main and obstacles
                    if obs.sample() {
                        game_status = Control::End;
                    }
                }
            }
        }
    }
}
