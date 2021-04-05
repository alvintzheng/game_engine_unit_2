use crate::texture::Texture;
use crate::types::{Rect, Vec2i};
use crate::Screen;
use std::rc::Rc;


pub const TILE_SZ: usize = 16;


/// An actual tilemap
#[derive(Clone)]
pub struct Guessing {
    //coordinates of square hit
    pub hitsquare: Vec2i,
    /// vec of adjacent guesses to make
    pub guessesvec: Vec<Vec2i>,
    //whether guessesvec is empty, ie whether smart guessing or random guessing
    pub smartguessing: bool,
    /// Which tilemap
    pub tilemap: Rc<Tilemap>,

}

impl Guessing {
    pub fn new(
        position: Vec2i,
        dims: (usize, usize),
        tileset: &Rc<Tileset>,
        map: Vec<usize>,
    ) -> Self {
        assert_eq!(dims.0 * dims.1, map.len(), "Tilemap is the wrong size!");
        assert!(
            map.iter().all(|tid| tileset.contains(TileID(*tid))),
            "Tilemap refers to nonexistent tiles"
        );
        Self {
            position,
            dims,
            tileset: Rc::clone(tileset),
            map: map.into_iter().map(TileID).collect(),
        }
    }
    //input: window coordinates
    //output: TileID - type of tile at that position in the map

    pub fn tile_id_at(&self, Vec2i(x, y): Vec2i) -> TileID {

            // Translate into map coordinates
            //let x = (x - self.position.0) / TILE_SZ as i32;
            //let y = (y - self.position.1) / TILE_SZ as i32;
            let x = (x - self.position.0) / 32 as i32; //32 coordinates per tile
            let y = (y - self.position.1) / 32 as i32; //32 coordinates per tile
            assert!(
                x >= 0 && x < self.dims.0 as i32,
                "Tile X coordinate {} out of bounds {}",
                x,
                self.dims.0
            );
            assert!(
                y >= 0 && y < self.dims.1 as i32,
                "Tile Y coordinate {} out of bounds {}",
                y,
                self.dims.1
            );
            //self.map[y as usize * self.dims.0 + x as usize]
            self.map[y as usize + x as usize]

    }
    //input: window coordinates
    //output: TileID as usize
    pub fn tile_id_num_at(&self, Vec2i(x, y): Vec2i) -> usize {
        // Translate into map coordinates
        //t x = (x - self.position.0) / TILE_SZ as i32;
        //let y = (y - self.position.1) / TILE_SZ as i32;
        let x = (x - self.position.0) / 32 as i32; //32 coordinates per tile
        let y = (y - self.position.1) / 32 as i32; //32 coordinates per tile
        assert!(
            x >= 0 && x < self.dims.0 as i32,
            "Tile X coordinate {} out of bounds {}",
            x,
            self.dims.0
        );
        assert!(
            y >= 0 && y < self.dims.1 as i32,
            "Tile Y coordinate {} out of bounds {}",
            y,
            self.dims.1
        );
        (y as usize + x as usize)
    }

    pub fn size(&self) -> (usize, usize) {
        self.dims
    }

    //input: window coordinates
    //output: Tile
    pub fn tile_at(&self, posn: Vec2i) -> Tile {
        self.tileset[self.tile_id_at(posn)]
    }

    pub fn set_tile_at(&mut self, Vec2i(x, y): Vec2i, id: usize) {
        //pub fn set_tile_at(mut self, Vec2i(x, y): Vec2i, id: usize) {
        // Translate into map coordinates

        let x = (x - self.position.0) / 32 as i32; //32 coordinates per tile
        let y = (y - self.position.1) / 32 as i32;
        println!("x: {}, y: {})", x, y);

        assert!(
            x >= 0 && x < self.dims.0 as i32,
            "Tile X coordinate {} out of bounds {}",
            x,
            self.dims.0
        );
        assert!(
            y >= 0 && y < self.dims.1 as i32,
            "Tile Y coordinate {} out of bounds {}",
            y,
            self.dims.1
        );
        self.map[y as usize * self.dims.0 + x as usize] = TileID(id);
    }

    //from Slack comments
    pub fn draw(&self, screen: &mut Screen) {
        let Rect {
            x: sx,
            y: sy,
            w: sw,
            h: sh,
        } = screen.bounds();
        // We'll draw from the topmost/leftmost visible tile to the bottommost/rightmost visible tile.
        // The camera combined with out position and size tell us what's visible.
        // leftmost tile: get camera.x into our frame of reference, then divide down to tile units
        // Note that it's also forced inside of 0..self.size.0
        let left = ((sx - self.position.0) / TILE_SZ as i32)
            .max(0)
            .min(self.dims.0 as i32) as usize;
        // rightmost tile: same deal, but with screen.x + screen.w plus a little padding to be sure we draw the rightmost tile even if it's a bit off screen.

        //let right = ((sx+((sw+TILE_SZ) as i32)-self.position.0) / TILE_SZ as i32).max(0).min(self.dims.0 as i32) as usize;
        let right = ((sx + ((sw + TILE_SZ as u16) as i32) - self.position.0) / TILE_SZ as i32)
            .max(0)
            .min(self.dims.0 as i32) as usize;

        // ditto top and bot
        let top = ((sy - self.position.1) / TILE_SZ as i32)
            .max(0)
            .min(self.dims.1 as i32) as usize;

        //let bot = ((sy+((sh+TILE_SZ) as i32)-self.position.1) / TILE_SZ as i32).max(0).min(self.dims.1 as i32) as usize;
        let bot = ((sy + ((sh + TILE_SZ as u16) as i32) - self.position.1) / TILE_SZ as i32)
            .max(0)
            .min(self.dims.1 as i32) as usize;

        // Now draw the tiles we need to draw where we need to draw them.
        // Note that we're zipping up the row index (y) with a slice of the map grid containing the necessary rows so we can avoid making a bounds check for each tile.
        for (y, row) in (top..bot)
            .zip(self.map[(top * self.dims.0)..(bot * self.dims.0)].chunks_exact(self.dims.0))
        {
            // We are in tile coordinates at this point so we'll need to translate back to pixel units and world coordinates to draw.
            let ypx = (y * TILE_SZ) as i32 + self.position.1;
            // Here we can iterate through the column index and the relevant slice of the row in parallel
            for (x, id) in (left..right).zip(row[left..right].iter()) {
                let xpx = (x * TILE_SZ) as i32 + self.position.0;
                let frame = self.tileset.get_rect(*id);

                ///////////////bitblt from screen.rs called here
                screen.bitblt(&self.tileset.texture, frame, Vec2i(xpx, ypx));
            }
        }
    }
}
