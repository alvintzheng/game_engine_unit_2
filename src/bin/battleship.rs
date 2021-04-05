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
extern crate savefile;
use savefile::prelude::*;


#[macro_use]
extern crate savefile_derive;

// Whoa what's this?
// Mod without brackets looks for a nearby file.
// Then we can use as usual.  The screen module will have drawing utilities.
use unit2::screen::Screen;
// Collision will have our collision bodies and contact types
// Lazy glob imports
use unit2::collision::*;
// Texture has our image loading and processing stuff
use unit2::texture::Texture;

// And we'll put our general purpose types like color and geometry here:
use unit2::types::*;
use unit2::tiles::*;
use unit2::sound::*;

// Now this main module is just for the run-loop and rules processing.
#[derive(Savefile)]
struct GameState {
    title_image: Rc<Texture>,
    tilemaps: Vec<Tilemap>, //vector of tilemaps stored in GameState
    //counts of how many ships sunk on both sides to track for end of game
    compsunk: usize,
    humansunk: usize,
}

struct GameData {
    sound: Sound,
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
    Reset,
    EndGame,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Turn {
    Human,
    Computer,
}

impl Mode {
    // update consumes self and yields a new state (which might also just be self)
    fn update(self, game: &mut GameState, data: &mut GameData, input: &WinitInputHelper) -> Self {
        match self {
            Mode::Title => {

                if input.key_pressed(VirtualKeyCode::P) {
                    Mode::Play(Turn::Human)
                }
                else if input.key_pressed(VirtualKeyCode::Q) {
                    Mode::EndGame
                } else if input.key_pressed(VirtualKeyCode::O) {
                        Mode::Options
                }else if input.key_pressed(VirtualKeyCode::R) {
                        Mode::Reset
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

                        //check if computer won
                        //"path statement with no effect"
                        /* if game.humansunk == 2 { 
                            println!("You lost!");
                            Mode::EndGame
                        } */

                        //these dont respond anymore
                        if input.key_pressed(VirtualKeyCode::Q) {
                            Mode::EndGame
                        }else if input.key_pressed(VirtualKeyCode::O) {
                            Mode::Options
                        }else if input.key_pressed(VirtualKeyCode::S) {
                            Mode::ScoreBoard
                        }else if input.key_pressed(VirtualKeyCode::R) {
                            Mode::Reset
                        }else if input.mouse_pressed(0) {
                            
                            println!("human's turn");

                            let xcoor = input.mouse().unwrap().0 as i32;
                            let ycoor = input.mouse().unwrap().1 as i32;

                            //change tile at coordinates
                            //was opponent's ship hidden there?
                            if game.tilemaps[0].tile_at(Vec2i(xcoor, ycoor)).opphit {
                                data.sound.play_sound("hit".to_string());
                                game.compsunk = game.compsunk + 1;
                                println!("compsunk: {}", game.compsunk);
                                game.tilemaps[0].set_tile_at(Vec2i(xcoor, ycoor), 8); //hit opponent
                            } else { //missed
                                data.sound.play_sound("splash".to_string());
                                game.tilemaps[0].set_tile_at(Vec2i(xcoor, ycoor), 12); //missed opponent
                            }

                            save_game(&game);
                            let reloaded_game = load_game();
                            *game = reloaded_game; 
                            //assert_eq!(reloaded_game.name,"Steve".to_string());

                            //check if human won
                            //endgame doesnt work
                            //"path statement with no effect"
                            /*  if game.compsunk == 2 {
                                println!("You won!");
                                Mode::EndGame;
                            } */


                            //this one works
                            Mode::Play(Turn::Computer)
                            
                        }
                        //this one works
                        else{
                            Mode::Play(Turn::Human)
                        }
                    }
                    Turn::Computer => {

                        println!("computer's turn");

                        //if guesses.smartguessing(){
                            //Vec2i theguess= guesses.makeguess()
                            //game.tilemaps[1].set_tile_at(theguess, 4);
                            //return array of guesses to make instead of just one at a time?
                        //}

                        //else

                        ///////seems to not be getting the whole field
                        //random guess
                        //let xcompguess = thread_rng().gen_range(1, WIDTH) as i32;
                        // let sleep_duration = time::Duration::from_millis(2000); 
                        // thread::sleep(sleep_duration);
                        let xcompguess = thread_rng().gen_range(1, WIDTH+191) as i32; 
                        //let ycompguess = thread_rng().gen_range(HEIGHT/2+1, HEIGHT) as i32;
                        let ycompguess = thread_rng().gen_range(HEIGHT/2+1, HEIGHT+127) as i32;  
                        //hits human's ship
                        ////////are none going into here?
                        if game.tilemaps[1].tile_at(Vec2i(xcompguess, ycompguess)).myship {
                            data.sound.play_sound("hit".to_string());
                            game.humansunk = game.humansunk + 1;
                            println!("humansunk: {}", game.humansunk);
                            game.tilemaps[1].set_tile_at(Vec2i(xcompguess, ycompguess), 4); //hit human's ship
                            Mode::Play(Turn::Human)

                        }
                        //misses human's ship
                        else if game.tilemaps[1].tile_id_num_at(Vec2i(xcompguess, ycompguess))!=4{
                            data.sound.play_sound("splash".to_string());
                            game.tilemaps[1].set_tile_at(Vec2i(xcompguess, ycompguess), 4); //misses human's ship
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
                }else if input.key_pressed(VirtualKeyCode::R) {
                    Mode::Reset
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
                }else if input.key_pressed(VirtualKeyCode::R) {
                    Mode::Reset
                }else if input.key_pressed(VirtualKeyCode::P) {
                    Mode::Play(Turn::Human) //need to track and save turn and what the board looks like
                } else {
                    self
                }
            }
            Mode::Reset => {
                let oppmap = Tilemap::new(
                    Vec2i(0, 0), //location
                    (12, 8),
                    &game.tilemaps[0].tileset,
                    vec![
                        3, 0, 0, 0, 0, 0, 3, 0, 0, 0, 3, 3, //3s are hidden opponents
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
                        3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, //
                    ],
                );
                //your ships
                let mymap = Tilemap::new(
                    Vec2i(0, MAPDIM * 2), //location
                    (12, 8),
                    &game.tilemaps[0].tileset,
                    vec![
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 5, 1, 1, //single ship
                        1, 1, 6, 7, 1, 1, 1, 1, 1, 1, 1, 1, //double ship
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
                        10, 11, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
                        14, 15, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
                    ],
                );
                
                // reset to initial game state...
                game.tilemaps = vec![oppmap, mymap];
                game.compsunk = 0;
                game.humansunk = 0;
                save_game(&game);
                Mode::Play(Turn::Human)
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
    fn display(&self, game: &GameState, data: &mut GameData, screen: &mut Screen) {
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
                // Creates a dialog with a single "Quit" button
                

            }
            Mode::Reset => {
                screen.clear(Rgba(0, 0, 0, 255));
            }
            Mode::EndGame => { // Draw game result?
                screen.clear(Rgba(255, 255, 80, 255));
            }
        }
    }
}

fn save_game(game:&GameState) {
    save_file("save.bin", 0, game).unwrap();
}

fn load_game() -> GameState {
    load_file("save.bin", 0).unwrap()
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
    let mut game_sound = Sound::new();
    let _ = game_sound.init_manager();
    game_sound.add_sound("hit".to_string(), "./res/hit.mp3".to_string());
    game_sound.add_sound("splash".to_string(), "./res/splash.mp3".to_string());
    let mut data = GameData {sound: game_sound};
    let title_image = Rc::new(Texture::with_file(Path::new("./res/logo.png")));

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
        texture: Rc::new(Texture::with_file(Path::new("./res/tileset.png"))), //bring in image as texture
    });

    // 6 tilemaps, each 4x4 tiles
    //tilemaps join together into a 3x2 map, i.e. 12x8 tile grid
    //opponent's ships

    //  let oppmap = Tilemap::new(
    //     Vec2i(0, 0), //location
    //     (12, 8),
    //     &boattileset,
    //     vec![
    //         3, 0, 0, 0, 0, 0, 3, 0, 0, 0, 3, 3, //3s are hidden opponents
    //         0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    //         0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    //         0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, //
    //         0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    //         0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, //
    //         0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    //         3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, //
    //     ],
    // );
    // //your ships
    // let mymap = Tilemap::new(
    //     Vec2i(0, MAPDIM * 2), //location
    //     (12, 8),
    //     &boattileset,
    //     vec![
    //         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
    //         1, 1, 1, 1, 1, 1, 1, 1, 1, 5, 1, 1, //single ship
    //         1, 1, 6, 7, 1, 1, 1, 1, 1, 1, 1, 1, //double ship
    //         1, 1, 1, 1, 1, 1, 1, 4, 1, 1, 1, 1, //x mark
    //         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
    //         10, 11, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
    //         14, 15, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
    //         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
    //     ],
    // ); 

    
    
    //  // initial game state...
    // let mut state = GameState {
    //     tilemaps: vec![oppmap, mymap], //vector of tilemaps
    //     title_image: title_image,
    //     compsunk: 0,
    //     humansunk: 0,
    // }; 

    let mut mode = Mode::Title;
    //load saved GameState
    let mut state = load_game();
    state.title_image = title_image;

    // How many frames have we simulated?
    let mut frame_count: usize = 0;
    // How many unsimulated frames have we saved up?
    let mut available_time = 0.0;
    // Track beginning of play
    let start = Instant::now();
    // Track end of the last frame
    let mut since = Instant::now();
    let camera_position = Vec2i(0,0);
    event_loop.run(move |event, _, control_flow| {


        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH, camera_position);
            screen.clear(Rgba(0, 0, 0, 0));

            // change to draw game using state and mode, i.e. mode.draw_game(state)
            //draw_game(&state, &mut screen);
            mode.display(&state, &mut data, &mut screen);

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
            mode = mode.update(&mut state, &mut data, &input);
            // Increment the frame counter
            frame_count += 1;
        }
        // Request redraw
        window.request_redraw();
        // When did the last frame end?
        since = Instant::now();


        //save_game(&state);
        //let reloaded_state = load_game();
        //assert_eq!(reloaded_player.name,"Steve".to_string());

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
