use crate::collision::*;
use crate::sprite::*;
use crate::types::*;

pub struct Entity {
    pub hitbox: Mobile,
    pub sprite: Sprite,
    pub gravity: bool,
}

impl Entity {
    pub fn new(hitbox:Mobile, sprite:Sprite, gravity:bool) -> Self {
        Self {hitbox, sprite, gravity}
    }
}

pub trait DrawEntityExt {
    fn draw_entity(&mut self, s: &mut Entity);
}

use crate::screen::Screen;
impl<'fb> DrawEntityExt for Screen<'fb> {
    fn draw_entity(&mut self, e: &mut Entity) {
        e.sprite.position = Vec2i(e.hitbox.rect.x, e.hitbox.rect.y);
        self.draw_sprite(&mut e.sprite);
    }
}

pub struct Bird {
    pub body: Entity,
    pub wing: Sprite,
}

pub trait DrawBirdExt {
    fn draw_bird(&mut self, s: &mut Bird);
}

impl<'fb> DrawBirdExt for Screen<'fb> {
    fn draw_bird(&mut self, b: &mut Bird) {
        self.draw_entity(&mut b.body);
        let Vec2i(x,y) = b.body.sprite.position;
        b.wing.position = Vec2i(x, y - 25);
        self.draw_sprite(&mut b.wing);
    }
}

//implement drawbird so that wing position depends on body position