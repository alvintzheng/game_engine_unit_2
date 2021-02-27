use pixels::{Pixels, SurfaceTexture};
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

// Whoa what's this?
// Mod without brackets looks for a nearby file.
mod screen;
// Then we can use as usual.  The screen module will have drawing utilities.
use screen::Screen;
// Collision will have our collision bodies and contact types
mod collision;
// Lazy glob imports
use collision::*;
// Texture has our image loading and processing stuff
mod texture;
use texture::Texture;

// And we'll put our general purpose types like color and geometry here:
mod types;
use types::*;

mod tiles;
use tiles::*;

// Now this main module is just for the run-loop and rules processing.
struct GameState {
    tilemaps: Vec<Tilemap>, //vector of tilemaps stored in GameState
}
// seconds per frame
const DT: f64 = 1.0 / 60.0;
const MAPDIM: i32 = 64;
const WIDTH: usize = MAPDIM as usize * 3;
const HEIGHT: usize = MAPDIM as usize * 4;
const DEPTH: usize = 4;

fn main() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Anim2D")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture).unwrap()
    };

    //create Tileset from tileset.png image
    let boattileset = Rc::new(Tileset {
        tiles: vec![
            //image comprises 16 tiles
            Tile { solid: false },
            Tile { solid: false },
            Tile { solid: false },
            Tile { solid: false },
            Tile { solid: true },
            Tile { solid: true },
            Tile { solid: true },
            Tile { solid: true },
            Tile { solid: true },
            Tile { solid: true },
            Tile { solid: true },
            Tile { solid: true },
            Tile { solid: true },
            Tile { solid: true },
            Tile { solid: true },
            Tile { solid: true },
        ],
        texture: Rc::new(Texture::with_file(Path::new("tileset.png"))), //bring in image as texture
    });

    // 6 tilemaps, each 4x4 tiles
    //tilemaps join together into a 3x2 map, i.e. 12x8 tile grid
    //opponent's ships
    let oppmap0 = Tilemap::new(
        Vec2i(0, 0),
        (4, 4),
        &boattileset,
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], //view of opponent
    );
    let oppmap1 = Tilemap::new(
        Vec2i(MAPDIM, 0),
        (4, 4),
        &boattileset,
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    );
    let oppmap2 = Tilemap::new(
        Vec2i(MAPDIM * 2, 0),
        (4, 4),
        &boattileset,
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0], //x mark
    );
    let oppmap3 = Tilemap::new(
        Vec2i(0, MAPDIM),
        (4, 4),
        &boattileset,
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    );
    let oppmap4 = Tilemap::new(
        Vec2i(MAPDIM, MAPDIM),
        (4, 4),
        &boattileset,
        vec![0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0], //check mark
    );
    let oppmap5 = Tilemap::new(
        Vec2i(MAPDIM * 2, MAPDIM),
        (4, 4),
        &boattileset,
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    );
    //your ships
    let map0 = Tilemap::new(
        Vec2i(0, MAPDIM * 2),
        (4, 4),
        &boattileset,
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 5], //single ship
    );
    let map1 = Tilemap::new(
        Vec2i(MAPDIM, MAPDIM * 2),
        (4, 4),
        &boattileset,
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 6, 7, 1, 1, 1, 1], //double ship
    );
    let map2 = Tilemap::new(
        Vec2i(MAPDIM * 2, MAPDIM * 2),
        (4, 4),
        &boattileset,
        vec![1, 1, 1, 1, 1, 1, 4, 1, 1, 1, 1, 1, 1, 1, 1, 1], //x mark
    );
    let map3 = Tilemap::new(
        Vec2i(0, MAPDIM * 3),
        (4, 4),
        &boattileset,
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    );
    let map4 = Tilemap::new(
        Vec2i(MAPDIM, MAPDIM * 3),
        (4, 4),
        &boattileset,
        vec![1, 1, 1, 1, 10, 11, 1, 1, 14, 15, 1, 1, 1, 1, 1, 1], //pirate ship
    );
    let map5 = Tilemap::new(
        Vec2i(MAPDIM * 2, MAPDIM * 3),
        (4, 4),
        &boattileset,
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    );

    let mut state = GameState {
        // initial game state...
        tilemaps: vec![
            oppmap0, oppmap1, oppmap2, oppmap3, oppmap4, oppmap5, map0, map1, map2, map3, map4,
            map5,
        ], //vector of tilemaps
    };
    // How many frames have we simulated?
    let mut frame_count: usize = 0;
    // How many unsimulated frames have we saved up?
    let mut available_time = 0.0;
    // Track beginning of play
    let start = Instant::now();
    // Track end of the last frame
    let mut since = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH);
            screen.clear(Rgba(0, 0, 0, 0));

            draw_game(&state, &mut screen);

            // Flip buffers
            if pixels.render().is_err() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Rendering has used up some time.
            // The renderer "produces" time...
            available_time += since.elapsed().as_secs_f64();
        }
        // Handle input events
        if input.update(event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            // Resize the window if needed
            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }
        }
        // And the simulation "consumes" it
        while available_time >= DT {
            // Eat up one frame worth of time
            available_time -= DT;

            update_game(&mut state, &input, frame_count);

            // Increment the frame counter
            frame_count += 1;
        }
        // Request redraw
        window.request_redraw();
        // When did the last frame end?
        since = Instant::now();
    });
}

fn draw_game(state: &GameState, screen: &mut Screen) {
    // Call screen's drawing methods to render the game state
    screen.clear(Rgba(80, 80, 80, 255));

    //draw each tilemap in vector to screen
    state.tilemaps[0].draw(screen);
    state.tilemaps[1].draw(screen);
    state.tilemaps[2].draw(screen);
    state.tilemaps[3].draw(screen);
    state.tilemaps[4].draw(screen);
    state.tilemaps[5].draw(screen);
    state.tilemaps[6].draw(screen);
    state.tilemaps[7].draw(screen);
    state.tilemaps[8].draw(screen);
    state.tilemaps[9].draw(screen);
    state.tilemaps[10].draw(screen);
    state.tilemaps[11].draw(screen);
}

fn update_game(state: &mut GameState, input: &WinitInputHelper, frame: usize) {
    // Player control goes here
    if input.key_held(VirtualKeyCode::Right) {
        //state.scroll.0 += 2;
    }
    if input.key_held(VirtualKeyCode::Left) {}
    if input.key_held(VirtualKeyCode::Up) {}
    if input.key_held(VirtualKeyCode::Down) {}
    // Update player position

    // Detect collisions: Generate contacts

    // Handle collisions: Apply restitution impulses.

    // Update game rules: What happens when the player touches things?
}
