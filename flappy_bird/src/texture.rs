use crate::types::Rect;
use image::{self, RgbaImage};
use std::path::Path;

pub struct Texture {
    pub image: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub depth: usize,
}
#[allow(dead_code)]
enum AlphaChannel {
    First,
    Last,
}
#[allow(dead_code)]
impl Texture {
    pub fn with_file(path: &Path) -> Self {
        Self::new(image::open(path).expect("Couldn't load image").into_rgba8())
    }
    pub fn new(image: RgbaImage) -> Self {
        let (width, height) = image.dimensions();
        let mut image = image.into_vec();
        premultiply(&mut image, 4, AlphaChannel::Last);
        Self {
            width: width as usize,
            height: height as usize,
            depth: 4,
            image: image.to_vec(),
        }
    }
    // for fonts
    pub fn from_vec(vec: Vec<u8>, width: usize, height: usize, depth: usize) -> Self {
        Self {
            width: width,
            height: height,
            depth: depth,
            image: vec,
        }
    }
    pub fn depth(&self) -> usize {
        self.depth
    }
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    pub fn pitch(&self) -> usize {
        self.width * self.depth
    }
    pub fn buffer(&self) -> &[u8] {
        &self.image
    }
    pub fn valid_frame(&self, frame: Rect) -> bool {
        0 <= frame.x
            && (frame.x + frame.w as i32) <= (self.width as i32)
            && 0 <= frame.y
            && (frame.y + frame.h as i32) <= (self.height as i32)
    }
    pub fn convert_to_rgba(&mut self) {
        if self.depth != 1 {
            return;
        }
        let mut i = 0;
        let mut new_image:Vec<u8> = vec![];
        while i < self.width * self.height {
            new_image.push(self.image[i]);
            new_image.push(self.image[i]);
            new_image.push(self.image[i]);
            new_image.push(255);
            i+=1;
        }
        self.image = new_image;
        self.depth = 4;
    }
}

pub fn stack_horizontal(textures: Vec<Texture>) -> Texture {
    let mut new_image: Vec<u8> = vec![];
    if textures.len() == 0 {
        return Texture{image: new_image, width: 0, height: 0, depth: 0};
    }
    
    let mut row = 0;
    let mut texture = 0;
    let mut column;
    let mut channel;
    let mut total_width = 0;
    let mut max_height = textures[0].height;
    //let mut current_col_start = 0;
    let sample = &textures[0];
    //let width = sample.width;
    let texture_count = textures.len();
    
    let depth = sample.depth;
    
    while texture < texture_count {
        total_width += textures[texture].width;
        if max_height < textures[texture].height {
            max_height = textures[texture].height;
        }
        texture += 1;
    }
    let height = max_height;

    while row < height {
        texture = 0;
        while texture < texture_count {
            column = 0;
            let current_width = textures[texture].width;
            if (textures[texture].height >= height - row) {
                let row_offset = height - textures[texture].height;
                while column < current_width {
                    channel = 0;
                    while channel < depth {
                        new_image.push(textures[texture].image[(row - row_offset) * current_width * depth + column * depth + channel]);
                        channel += 1;
                    }
                    column += 1;
                }
            } else {
                while column < current_width {
                    channel = 0;
                    while channel < depth {
                        new_image.push(0);
                        channel += 1;
                    }
                    column += 1;
                }
            }
            
            //current_col_start += textures[texture].width;
            texture += 1;
        }
        row += 1;
    }
    Texture{image: new_image, width: total_width, height: height, depth: depth}

}

fn premultiply(img: &mut [u8], depth: usize, alpha: AlphaChannel) {
    match alpha {
        AlphaChannel::First => {
            for px in img.chunks_exact_mut(depth) {
                let a = px[0] as f32 / 255.0;
                for component in px[1..].iter_mut() {
                    *component = (*component as f32 * a).round() as u8;
                }
                // swap around to rgba8888
                let a = px[0];
                px[0] = px[1];
                px[1] = px[2];
                px[2] = px[3];
                px[3] = a;
            }
        }
        AlphaChannel::Last => {
            for px in img.chunks_exact_mut(depth) {
                let a = *px.last().unwrap() as f32 / 255.0;
                for component in px[0..(depth - 1)].iter_mut() {
                    *component = (*component as f32 * a) as u8;
                }
                // already rgba8888
            }
        }
    }
}