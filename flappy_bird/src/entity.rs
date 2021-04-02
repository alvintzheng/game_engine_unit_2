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

// pub struct Bird {
//     pub body: Entity,
//     pub wing: Sprite,
// }

// impl Bird {
//     pub fn new(hitbox:Mobile, sprite:Sprite, gravity:bool) -> Self {
//         Self {hitbox, sprite, gravity}
//     }
// }

//implement drawbird so that wing position depends on body position