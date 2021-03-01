use crate::types::Rect;
use std::time::{Duration, SystemTime};


pub struct Animation {
    // Do this for the exercise today!
    // You'll want to know the frames involved and the timing for each frame
    // But then there's also dynamic data, which might live in this struct or might live somewhere else
    // An Animation/AnimationState split could be fine, if AnimationState holds the start time and the present frame (or just the start time) and possibly a reference to the Animation
    // but there are lots of designs that will work!
    pub frame_width: u16,
    pub frame_height: u16,
    start_x: i32,
    pub start_y: i32,
    frame_count: u16,
    pub current_frame: u16,
    frame_duration: Duration,
    last_frame_time: SystemTime,
    active: bool,
}

#[allow(dead_code)]
impl Animation {
    // Should hold some data...
    // Be used to decide what frame to use...
    // And sprites can be updated based on that information.
    // Or we could give sprites an =animation= field instead of a =frame=!
    // Could have a query function like current_frame(&self, start_time:usize, now:usize, speedup_factor:usize)
    // Or could be ticked in-place
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
        }
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
                self.current_frame = (self.current_frame + 1) % self.frame_count;}},
            Err(error) => {println!("oops");},
        }
    }
}