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

struct GameState {
    // What data do we need for this game?  Wall positions?
    // Colliders?  Sprites and stuff?
    //animations: Vec<Animation>,
    //textures: Vec<Rc<Texture>>,
    player: Entity,
    obstacles: Vec<Entity>,
    //tiles: Vec<Tilemap>,
}

fn main() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Collision2D")
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
    let player_tex = Rc::new(Texture::with_file(Path::new("./res/sprites.png")));
    let obstacle_tex = Rc::new(Texture::with_file(Path::new("./res/Warp_pipe.png")));
    let mut player_sprite = Sprite::new(
        &player_tex,
        Rect {
            x: 0,
            y: 0,
            w: 64,
            h: 64,
        },
        Vec2i(0, 0),
    );
    let mut player_animation = Animation::new(64, 64, 0, 0, 4);
    player_sprite.animations.push(player_animation);
    let mut player_hitbox = Mobile{rect: Rect{x:32, y:45, w: 16, h: 16}, vx:0, vy: 0};
    let mut player = Entity::new(player_hitbox, player_sprite, true);
    
    let obstacles: Vec<Entity> = vec![];
    let obstacle_spacing = 100;
    let obstacle_width: u16 = 30;
    let obstacle_max_height: u16 = 50;
    let obstacle_min_height: u16 = 20;
    let obstacle_speed: u16 = 2;
    let min_obstacles = WIDTH / (obstacle_spacing + obstacle_width) as usize;
    println!("{}", min_obstacles);

    let mut state = GameState {
        // initial game state...
        player: player,
        obstacles: obstacles,
        //tiles: tilemaps,
    };

    let mut camera_position = Vec2i(0,0);

    // How many frames have we simulated?
    let mut frame_count: usize = 0;
    // How many unsimulated frames have we saved up?
    let mut available_time = 0.0;
    // Track beginning of play
    let start = Instant::now();
    //let mut contacts = vec![];
    //let mut mobiles = [player, mover];
    // Track end of the last frame
    let mut since = Instant::now();
    event_loop.run(move |event, _, control_flow| {

        let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH, camera_position);
        screen.clear(CLEAR_COL);
        draw_game(&mut state, &mut screen);
        // Flip buffers
        if pixels.render().is_err() {
            *control_flow = ControlFlow::Exit;
            return;
        }

        // delete old pipes and add new ones
        // add new elems
        if state.obstacles.len() < min_obstacles {
            if state.obstacles.len() == 0 || WIDTH as i32 - state.obstacles[state.obstacles.len() - 1].hitbox.rect.x - (obstacle_width as i32) >= obstacle_spacing as i32 {
            //println!("creating obstacle");
            let new_height = thread_rng().gen_range(obstacle_min_height, obstacle_max_height);
            let mut new_hitbox = Mobile {
                rect: Rect {
                    x: WIDTH as i32 - obstacle_width as i32,
                    y: HEIGHT as i32 - new_height as i32,
                    w: obstacle_width,
                    h: new_height as u16,
                },
                vx: obstacle_speed as i32 * -1,
                vy: 0,
            };
            let mut new_sprite = Sprite::new(
                &obstacle_tex,
                Rect {
                    x: 0,
                    y: 0,
                    w: 64,
                    h: 64,
                },
                Vec2i(0, 0),
            );
            let mut new_animation = Animation::new(obstacle_width, new_height as u16, 0, 0, 1);
            new_sprite.animations.push(new_animation);
            let mut new_obstacle = Entity::new(new_hitbox, new_sprite, false);
            state.obstacles.push(new_obstacle);
            }
        }

        // check front pipe to see if it needs to be deleted

        if state.obstacles[0].hitbox.rect.x < 0 - obstacle_width as i32 {
                let _ = state.obstacles.remove(0);
        }

        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH, camera_position);
            screen.clear(CLEAR_COL);

            draw_game(&mut state, &mut screen);

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

fn draw_game(state: &mut GameState, screen: &mut Screen) {
    // Call screen's drawing methods to render the game state
    screen.clear(Rgba(80, 80, 80, 255));

    for obs in state.obstacles.iter_mut() {
        screen.draw_entity(obs);
    }
    screen.draw_entity(&mut state.player);
}


fn update_game(state: &mut GameState, input: &WinitInputHelper, frame: usize) {
    // let player = &mut state.player.hitbox;
    // // Determine player velocity
    // let movespeed: i32 = 2;
    // if input.key_held(VirtualKeyCode::Left) {
    //     //player.vx = -1 * movespeed;
    // } else if input.key_held(VirtualKeyCode::Right) {
    //     //player.vx = 1 * movespeed;
    // } else if input.key_pressed(VirtualKeyCode::Down) {
    //     //player.vx = 0;
    // } else {
    //     player.vx = 0;
    // }
    // let mut accel_down = 1;
    // if input.key_held(VirtualKeyCode::Up) {
    //     accel_down -= 2;
    // }
    // player.vy += accel_down;
    // //clamp velocity since this restitution assumes objects aren't speeding too much
    // if player.vy > 4 {
    //     player.vy = 4;
    // }
    // if player.vy < -4 {
    //     player.vy = -4;
    // }

    for mut obs in state.obstacles.iter_mut() {
        obs.hitbox.update();
    }
}