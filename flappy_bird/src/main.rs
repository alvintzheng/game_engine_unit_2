/**** NOTES ****
    - 
*/
/**** TO DO ****
    - game over page, set up gamestate/menu structure
    - read/write to external file for high score (time survived=score?)
*/

use pixels::{Pixels, SurfaceTexture};
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use std::time::{Duration};

use rand::{thread_rng, Rng};

mod screen;
use screen::Screen;
mod texture;
use texture::Texture;
mod animation;
use animation::Animation;
mod sprite;
use sprite::*;
mod types;
use types::*;
mod collision;
use collision::*;
mod entity;
use entity::*;
mod sound;
use sound::Sound;
mod tiles;
use tiles::*;

// seconds per frame
const DT: f64 = 1.0 / 60.0;

const DEPTH: usize = 4;
const WIDTH: usize = 800;
const HEIGHT: usize = 500;
//const PITCH: usize = WIDTH * DEPTH;

// We'll make our Color type an RGBA8888 pixel.
//type Color = [u8; DEPTH];

const CLEAR_COL: Rgba = Rgba(0, 0, 0, 0);
//const WALL_COL: Color = [200, 200, 200, 255];
//const PLAYER_COL: Color = [255, 255, 0, 255];

const OBSTACLE_SPACING: u16 = 250;
const OBSTACLE_WIDTH: u16 = 30;
const GAP_HEIGHT: usize = 175;
const OBSTACLE_MIN_HEIGHT: u16 = 50;
const OBSTACLE_MAX_HEIGHT: u16 = (HEIGHT - GAP_HEIGHT) as u16 - OBSTACLE_MIN_HEIGHT;
const OBSTACLE_SPEED: u16 = 4;
const MIN_OBSTACLES: usize = (WIDTH / (OBSTACLE_SPACING + OBSTACLE_WIDTH) as usize) * 2 + 1;
const BACKGROUND_SPEED: u16 = 1;
const MAP_WIDTH: usize = WIDTH / tiles::TILE_SZ + 1;
const MAP_HEIGHT: usize = HEIGHT / tiles::TILE_SZ + 1;
const MAP_SIZE: usize = MAP_WIDTH * MAP_HEIGHT;

#[derive(Debug, Copy, Clone)]
enum Mode {
    Title,
    Play(bool),
    Options,
    ScoreBoard,
    EndGame,
}

struct GameState {
    player: Bird,
    obstacles: Vec<Entity>,
    accel_down: i32,
    finished: bool,
    score: usize,
    score_tex: Rc<Texture>,
    tilemaps: Vec<Tilemap>,
    walls: Vec<Wall>,
}

struct GameData {
    obstacle_tex_up: Rc<Texture>,
    obstacle_tex_down: Rc<Texture>,
    title_tex: Rc<Texture>,
    player_tex: Rc<Texture>,
    wing_tex: Rc<Texture>,
    font: fontdue::Font,
    sound: Sound,
    sky_tex: Rc<Texture>,
    highscore: usize,
    // should not use hashmap? because result of get() will be option?
}

impl Mode {
    // update consumes self and yields a new state (which might also just be self)
    fn update(self, state: &mut GameState, data: &mut GameData, input: &WinitInputHelper) -> Self {
        match self {
            Mode::Title => {

                if input.key_pressed(VirtualKeyCode::P) {
                    *state = new_game(data);
                    Mode::Play(false)
                }
                else if input.key_pressed(VirtualKeyCode::O) {
                    Mode::Options
                } 
                else if input.key_pressed(VirtualKeyCode::S) {
                    Mode::ScoreBoard
                }
                else if input.key_pressed(VirtualKeyCode::Q) {
                    panic!();
                }
                else {
                    self
                }
            }
            Mode::Play(paused) => {
                if !paused {
                    update_game(state, input, data);
                }
                if state.finished {
                    if data.highscore < state.score {
                        data.highscore = state.score;
                    }
                    Mode::EndGame // should be endgame
                }
                else if input.key_pressed(VirtualKeyCode::Space) {
                    Mode::Play(!paused)
                }
                else if input.key_pressed(VirtualKeyCode::T) {
                    Mode::Title
                }
                else {
                    self
                }
            }
            Mode::Options => {
                if input.key_pressed(VirtualKeyCode::T) {
                    Mode::Title
                }
                else if input.key_pressed(VirtualKeyCode::P) {
                    *state = new_game(data);
                    Mode::Play(false)
                }
                else if input.key_pressed(VirtualKeyCode::S) {
                    Mode::ScoreBoard
                }
                else if input.key_pressed(VirtualKeyCode::Q) {
                    panic!();
                }
                else {
                    self
                }
            }
            Mode::ScoreBoard => {
                if input.key_pressed(VirtualKeyCode::T) {
                    Mode::Title
                }
                else if input.key_pressed(VirtualKeyCode::P) {
                    *state = new_game(data);
                    Mode::Play(false)
                }
                else if input.key_pressed(VirtualKeyCode::O) {
                    Mode::Options
                } 
                else if input.key_pressed(VirtualKeyCode::Q) {
                    panic!();
                }
                else {
                    self
                }
            }
            Mode::EndGame => {
                if input.key_pressed(VirtualKeyCode::T) {
                    Mode::Title
                }
                else if input.key_pressed(VirtualKeyCode::P) {
                    *state = new_game(data);
                    Mode::Play(false)
                }
                else {
                    self
                }
            }
        }
    }
    fn display(&self, state: &mut GameState, data: &mut GameData, screen: &mut Screen) {
        match self {
            Mode::Title => {
                //draw a (static?) title
                screen.clear(Rgba(0, 0, 0, 255));
                let display_rect = Rect {
                    x: 0,
                    y: 0,
                    w: 250,
                    h: 51,
                };
                screen.bitblt(&data.title_tex, display_rect, Vec2i(275, 224));
                let play_tex = create_text_tex(&data.font, "P>>>Play".to_string());
                let from_rect_play = Rect{x: 0, y: 0, w: play_tex.width as u16, h: play_tex.height as u16};
                let to_pos_play = Vec2i((WIDTH - play_tex.width) as i32 / 2, (HEIGHT - play_tex.height) as i32 / 3 * 2);
                screen.bitblt(&play_tex, from_rect_play, to_pos_play);
            }
            Mode::Play(paused) => {
                // Call screen's drawing methods to render the game state
                screen.clear(Rgba(80, 80, 80, 255));

                //draw each tilemap in vector to screen
                draw_game(state, data, screen);
            }
            Mode::Options => {
                screen.clear(Rgba(0, 0, 0, 255));
                let options_tex = create_text_tex(&data.font, "OPTIONS".to_string());
                let from_rect_options = Rect{x: 0, y: 0, w: options_tex.width as u16, h: options_tex.height as u16};
                let to_pos_options = Vec2i((WIDTH - options_tex.width) as i32 / 2, (HEIGHT - options_tex.height) as i32 / 6);
                screen.bitblt(&options_tex, from_rect_options, to_pos_options);

                let score_tex = create_text_tex(&data.font, "S>>>Highscore".to_string());
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
                let highscore_tex = create_text_tex(&data.font, "Highscore:    ".to_string() + &data.highscore.to_string());
                // todo: change stack_horizontal to fill space rather than take shortest character
                let from_rect = Rect{x: 0, y: 0, w: highscore_tex.width as u16, h: highscore_tex.height as u16};
                let to_pos = Vec2i((WIDTH - highscore_tex.width) as i32 / 2, (HEIGHT - highscore_tex.height) as i32 / 2);
                screen.bitblt(&highscore_tex, from_rect, to_pos);
            }
            Mode::EndGame => { // Draw game result?
                screen.clear(Rgba(255, 255, 80, 255));
                draw_game(state, data, screen);
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
            .with_title("Flappy Bird")
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

    //load assets
    let player_tex = Rc::new(Texture::with_file(Path::new("./res/bird.png")));
    let obstacle_tex_up = Rc::new(Texture::with_file(Path::new("./res/pipe_up.png")));
    let obstacle_tex_down = Rc::new(Texture::with_file(Path::new("./res/pipe_down.png")));
    let title_tex = Rc::new(Texture::with_file(Path::new("./res/TitleImage.png")));
    let wing_tex = Rc::new(Texture::with_file(Path::new("./res/wings.png")));
    let sky_tex = Rc::new(Texture::with_file(Path::new("./res/flappy_sky_dilute.png")));

    let mut game_sound = Sound::new();
    let _ = game_sound.init_manager();
    ///////////////mp3 files aren't in res folder
    game_sound.add_sound("jump".to_string(), "./res/jump.mp3".to_string());
    game_sound.add_sound("pass".to_string(), "./res/pass.mp3".to_string());
    game_sound.add_sound("die".to_string(), "./res/die.mp3".to_string());

    let mut mode = Mode::Title;
    let mut font:&[u8];// = include_bytes!("..\\res\\Exo2-Regular.ttf") as &[u8];

    if cfg!(target_os = "windows") {
        font = include_bytes!("..\\res\\Exo2-Regular.ttf") as &[u8];
      } else {
        font = include_bytes!("../res/Exo2-Regular.ttf") as &[u8];
      }
    
    let settings = fontdue::FontSettings {
        scale: 12.0,
        ..fontdue::FontSettings::default()
    };
    let font = fontdue::Font::from_bytes(font, settings).unwrap();

    let mut data = GameData {
        obstacle_tex_up: obstacle_tex_up,
        obstacle_tex_down: obstacle_tex_down,
        title_tex: title_tex,
        player_tex: player_tex,
        font: font,
        wing_tex: wing_tex,
        sound: game_sound,
        sky_tex: sky_tex,
        highscore: 0,
    };

    let mut state = new_game(&data);
    let camera_position = Vec2i(0,0);

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
            let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH, camera_position);
            screen.clear(CLEAR_COL);
            mode.display(&mut state, &mut data, &mut screen);

            // Flip buffers
            if pixels.render().is_err() {
                *control_flow = ControlFlow::Exit;
                println!("stop");
                return;
            }

            // Rendering has used up some time.
            // The renderer "produces" time...
            let additional_time = since.elapsed().as_secs_f64();
            available_time += additional_time;
            // When did the last frame end?
            since = Instant::now();
        }
        // Handle input events
        if input.update(event) {
            //println!("input");
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
            mode = mode.update(&mut state, &mut data, &input);

            // Increment the frame counter
            frame_count += 1;
        }
        // Request redraw
        window.request_redraw();
        
    });
}

fn draw_game(state: &mut GameState, data: &mut GameData, screen: &mut Screen) {
    // Call screen's drawing methods to render the game state
    screen.clear(Rgba(80, 80, 80, 255));

    for tilemap in state.tilemaps.iter() {
        tilemap.draw(screen);
    }

    for obs in state.obstacles.iter_mut() {
        screen.draw_entity(obs);
    }
    //draw score
    
    let score_rect = Rect{x: (WIDTH / 2 - 70) as i32, y: 0, w: 160, h: 30};
    screen.rect(score_rect, Rgba(0, 0, 0,255));
    screen.rect_outline(score_rect, Rgba(255, 255, 100, 255));
    let score_text_rect = Rect{x: 0, y: 0, w: state.score_tex.width as u16, h: state.score_tex.height as u16};
    let score_text_pos = Vec2i((WIDTH / 2) as i32, 10);
    
    screen.bitblt(&state.score_tex, score_text_rect, score_text_pos);
    
    
    state.player.body.sprite.animations[0].current_frame = scale_range(state.player.body.hitbox.vy, -10.0, 7.0, 0.0, 4.0) as u16;
    screen.draw_bird(&mut state.player);

    
}


fn update_game(state: &mut GameState, input: &WinitInputHelper, data: &mut GameData) {
    let player = &mut state.player.body.hitbox;
    // Determine player velocity
    //let movespeed: i32 = 2;
    if input.key_held(VirtualKeyCode::Left) {
        //player.vx = -1 * movespeed;
    } else if input.key_held(VirtualKeyCode::Right) {
        //player.vx = 1 * movespeed;
    } else if input.key_pressed(VirtualKeyCode::Down) {
        //player.vx = 0;
    } else {
        player.vx = 0;
    }
    let mut accel_down = state.accel_down;
    if input.key_pressed(VirtualKeyCode::Up) {
        //accel_down = -5;
        accel_down = -2;
        data.sound.play_sound("jump".to_string());
        //player.vy -= 40; //method 2
        state.player.wing.animations[0].current_frame = 0;
    } else {
        accel_down += 1;
        //player.vy += 3; //method 2
        if accel_down > 1 {
            accel_down = 1;
        }
    }
    state.accel_down = accel_down;
    player.vy += accel_down;
    //clamp velocity since this restitution assumes objects aren't speeding too much
    let min_velocity = -10;
    let max_velocity = 10;
    if player.vy > max_velocity {
        player.vy = max_velocity;
    }
    if player.vy < min_velocity {
        player.vy = min_velocity;
    }

    if state.obstacles.len() < MIN_OBSTACLES {
        if state.obstacles.len() == 0 || WIDTH as i32 - state.obstacles[state.obstacles.len() - 1].hitbox.rect.x - (OBSTACLE_WIDTH as i32) >= OBSTACLE_SPACING as i32 {
            let new_height = thread_rng().gen_range(OBSTACLE_MIN_HEIGHT, OBSTACLE_MAX_HEIGHT);
            // pipe_up; bottom pipe
            {let new_hitbox = Mobile {
                rect: Rect {
                    x: WIDTH as i32 - OBSTACLE_WIDTH as i32,
                    y: HEIGHT as i32 - new_height as i32,
                    w: OBSTACLE_WIDTH,
                    h: new_height as u16,
                },
                vx: OBSTACLE_SPEED as i32 * -1,
                vy: 0,
            };
            let mut new_sprite = Sprite::new(
                &data.obstacle_tex_up,
                Rect {
                    x: 0,
                    y: 0,
                    w: 30,
                    h: 400,
                },
                Vec2i(0, 0),
            );
            
            let new_animation = Animation::new(OBSTACLE_WIDTH, new_height as u16, 0, 0, 1);
            new_sprite.animations.push(new_animation);
            let new_obstacle = Entity::new(new_hitbox, new_sprite, false);
            state.obstacles.push(new_obstacle);}
            // pipe_down, top pipe
            {
            let new_height_2 = HEIGHT - new_height as usize - GAP_HEIGHT;
            let new_hitbox = Mobile {
                rect: Rect {
                    x: WIDTH as i32 - OBSTACLE_WIDTH as i32,
                    y: 0,
                    w: OBSTACLE_WIDTH,
                    h: new_height_2 as u16,
                },
                vx: OBSTACLE_SPEED as i32 * -1,
                vy: 0,
            };
            let mut new_sprite = Sprite::new(
                &data.obstacle_tex_down,
                Rect {
                    x: 0,
                    y: 0,
                    w: 30,
                    h: 400,
                },
                Vec2i(0, 0),
            );
            let new_animation = Animation::new(OBSTACLE_WIDTH, new_height_2 as u16, 0, 400 - new_height_2 as i32, 1);
            new_sprite.animations.push(new_animation);
            let new_obstacle = Entity::new(new_hitbox, new_sprite, false);
            state.obstacles.push(new_obstacle);}
        }
    }

    // check front pipe to see if it needs to be deleted
    if state.obstacles[0].hitbox.rect.x < 0 - OBSTACLE_WIDTH as i32 {
            let _ = state.obstacles.remove(0);
            let _2 = state.obstacles.remove(0);
            state.score += 1;
            data.sound.play_sound("pass".to_string());
            state.score_tex = create_score_tex(&data.font, state.score);
    }
    player.update();
    
    // collisions

    for wall in state.walls.iter() {
        if collision::rect_touching(wall.rect, player.rect) {
            state.finished = true;
            data.sound.play_sound("die".to_string());
            break;
        }
    }

    for obs in state.obstacles.iter() {
        if collision::rect_touching(obs.hitbox.rect, player.rect) {
            state.finished = true;
            data.sound.play_sound("die".to_string());
            break;
        }
    }

    for obs in state.obstacles.iter_mut() {
        obs.hitbox.update();
    }

    for tm in state.tilemaps.iter_mut() {
        tm.position.0 -= BACKGROUND_SPEED as i32;
    }
    if (state.tilemaps[0].position.0 + (state.tilemaps[0].dims.0 * tiles::TILE_SZ) as i32) < 0 {
        state.tilemaps.remove(0);
    }
    if state.tilemaps.len() < 2 {
        let mut new_tilemap = new_sky(data);
        new_tilemap.position = Vec2i((MAP_WIDTH * tiles::TILE_SZ) as i32, 0);
        state.tilemaps.push(new_tilemap);
    }
}

fn new_game(data: &GameData) -> GameState {

    let mut player_sprite = Sprite::new(
        &data.player_tex,
        Rect {
            x: 0,
            y: 128,
            w: 64,
            h: 64,
        },
        Vec2i(0, 0),
    );
    let mut player_animation = Animation::new(32, 32, 0, 0, 5);
    player_animation.set_duration(Duration::new(3600, 0));
    player_sprite.animations.push(player_animation);
    let player_hitbox = Mobile{rect: Rect{x:32, y:45, w: 25, h: 25}, vx:0, vy: 0};
    let body = Entity::new(player_hitbox, player_sprite, true);
    let mut wing = Sprite::new(
        &data.wing_tex,
        Rect {
            x: 0,
            y: 128,
            w: 64,
            h: 64,
        },
        Vec2i(0, 0),
    );
    let mut wing_animation = Animation::new(22, 48, 0, 0, 9);
    wing_animation.set_duration(Duration::from_millis(30));
    wing_animation.do_loop = false;
    wing.animations.push(wing_animation);
    let player = Bird{body: body, wing: wing};
    
    let obstacles: Vec<Entity> = vec![];
    let mut tilemaps: Vec<Tilemap> = vec![];
    let sky1 = new_sky(data);
    tilemaps.push(sky1);
    let mut walls: Vec<Wall> = vec![];
    walls.push(Wall{rect:Rect{x: 0, y: -1, w: WIDTH as u16, h: 1}});
    walls.push(Wall{rect:Rect{x: 0, y: HEIGHT as i32, w: WIDTH as u16, h: 1}});

    let state = GameState {
        // initial game state
        player: player,
        obstacles: obstacles,
        accel_down: 0,
        finished: false,
        score: 0,
        score_tex: create_score_tex(&data.font, 0),
        tilemaps: tilemaps,
        walls: walls,
    };
    return state;
}

fn create_score_tex(font: &fontdue::Font, score: usize) -> Rc<Texture> {
    let font_size = 20.0;
    let score_string = score.to_string();
    let mut digit_textures: Vec<Texture> = vec![];
    let mut i = 0;
    while i < score_string.len() {
        let digit = score_string.chars().nth(i).unwrap();
        let (metrics, bitmap) = font.rasterize(digit, font_size);
        let mut score_tex = Texture::from_vec(bitmap, metrics.width, metrics.height, 1);
        score_tex.convert_to_rgba();
        digit_textures.push(score_tex);
        i += 1;
    }
    
    return Rc::new(texture::stack_horizontal(digit_textures));
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
    return Rc::new(texture::stack_horizontal(char_textures));
}

fn scale_range(value: i32, value_min: f32, value_max: f32, scale_min:f32, scale_max:f32) -> i32{
    let mut scaled: f32 = value as f32;
    scaled = (scaled - value_min) / (value_max - value_min) * (scale_max - scale_min) + scale_min;
    if scaled < scale_min {
        scaled = scale_min;
    }
    if scaled > scale_max {
        scaled = scale_max;
    }
    return scaled as i32;
}

fn new_sky(data: &GameData) -> Tilemap {
    let position = Vec2i(0, 0);
    let tile_types = 16;
    let dims = (MAP_WIDTH, MAP_HEIGHT);
    let mut tiles:Vec<Tile> = vec![];
    let mut i = 0;
    while i < tile_types {
        let new_tile = Tile{oppgrid: true, opphit: false, myship: false};
        tiles.push(new_tile);
        i += 1;
    }
    let tileset = Tileset::new(tiles, &data.sky_tex);
    let mut map:Vec<usize> = vec![];
    i = 0;
    
    while i < MAP_SIZE {
        let new_tile_id = thread_rng().gen_range(0, tile_types);
        map.push(new_tile_id);
        i += 1;
    }
    
    return Tilemap::new(position, dims, &Rc::new(tileset), map);
}