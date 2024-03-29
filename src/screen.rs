// We can pull in definitions from elsewhere in the crate!
use crate::texture::Texture;
use crate::types::{Rect, Rgba, Vec2i};
pub struct Screen<'fb> {
    framebuffer: &'fb mut [u8],
    width: usize,
    height: usize,
    depth: usize,
    position: Vec2i,
}
#[allow(dead_code)]
impl<'fb> Screen<'fb> {
    // Call =wrap= every frame; that means the camera position will need to be stored in the game state
    pub fn wrap(framebuffer: &'fb mut [u8], width: usize, height: usize, depth: usize, position:Vec2i) -> Self {
        Self {
            framebuffer,
            width,
            height,
            depth,
            position
        }
    }
    // pub fn size(&self) -> (usize, usize) {
    //     (self.width, self.height)
    // }
    pub fn bounds(&self) -> Rect {
        Rect{x:self.position.0, y:self.position.1, w:self.width as u16, h:self.height as u16}
    }
    // Our old, slow friend draw_at, now with super scrolling powers!
    // #[inline(always)]
    // pub fn draw_at(&mut self, col: Rgba, Vec2i(x,y) : Vec2i) {
    //     let x = x - self.position.0;
    //     let y = y - self.position.1;
    //     // The rest is about the same
    //     if x < 0 || (self.width as i32) <= x || y < 0 || (self.height as i32) <= y {
    //         return;
    //     }
    //     assert_eq!(self.depth, 4);
    //     // Now x and y are within framebuffer bounds so go ahead and draw
    //     let c = [col.0, col.1, col.2, col.3];
    //     let idx = y * self.width as i32 * self.depth as i32 + x * self.depth as i32;
    //     assert!(idx>=0);
    //     let idx = idx as usize;
    //     self.framebuffer[idx..(idx+self.depth)].copy_from_slice(&c);
    // }
    // Clear's the same...
    pub fn clear(&mut self, col: Rgba) {
        let c = [col.0, col.1, col.2, col.3];
        for px in self.framebuffer.chunks_exact_mut(4) {
            px.copy_from_slice(&c);
        }
    }
    // Rect needs a translation to start
    pub fn rect(&mut self, r: Rect, col: Rgba) {
        let c = [col.0, col.1, col.2, col.3];
        // Here's the translation
        let r = Rect{x:r.x-self.position.0, y:r.y-self.position.1, ..r};
        // And the rest is just the same
        let x0 = r.x.max(0).min(self.width as i32) as usize;
        let x1 = (r.x + r.w as i32).max(0).min(self.width as i32) as usize;
        let y0 = r.y.max(0).min(self.height as i32) as usize;
        let y1 = (r.y + r.h as i32).max(0).min(self.height as i32) as usize;
        let depth = self.depth;
        let pitch = self.width * depth;
        for row in self.framebuffer[(y0 * pitch)..(y1 * pitch)].chunks_exact_mut(pitch) {
            for p in row[(x0 * depth)..(x1 * depth)].chunks_exact_mut(depth) {
                p.copy_from_slice(&c);
            }
        }
    }
    // Ditto line
    pub fn line(&mut self, Vec2i(x0, y0): Vec2i, Vec2i(x1, y1): Vec2i, col: Rgba) {
        let col = [col.0, col.1, col.2, col.3];
        // translate translate
        let x0 = x0 - self.position.0;
        let y0 = y0 - self.position.1;
        // translate translate
        let x1 = x1 - self.position.0;
        let y1 = y1 - self.position.1;
        // Now proceed as we were
        let mut x = x0;
        let mut y = y0;
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let width = self.width as i32;
        let height = self.height as i32;
        while x != x1 || y != y1 {
            if 0 <= x && x < width && 0 <= y && y < height {
                // TODO this bounds check could in theory be avoided with
                // the unsafe get_unchecked, but maybe better not...
                self.framebuffer[(y as usize * self.width * self.depth + x as usize * self.depth)
                    ..(y as usize * self.width * self.depth + (x as usize + 1) * self.depth)]
                    .copy_from_slice(&col);
                // We couldn't just clamp x0/y0 and x1/y1 into bounds, because then
                // we might change the slope of the line.
            }
            let e2 = 2 * err;
            if dy <= e2 {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn rect_outline(&mut self, r: Rect, col: Rgba) {
        self.line(Vec2i(r.x, r.y), Vec2i(r.x + r.w as i32, r.y), col);
        self.line(Vec2i(r.x, r.y + r.h as i32), Vec2i(r.x + r.w as i32, r.y + r.h as i32), col);
        self.line(Vec2i(r.x, r.y), Vec2i(r.x, r.y + r.h as i32), col);
        self.line(Vec2i(r.x + r.w as i32, r.y as i32), Vec2i(r.x + r.w as i32, r.y + r.h as i32), col);
    }
    // Bitblt too begins with a translation
    pub fn bitblt(&mut self, src:&Texture, from: Rect, Vec2i(to_x, to_y): Vec2i) {
        let (tw,th) = src.size();
        assert!(0 <= from.x);
        assert!(from.x < tw as i32);
        assert!(0 <= from.y);
        assert!(from.y < th as i32);
        let to_x = to_x - self.position.0;
        let to_y = to_y - self.position.1;
        if (to_x + from.w as i32) < 0
            || (self.width as i32) <= to_x
            || (to_y + from.h as i32) < 0
            || (self.height as i32) <= to_y
        {
            return;
        }
        let depth = self.depth;
        assert_eq!(depth, src.depth());
        let src_pitch = src.pitch();
        let dst_pitch = self.width * depth;
        // All this rigmarole is just to avoid bounds checks on each pixel of the blit.
        // We want to calculate which row/col of the src image to start at and which to end at.
        // This way there's no need to even check for out of bounds draws.
        let y_skip = to_y.max(0) - to_y;
        let x_skip = to_x.max(0) - to_x;
        let y_count = (to_y + from.h as i32).min(self.height as i32) - to_y;
        let x_count = (to_x + from.w as i32).min(self.width as i32) - to_x;
        let src_buf = src.buffer();
        for (row_a, row_b) in src_buf[(src_pitch * ((from.y + y_skip) as usize))..(src_pitch * ((from.y + y_count) as usize))]
            .chunks_exact(src_pitch)
            .zip(
                self.framebuffer[(dst_pitch * ((to_y + y_skip) as usize))..(dst_pitch * ((to_y + y_count) as usize))]
                    .chunks_exact_mut(dst_pitch),
            )
        {
            let to_cols = row_b
                [(depth * (to_x + x_skip) as usize)..(depth * (to_x + x_count) as usize)]
                .chunks_exact_mut(depth);
            let from_cols = row_a
                [(depth * (from.x + x_skip) as usize)..(depth * (from.x + x_count) as usize)]
                .chunks_exact(depth);
            // Composite over, assume premultiplied rgba8888
            for (to, from) in to_cols.zip(from_cols) {
                let ta = to[3] as f32 / 255.0;
                let fa = from[3] as f32 / 255.0;
                for i in 0..3 {
                    to[i] = from[i].saturating_add((to[i] as f32 * (1.0 - fa)).round() as u8);
                }
                to[3] = ((fa + ta * (1.0 - fa)) * 255.0).round() as u8;
            }
        }
    }
}