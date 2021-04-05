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


use unit2::screen::Screen;
use unit2::collision::*;
use unit2::texture::Texture;
use unit2::types::*;
use unit2::tiles::*;
use unit2::sound::*;
use unit2::texture::stack_horizontal;


//GameState - saved and loaded from file
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
    font: fontdue::Font,
}
// seconds per frame
const DT: f64 = 1.0 / 60.0;
const MAPDIM: i32 = 64;

const WIDTH: usize = 16*12; //192
const HEIGHT: usize = 16*16; //256
const DEPTH: usize = 4;

///////////Mac: 
const XMAX: usize = 384;
const YMAX: usize = 384;
//////////Windows: 
//const XMAX: usize = 192;
//const YMAX: usize = 256;

//8 squares H by 12 squares W

#[derive(Debug, Copy, Clone)]
enum Mode {
    Title,
    Play(Turn),
    Options,
    ScoreBoard, 
    Reset,
    EndGame,
    WonGame,
    LostGame,
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
                } else if input.key_pressed(VirtualKeyCode::Q) {
                    //Mode::EndGame
                    panic!();
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

                        //check if computer won
                        if game.humansunk == 9 { 
                            println!("You lost!");
                            Mode::LostGame
                        }
                        //check if human won
                        else if game.compsunk == 9 {
                            println!("You won!");
                            Mode::WonGame
                        }
                        else if input.key_pressed(VirtualKeyCode::Q) {
                            Mode::EndGame
                        }else if input.key_pressed(VirtualKeyCode::O) {
                            Mode::Options
                        }else if input.key_pressed(VirtualKeyCode::S) {
                            Mode::ScoreBoard
                        }else if input.key_pressed(VirtualKeyCode::R) {
                            Mode::Reset
                        }else if input.mouse_pressed(0) {
                            
                            //println!("human's turn");

                            let xcoor = input.mouse().unwrap().0 as i32;
                            let ycoor = input.mouse().unwrap().1 as i32;

                            //change tile at coordinates
                            //was opponent's ship hidden there?
                            if game.tilemaps[0].tile_at(Vec2i(xcoor, ycoor)).opphit {
                                data.sound.play_sound("hit".to_string());
                                game.compsunk = game.compsunk + 1;
                                //println!("compsunk: {}", game.compsunk);
                                game.tilemaps[0].set_tile_at(Vec2i(xcoor, ycoor), 8); //hit opponent
                            } else { //missed
                                data.sound.play_sound("splash".to_string());
                                game.tilemaps[0].set_tile_at(Vec2i(xcoor, ycoor), 12); //missed opponent
                            }

                            save_game(&game);
                            let reloaded_game = load_game();
                            *game = reloaded_game; 
                            //assert_eq!(reloaded_game.name,"Steve".to_string());

                            
                            Mode::Play(Turn::Computer)
                            
                        }
                        else{
                            Mode::Play(Turn::Human)
                        }
                    }
                    Turn::Computer => {

                        ///////////// width of screen: 1-384
                        ///////////// height of screen: 128-384

                        //Mac:
                        //let xcompguess = thread_rng().gen_range(1, WIDTH+191) as i32; //1-383
                        //let ycompguess = thread_rng().gen_range(HEIGHT/2+1, HEIGHT+127) as i32; //128-383

                        //Windows:
                        //let xcompguess = thread_rng().gen_range(1, WIDTH) as i32; //1-192
                        //let ycompguess = thread_rng().gen_range(SHEIGHT, HEIGHT) as i32; //128-256
  
                        //both
                        let xcompguess = thread_rng().gen_range(1, XMAX-1) as i32; //Mac: XMAX=384, Windows: XMAX=192
                        let ycompguess = thread_rng().gen_range(HEIGHT/2, YMAX-1) as i32; //Mac: YMAX=384, Windows: YMAX=256

                        //hits human's ship
                        if game.tilemaps[1].tile_at(Vec2i(xcompguess, ycompguess)).myship {
                            data.sound.play_sound("hit".to_string());
                            game.humansunk = game.humansunk + 1;
                            //println!("humansunk: {}", game.humansunk);
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
                    Mode::Play(Turn::Human)
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
                //computer's ships
                let oppmap = Tilemap::new(
                    Vec2i(0, 0), //location
                    (12, 8),
                    &game.tilemaps[0].tileset,
                    vec![
                        0, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, // hidden double ship
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
                        0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 0, 0, // hidden pirate ship
                        0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 0, 0, // hidden pirate ship
                        0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, // hidden tall ship
                        0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, // hidden tall ship
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, // hidden single ship
                    ],
                );
                //your ships
                let mymap = Tilemap::new(
                    Vec2i(0, MAPDIM * 2), //location 0,128
                    (12, 8),
                    &game.tilemaps[0].tileset,
                    vec![
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 5, 1, 1, //single ship
                        1, 1, 6, 7, 1, 1, 1, 1, 1, 1, 1, 1, //double ship
                        1, 1, 1, 1, 1, 1, 1, 1, 9, 1, 1, 1, // tall ship
                        1, 1, 1, 1, 1, 1, 1, 1, 13, 1, 1, 1, // tall ship
                        10, 11, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // pirate ship
                        14, 15, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // pirate ship
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
                    ],
                );
                
                // reset to initial game state
                game.tilemaps = vec![oppmap, mymap];
                game.compsunk = 0;
                game.humansunk = 0;
                save_game(&game);

                Mode::Play(Turn::Human)
            }
            Mode::WonGame => {
                //resetting
                //computer's ships
                let oppmap = Tilemap::new(
                    Vec2i(0, 0), //location
                    (12, 8),
                    &game.tilemaps[0].tileset,
                    vec![
                        0, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, // hidden double ship
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
                        0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 0, 0, // hidden pirate ship
                        0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 0, 0, // hidden pirate ship
                        0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, // hidden tall ship
                        0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, // hidden tall ship
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, // hidden single ship
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
                        1, 1, 1, 1, 1, 1, 1, 1, 9, 1, 1, 1, // tall ship
                        1, 1, 1, 1, 1, 1, 1, 1, 13, 1, 1, 1, // tall ship
                        10, 11, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // pirate ship
                        14, 15, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // pirate ship
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
                    ],
                );
                
                // reset to initial game state
                game.tilemaps = vec![oppmap, mymap];
                game.compsunk = 0;
                game.humansunk = 0;
                save_game(&game);
                
                if input.key_pressed(VirtualKeyCode::Q) {
                    panic!();
                } else if input.key_pressed(VirtualKeyCode::P) {
                    Mode::Play(Turn::Human)
                } else if input.key_pressed(VirtualKeyCode::T) {
                    Mode::Title
                }
                else {
                    self
                }
            }
            Mode::LostGame => {
                //reseting game
                //computer's ships
                let oppmap = Tilemap::new(
                    Vec2i(0, 0), //location
                    (12, 8),
                    &game.tilemaps[0].tileset,
                    vec![
                        0, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, // hidden double ship
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
                        0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 0, 0, // hidden pirate ship
                        0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 0, 0, // hidden pirate ship
                        0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, // hidden tall ship
                        0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, // hidden tall ship
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, // hidden single ship
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
                        1, 1, 1, 1, 1, 1, 1, 1, 9, 1, 1, 1, // tall ship
                        1, 1, 1, 1, 1, 1, 1, 1, 13, 1, 1, 1, // tall ship
                        10, 11, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // pirate ship
                        14, 15, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // pirate ship
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, //
                    ],
                );
                
                // reset to initial game state
                game.tilemaps = vec![oppmap, mymap];
                game.compsunk = 0;
                game.humansunk = 0;
                save_game(&game);

                if input.key_pressed(VirtualKeyCode::Q) {
                    panic!();
                } else if input.key_pressed(VirtualKeyCode::P) {
                    Mode::Play(Turn::Human)
                } else if input.key_pressed(VirtualKeyCode::T) {
                    Mode::Title
                }
                else {
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
    fn display(&self, game: &GameState, data: &mut GameData, screen: &mut Screen) {
        match self {
            Mode::Title => {
                //draw a (static?) title
                screen.clear(Rgba(80, 80, 80, 255));
                let display_rect = Rect {
                    x: 0,
                    y: 0,
                    w: 186,
                    h: 158,
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
                screen.clear(Rgba(0, 0, 0, 255));

                let options_tex = create_text_tex(&data.font, "OPTIONS".to_string());
                let from_rect_options = Rect{x: 0, y: 0, w: options_tex.width as u16, h: options_tex.height as u16};
                let to_pos_options = Vec2i((WIDTH - options_tex.width) as i32 / 2, (HEIGHT - options_tex.height) as i32 / 6);
                screen.bitblt(&options_tex, from_rect_options, to_pos_options);

                let score_tex = create_text_tex(&data.font, "S>>>Score".to_string());
                let from_rect_score = Rect{x: 0, y: 0, w: score_tex.width as u16, h: score_tex.height as u16};
                let to_pos_score = Vec2i((WIDTH - score_tex.width) as i32 / 2, (HEIGHT - score_tex.height) as i32 / 3);
                screen.bitblt(&score_tex, from_rect_score, to_pos_score);

                let quit_tex = create_text_tex(&data.font, "Q>>>Quit".to_string());
                let from_rect_quit = Rect{x: 0, y: 0, w: quit_tex.width as u16, h: quit_tex.height as u16};
                let to_pos_quit = Vec2i((WIDTH - quit_tex.width) as i32 / 2, (HEIGHT - quit_tex.height) as i32 / 2);
                screen.bitblt(&quit_tex, from_rect_quit, to_pos_quit);

                let play_tex = create_text_tex(&data.font, "P>>>Play".to_string());
                let from_rect_play = Rect{x: 0, y: 0, w: play_tex.width as u16, h: play_tex.height as u16};
                let to_pos_play = Vec2i((WIDTH - play_tex.width) as i32 / 2, (HEIGHT - play_tex.height) as i32 / 3 * 2);
                screen.bitblt(&play_tex, from_rect_play, to_pos_play);
            }
            Mode::ScoreBoard => {
                screen.clear(Rgba(0, 0, 0, 255));

                let highscore_tex = create_text_tex(&data.font, "TALLY".to_string());
                // todo: change stack_horizontal to fill space rather than take shortest character
                let from_rect = Rect{x: 0, y: 0, w: highscore_tex.width as u16, h: highscore_tex.height as u16};
                let to_pos = Vec2i((WIDTH - highscore_tex.width) as i32 / 2, (HEIGHT - highscore_tex.height) as i32 / 4);
                screen.bitblt(&highscore_tex, from_rect, to_pos);

                let comp_score_tex = create_text_tex(&data.font, "Computer:    ".to_string() + &game.humansunk.to_string());
                let comp_from_rect = Rect{x: 0, y: 0, w: comp_score_tex.width as u16, h: comp_score_tex.height as u16};
                let comp_to_pos = Vec2i((WIDTH - comp_score_tex.width) as i32 / 2, (HEIGHT - comp_score_tex.height) as i32 / 2);
                screen.bitblt(&comp_score_tex, comp_from_rect, comp_to_pos);

                let hum_score_tex = create_text_tex(&data.font, "You:    ".to_string() + &game.compsunk.to_string());
                let hum_from_rect = Rect{x: 0, y: 0, w: hum_score_tex.width as u16, h: hum_score_tex.height as u16};
                let hum_to_pos = Vec2i((WIDTH - hum_score_tex.width) as i32 / 2, (HEIGHT - hum_score_tex.height) as i32 / 4 * 3);
                screen.bitblt(&hum_score_tex, hum_from_rect, hum_to_pos);
            }
            Mode::Reset => {
                screen.clear(Rgba(0, 0, 0, 255));
            }
            Mode::EndGame => { // Draw game result?
                screen.clear(Rgba(255, 255, 80, 255));
            }
            Mode::WonGame => { 
                screen.clear(Rgba(0, 0, 0, 255));
                let tex = create_text_tex(&data.font, "WINNER!".to_string());
                let from_rect = Rect{x: 0, y: 0, w: tex.width as u16, h: tex.height as u16};
                let to_pos = Vec2i((WIDTH - tex.width) as i32 / 2, (HEIGHT - tex.height) as i32 / 2);
                screen.bitblt(&tex, from_rect, to_pos);
            }
            Mode::LostGame => { 
                screen.clear(Rgba(0, 0, 0, 255));
                let tex = create_text_tex(&data.font, "GAME OVER!".to_string());
                let from_rect = Rect{x: 0, y: 0, w: tex.width as u16, h: tex.height as u16};
                let to_pos = Vec2i((WIDTH - tex.width) as i32 / 2, (HEIGHT - tex.height) as i32 / 2);
                screen.bitblt(&tex, from_rect, to_pos);
            }
        }
    }
}

fn save_game(game:&GameState) {
    save_file("save_battleship.bin", 0, game).unwrap();
}

fn load_game() -> GameState {
    load_file("save_battleship.bin", 0).unwrap()
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

    //sound
    let mut game_sound = Sound::new();
    let _ = game_sound.init_manager();
    game_sound.add_sound("hit".to_string(), "./res/hit.mp3".to_string());
    game_sound.add_sound("splash".to_string(), "./res/splash.mp3".to_string());

    //font
    let mut font:&[u8];
    //Mac
    font = include_bytes!("../../res/Exo2-Regular.ttf");
    //Windows
    //font = include_bytes!("..\\..\\res\\Exo2-Regular.ttf") as &[u8];

    
    let settings = fontdue::FontSettings {
        scale: 12.0,
        ..fontdue::FontSettings::default()
    };
    let font = fontdue::Font::from_bytes(font, settings).unwrap();

    let mut data = GameData {sound: game_sound, font: font};

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


    });
}


fn create_text_tex(font: &fontdue::Font, text: String) -> Rc<Texture> {
    let font_size = 30.0;
    let mut char_textures: Vec<Texture> = vec![];
    let mut i = 0;
    while i < text.len() {
        let character = text.chars().nth(i).unwrap();
        let (metrics, bitmap) = font.rasterize(character, font_size);
        let mut char_tex = Texture::from_vec(bitmap, metrics.width, metrics.height, 1);
        char_tex.convert_to_rgba();
        char_textures.push(char_tex);
        i += 1;
    }
    return Rc::new(stack_horizontal(char_textures));
}

