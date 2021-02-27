// We can pull in definitions from elsewhere in the crate!
use crate::texture::Texture;
use crate::types::{Rect, Rgba, Vec2i};
pub struct Screen<'fb> {
    framebuffer: &'fb mut [u8],
    width: usize,
    height: usize,
    depth: usize,
    //position: Vec2i,
}
impl<'fb> Screen<'fb> {
    // Call =wrap= every frame; that means the camera position will need to be stored in the game state

    pub fn wrap(framebuffer: &'fb mut [u8], width: usize, height: usize, depth: usize) -> Self {
        Self {
            framebuffer,
            width,
            height,
            depth,
        }
    }
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    pub fn bounds(&self) -> Rect {
        Rect {
            x: 0,
            y: 0,
            w: self.width as u16,
            h: self.height as u16,
        }
    }
    // Our old, slow friend draw_at, now with super scrolling powers!
    #[inline(always)]
    pub fn draw_at(&mut self, col: Rgba, Vec2i(x, y): Vec2i) {
        // The rest is about the same
        if x < 0 || (self.width as i32) <= x || y < 0 || (self.height as i32) <= y {
            return;
        }
        assert_eq!(self.depth, 4);
        // Now x and y are within framebuffer bounds so go ahead and draw
        let c = [col.0, col.1, col.2, col.3];
        let idx = y * self.width as i32 * self.depth as i32 + x * self.depth as i32;
        assert!(idx >= 0);
        let idx = idx as usize;
        self.framebuffer[idx..(idx + self.depth)].copy_from_slice(&c);
    }
    // Clear's the same...
    pub fn clear(&mut self, col: Rgba) {
        let c = [col.0, col.1, col.2, col.3];
        for px in self.framebuffer.chunks_exact_mut(4) {
            px.copy_from_slice(&c);
        }
    }

    //from Slack comments
    pub fn bitblt(&mut self, src: &Texture, from: Rect, Vec2i(to_x, to_y): Vec2i) {
        let (tw, th) = src.size();
        assert!(0 <= from.x);
        assert!(from.x < tw as i32);
        assert!(0 <= from.y);
        assert!(from.y < th as i32);

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
        for (row_a, row_b) in src_buf[(src_pitch * ((from.y + y_skip) as usize))
            ..(src_pitch * ((from.y + y_count) as usize))]
            .chunks_exact(src_pitch)
            .zip(
                self.framebuffer[(dst_pitch * ((to_y + y_skip) as usize))
                    ..(dst_pitch * ((to_y + y_count) as usize))]
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
