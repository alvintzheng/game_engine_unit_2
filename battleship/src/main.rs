use pixels::{Pixels, SurfaceTexture};
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use rand::{thread_rng, Rng};

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
    title_image: Rc<Texture>,
    tilemaps: Vec<Tilemap>, //vector of tilemaps stored in GameState
    //counts of how many ships sunk on both sides to track for end of game
    /////////////compsunk
    ////////////humansunk
}
// seconds per frame
const DT: f64 = 1.0 / 60.0;
const MAPDIM: i32 = 64;
const WIDTH: usize = MAPDIM as usize * 3;
const HEIGHT: usize = MAPDIM as usize * 4;
const DEPTH: usize = 4;

#[derive(Debug, Copy, Clone)]
enum Mode {
    Title,
    Play(Turn),
    Options,
    ScoreBoard, 
    EndGame,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Turn {
    Human,
    Computer,
}

impl Mode {
    // update consumes self and yields a new state (which might also just be self)
    fn update(self, game: &mut GameState, input: &WinitInputHelper) -> Self {
        match self {
            Mode::Title => {

                if input.key_pressed(VirtualKeyCode::P) {
                    Mode::Play(Turn::Human)
                }
                else if input.key_pressed(VirtualKeyCode::Q) {
                    Mode::EndGame
                } else {
                    self
                }
            }
            Mode::Play(pm) => {
                match pm {
                    Turn::Human => {
                        //move update_game to here
                        // if let Some(pm) = pm.update(game, input) {
                        //     Mode::Play(pm);
                        // }
                        println!("human's turn");

                        if input.mouse_pressed(0) {

                            let xcoor = input.mouse().unwrap().0 as i32;
                            let ycoor = input.mouse().unwrap().1 as i32;

                            //change tile at coordinates
                            //was opponent's ship hidden there?
                            if game.tilemaps[0].tile_at(Vec2i(xcoor, ycoor)).opphit {
                                game.tilemaps[0].set_tile_at(Vec2i(xcoor, ycoor), 8) //hit opponent
                            } else { //missed
                                game.tilemaps[0].set_tile_at(Vec2i(xcoor, ycoor), 12) //missed opponent
                            }
                            if input.key_pressed(VirtualKeyCode::Q) {
                                Mode::EndGame
                            }else if input.key_pressed(VirtualKeyCode::O) {
                                Mode::Options
                            }else if input.key_pressed(VirtualKeyCode::S) {
                                Mode::ScoreBoard
                            }else {
                               Mode::Play(Turn::Computer)
                            }
                        }
                        else{
                            Mode::Play(Turn::Human)
                        }
                    }
                    Turn::Computer => {
                        println!("computer's turn");
                        let xcompguess = thread_rng().gen_range(1, WIDTH) as i32; //change range values
                        let ycompguess = thread_rng().gen_range(HEIGHT/2+1, HEIGHT) as i32; //change range values
                        //hits human's ship
                        if game.tilemaps[1].tile_at(Vec2i(xcompguess, ycompguess)).myship {
                            game.tilemaps[1].set_tile_at(Vec2i(xcompguess, ycompguess), 4); //hit human's ship
                            Mode::Play(Turn::Human)
                            ///////compsunk++
                        }
                        //misses human's ship
                        else if game.tilemaps[1].tile_id_num_at(Vec2i(xcompguess, ycompguess))!=4{
                            game.tilemaps[1].set_tile_at(Vec2i(xcompguess, ycompguess), 4); //hit human's ship
                            Mode::Play(Turn::Human)
                        }
                        //already tried that square (tile 4)
                        else {
                           Mode::Play(Turn::Computer) //make another guess
                        }
                    }
                }

                
            }
            Mode::Options => {
                if input.key_pressed(VirtualKeyCode::Q) {
                    Mode::EndGame
                }else if input.key_pressed(VirtualKeyCode::S) {
                    Mode::ScoreBoard
                }else if input.key_pressed(VirtualKeyCode::P) {
                    Mode::Play(Turn::Human) //need to track and save turn and what the board looks like
                } else {
                    self
                }
            }
            Mode::ScoreBoard => {
                if input.key_pressed(VirtualKeyCode::Q) {
                    Mode::EndGame
                }else if input.key_pressed(VirtualKeyCode::O) {
                    Mode::Options
                }else if input.key_pressed(VirtualKeyCode::P) {
                    Mode::Play(Turn::Human) //need to track and save turn and what the board looks like
                } else {
                    self
                }
            }
            Mode::EndGame => {
                if input.key_pressed(VirtualKeyCode::Q) {
                    panic!();
                } else if input.key_pressed(VirtualKeyCode::T) {
                    Mode::Title
                }
                else {
                    self
                }
            }
        }
    }
    fn display(&self, game: &GameState, screen: &mut Screen) {
        match self {
            Mode::Title => {
                //draw a (static?) title
                screen.clear(Rgba(80, 80, 80, 255));
                let display_rect = Rect {
                    x: 0,
                    y: 0,
                    w: 200,
                    h: 210,
                };
                screen.bitblt(&game.title_image, display_rect, Vec2i(0, 0));
            }
            Mode::Play(pm) => {
                // Call screen's drawing methods to render the game state
                screen.clear(Rgba(80, 80, 80, 255));

                //draw each tilemap in vector to screen
                game.tilemaps[0].draw(screen);
                game.tilemaps[1].draw(screen);
            }
            Mode::Options => {
                screen.clear(Rgba(80, 255, 255, 255));
            }
            Mode::ScoreBoard => {
                screen.clear(Rgba(255, 80, 255, 255));
            }
            Mode::EndGame => { // Draw game result?
                screen.clear(Rgba(255, 255, 80, 255));
            }
        }
    }
}

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

    let title_image = Rc::new(Texture::with_file(Path::new("res/logo.png")));

    //create Tileset from tileset.png image
    let boattileset = Rc::new(Tileset {
        tiles: vec![
            //image comprises 16 tiles
            Tile {
                oppgrid: true,
                opphit: false,
                myship: false,
            }, //empty opponent - 0
            Tile {
                oppgrid: false,
                opphit: false,
                myship: false,
            }, //ocean - 1
            Tile {
                oppgrid: false,
                opphit: false,
                myship: false,
            }, //ocean - 2
            Tile {
                oppgrid: true,
                opphit: true,
                myship: false,
            }, //hidden opponent - 3
            Tile {
                oppgrid: false,
                opphit: false,
                myship: false,
            }, //my ship hit - 4
            Tile {
                oppgrid: false,
                opphit: false,
                myship: true,
            }, //single ship - 5
            Tile {
                oppgrid: false,
                opphit: false,
                myship: true,
            }, //double ship 1 - 6
            Tile {
                oppgrid: false,
                opphit: false,
                myship: true,
            }, //double ship 2 - 7
            Tile {
                oppgrid: true,
                opphit: false,
                myship: false,
            }, //hit opponent - 8
            Tile {
                oppgrid: false,
                opphit: false,
                myship: true,
            }, //tall ship 1 - 9
            Tile {
                oppgrid: false,
                opphit: false,
                myship: true,
            }, //pirate ship 1 - 10
            Tile {
                oppgrid: false,
                opphit: false,
                myship: true,
            }, //pirate ship 2 - 11
            Tile {
                oppgrid: true,
                opphit: false,
                myship: false,
            }, //missed opponent - 12
            Tile {
                oppgrid: false,
                opphit: false,
                myship: true,
            }, //tall ship 2 - 13
            Tile {
                oppgrid: false,
                opphit: false,
                myship: true,
            }, //pirate ship 3 - 14
            Tile {
                oppgrid: false,
                opphit: false,
                myship: true,
            }, //pirate ship 4 - 15
        ],
        texture: Rc::new(Texture::with_file(Path::new("tileset.png"))), //bring in image as texture
    });

    // 6 tilemaps, each 4x4 tiles
    //tilemaps join together into a 3x2 map, i.e. 12x8 tile grid
    //opponent's ships

    let oppmap = Tilemap::new(
        Vec2i(0, 0), //location
        (12, 8),
        &boattileset,
        vec![
            3, 0, 0, 0, 0, 0, 3, 0, 0, 0, 3, 3, //3s are hidden opponents
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, //
        ],
    );
    //your ships
    let mymap = Tilemap::new(
        Vec2i(0, MAPDIM * 2), //location
        (12, 8),
        &boattileset,
        vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, 5, 1, 1, //single ship
            1, 1, 6, 7, 1, 1, 1, 1, 1, 1, 1, 1, //double ship
            1, 1, 1, 1, 1, 1, 1, 4, 1, 1, 1, 1, //x mark
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
            10, 11, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
            14, 15, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
        ],
    );

    let mut mode = Mode::Title;

    let mut state = GameState {
        // initial game state...
        tilemaps: vec![oppmap, mymap], //vector of tilemaps
        title_image: title_image,
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

            // change to draw game using state and mode, i.e. mode.draw_game(state)
            //draw_game(&state, &mut screen);
            mode.display(&state, &mut screen);

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

            // change to use mode
            //update_game(&mut state, &input, frame_count);
            mode = mode.update(&mut state, &input);
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
}
/*
fn update_game(state: &mut GameState, input: &WinitInputHelper, frame: usize) {
    // Player control goes here

    //0 == Left
    if input.mouse_pressed(0) {
       ////need set tile function to call here

       //prints twice?
       println!("mouse coordinates: ({}, {})", input.mouse().unwrap().0, input.mouse().unwrap().1);

/*        //tester writing over a whole tilemap
        state.tilemaps[1] = Tilemap::new(
            Vec2i(64, 0),
            (4, 4),
            &state.tilemaps[0].tileset,
            vec![0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 8, 0, 0, 0, 0, 0], //view of opponent
        ); */

        //coordinates are off
        state.tilemaps[0].set_tile_at(Vec2i(input.mouse().unwrap().0 as i32, input.mouse().unwrap().1 as i32), 12);

    }

    if input.key_held(VirtualKeyCode::Right) {}
    if input.key_held(VirtualKeyCode::Left) {}
    if input.key_held(VirtualKeyCode::Up) {}
    if input.key_held(VirtualKeyCode::Down) {}
}
 */
