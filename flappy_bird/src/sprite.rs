use crate::texture::Texture;
use crate::types::{Rect, Vec2i};
use std::rc::Rc;
use crate::Animation;

pub struct Sprite {
    image: Rc<Texture>,
    pub frame: Rect, // Maybe better to use a type that can't have a negative origin
    // Or use =animation:Animation= instead of a frame field
    pub position: Vec2i,
    //include imageheight/width?
    pub animations: Vec<Animation>,
    pub current_animation: usize,
}

#[allow(dead_code)]
impl Sprite {
    pub fn new(image: &Rc<Texture>, frame: Rect, position: Vec2i) -> Self {
        Self {
            image: Rc::clone(image),
            frame,
            position,
            animations: Vec::new(),
            current_animation: 0,
        }
    }
    pub fn set_animation(&mut self, index: usize) {
        self.current_animation = index;
    }
}

pub trait DrawSpriteExt {
    fn draw_sprite(&mut self, s: &mut Sprite);
}

use crate::screen::Screen;
impl<'fb> DrawSpriteExt for Screen<'fb> {
    fn draw_sprite(&mut self, s: &mut Sprite) {
        // This works because we're only using a public method of Screen here,
        // and the private fields of sprite are visible inside this module
        let ca = &mut s.animations[s.current_animation];
        let x_pos = ca.current_frame * ca.frame_width;
        let new_frame = Rect{x: x_pos as i32, y: ca.start_y, w: ca.frame_width, h: ca.frame_height};
        ca.tick();

        self.bitblt(&s.image, new_frame, s.position);
    }
}