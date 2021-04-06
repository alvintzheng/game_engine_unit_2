use crate::types::Rect;
use std::time::{Duration, SystemTime};

#[allow(dead_code)]
pub struct Animation {

    pub frame_width: u16,
    pub frame_height: u16,
    start_x: i32,
    pub start_y: i32,
    frame_count: u16,
    pub current_frame: u16,
    frame_duration: Duration,
    last_frame_time: SystemTime,
    active: bool,
    pub do_loop: bool,
}

#[allow(dead_code)]
impl Animation {

    pub fn new (frame_width: u16, frame_height: u16, start_x: i32, start_y: i32, frame_count: u16)-> Self {
        Self {
            frame_width: frame_width,
            frame_height: frame_height,
            start_x: start_x,
            start_y: start_y,
            frame_count: frame_count,
            current_frame: 0,
            frame_duration: Duration::from_millis(500),
            last_frame_time: SystemTime::now(),
            active: false,
            do_loop: true,
        }
    }
    pub fn set_duration(&mut self, duration: Duration) {
        self.frame_duration = duration;
    }

    pub fn calc_frame(&self) -> Rect {
        let x_pos = self.start_x + (self.current_frame as i32) * (self.frame_width as i32);

        return Rect{x: x_pos, y: self.start_y, w: self.frame_width, h: self.frame_height};
    }

    pub fn tick (&mut self) {
        let now = SystemTime::now();
        let time_elapsed = now.duration_since(self.last_frame_time);
        match time_elapsed {
            Ok(duration) => {if duration.as_millis() > self.frame_duration.as_millis() {
                self.last_frame_time = now;
                if self.do_loop {
                    self.current_frame = (self.current_frame + 1) % self.frame_count;
                } else {
                    if self.current_frame < self.frame_count - 1 {
                        self.current_frame += 1;
                    }
                }
                }},
            Err(_) => {println!("oops");},
        }
    }
}