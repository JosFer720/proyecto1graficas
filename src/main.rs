mod framebuffer;
mod maze;
mod caster;
mod player;
mod texture;
mod sprites;
mod taylor_sprite;
mod taylor_ai;

use maze::{Maze, load_maze, extract_sprite_positions, clean_maze};
use caster::render_world_with_textures_sprites_and_taylor;
use framebuffer::Framebuffer;
use player::{Player, process_events};
use texture::TextureManager;
use sprites::SpriteManager;
use taylor_sprite::TaylorSprite;
use raylib::prelude::*;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time::Duration;
use std::f32::consts::PI;
use taylor_ai::TaylorAI;

#[derive(Clone, Copy, PartialEq)]
pub enum GameState {
    Menu,
    Playing,
    LevelComplete,
    GameOver,
    Victory,
}

struct GameLevel {
    maze_file: String,
    level_name: String,
    required_cans: usize,
    taylor_speed: f32,
    taylor_spawn_x: f32,
    taylor_spawn_y: f32,
}

pub struct GameData {
    pub current_level: usize,
    pub levels: Vec<GameLevel>,
    pub gasoline_collected: usize,
    pub game_timer: f32,
    pub taylor_position: Vector2,
    pub taylor_target: Vector2,
    pub taylor_last_move_time: f32,
    pub car_reached: bool,
    pub taylor_ai: TaylorAI,
}

impl GameData {
    fn new() -> Self {
        let levels = vec![
            GameLevel {
                maze_file: "levels/level1.txt".to_string(),
                level_name: "Hollywood Studio".to_string(),
                required_cans: 4,
                taylor_speed: 2.6, 
                taylor_spawn_x: 250.0,
                taylor_spawn_y: 950.0,
            },
            GameLevel {
                maze_file: "levels/level2.txt".to_string(),
                level_name: "Recording Studio".to_string(),
                required_cans: 4,
                taylor_speed: 3.25, 
                taylor_spawn_x: 250.0,
                taylor_spawn_y: 950.0,
            },
            GameLevel {
                maze_file: "levels/level3.txt".to_string(),
                level_name: "Concert Venue".to_string(),
                required_cans: 4,
                taylor_speed: 3.9,
                taylor_spawn_x: 250.0,
                taylor_spawn_y: 1150.0,
            },
        ];

        Self {
            current_level: 0,
            levels,
            gasoline_collected: 0,
            game_timer: 0.0,
            taylor_position: Vector2::new(800.0, 800.0),
            taylor_target: Vector2::new(800.0, 800.0),
            taylor_last_move_time: 0.0,
            car_reached: false,
            taylor_ai: TaylorAI::new(),
        }
    }

    pub fn get_current_level(&self) -> &GameLevel {
        &self.levels[self.current_level]
    }

    pub fn next_level(&mut self) {
        self.current_level += 1;
        self.gasoline_collected = 0;
        self.car_reached = false;
        if self.current_level < self.levels.len() {
            let level = &self.levels[self.current_level];
            self.taylor_position = Vector2::new(level.taylor_spawn_x, level.taylor_spawn_y);
            self.taylor_target = self.taylor_position;
        }
    }

    pub fn reset_level(&mut self) {
        self.gasoline_collected = 0;
        self.car_reached = false;
        let level = &self.levels[self.current_level];
        self.taylor_position = Vector2::new(level.taylor_spawn_x, level.taylor_spawn_y);
        self.taylor_target = self.taylor_position;
    }
}

fn update_taylor_ai(
    game_data: &mut GameData,
    player: &Player,
    maze: &Maze,
    block_size: usize,
    delta_time: f32,
) {
    let taylor_speed = game_data.get_current_level().taylor_speed;
    let current_level_index = game_data.current_level;
    
    let speed_multiplier = match current_level_index {
        0 => 1.1,   
        1 => 1.3,   
        2 => 1.6,   
        _ => 1.0,
    };
    
    let effective_speed = taylor_speed * speed_multiplier;
    
    game_data.taylor_ai.update_ai(
        &mut game_data.taylor_position,
        player,
        maze,
        block_size,
        delta_time,
        effective_speed,
    );
}
fn start_level(
    maze: &mut Maze,
    sprite_manager: &mut SpriteManager,
    game_data: &mut GameData,
    player: &mut Player,
    block_size: usize,
    stream_handle: &rodio::OutputStreamHandle,
    current_sink: &mut Option<Sink>,
) {
    *maze = load_maze(&game_data.get_current_level().maze_file);
    let sprite_positions = extract_sprite_positions(&maze, block_size);
    clean_maze(maze);
    sprite_manager.initialize_gasoline_cans(&sprite_positions);
    
    player.pos = Vector2::new(150.0, 150.0);
    game_data.game_timer = 0.0;
    game_data.gasoline_collected = 0;
    
    let level = &game_data.levels[game_data.current_level];
    game_data.taylor_position = Vector2::new(level.taylor_spawn_x, level.taylor_spawn_y);
    game_data.taylor_target = game_data.taylor_position;
    
    game_data.taylor_ai = TaylorAI::new();
    
    if let Ok(game_file) = File::open("audio/getaway_car.mp3") {
        if let Ok(game_source) = Decoder::new(BufReader::new(game_file)) {
            let sink = Sink::try_new(stream_handle).unwrap();
            sink.append(game_source.repeat_infinite());
            sink.set_volume(0.4);
            sink.play();
            *current_sink = Some(sink);
        }
    }
}


fn check_taylor_collision(game_data: &GameData, player: &Player) -> bool {
    const TAYLOR_RADIUS: f32 = 25.0;
    const PLAYER_RADIUS: f32 = 15.0;
    
    let dx = game_data.taylor_position.x - player.pos.x;
    let dy = game_data.taylor_position.y - player.pos.y;
    let distance = (dx * dx + dy * dy).sqrt();
    
    let collision_distance = TAYLOR_RADIUS + PLAYER_RADIUS + match game_data.current_level {
        0 => 10.0,
        1 => 5.0,
        2 => 0.0,
        _ => 10.0,
    };
    
    distance < collision_distance
}

fn main() {
    let window_width = 1280;
    let window_height = 720;
    let block_size = 100;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Tom Hiddleston's Great Escape")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    window.hide_cursor();
    window.disable_cursor();

    println!("Buscando controles...");
    for gamepad_id in 0..8 { 
        println!("Probando gamepad {}: {}", gamepad_id, window.is_gamepad_available(gamepad_id));
        if window.is_gamepad_available(gamepad_id) {
            println!("Control {} detectado: {:?}", gamepad_id, window.get_gamepad_name(gamepad_id));
        }
    }


    if window.is_gamepad_available(1) {
        if let Some(name) = window.get_gamepad_name(1) {
            println!("Control detectado: {}", name);
        } else {
            println!("Control detectado pero sin nombre");
        }
    } else {
        println!("No se detectó ningún control");
    }

    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);
    framebuffer.set_background_color(Color::new(10, 10, 30, 255));

    let texture_manager = TextureManager::new();
    let mut sprite_manager = SpriteManager::new();
    let mut taylor_sprite = TaylorSprite::new();
    let mut game_data = GameData::new();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut current_sink: Option<Sink> = None;

    let mut state = GameState::Menu;
    let mut last_time = std::time::Instant::now();

    let mut maze = load_maze(&game_data.get_current_level().maze_file);
    let sprite_positions = extract_sprite_positions(&maze, block_size);
    clean_maze(&mut maze);
    sprite_manager.initialize_gasoline_cans(&sprite_positions);

    let mut player = Player {
        pos: Vector2::new(150.0, 150.0),
        a: PI / 4.0,
        fov: PI / 3.0,
    };

    while !window.window_should_close() {
        let current_time = std::time::Instant::now();
        let delta_time = current_time.duration_since(last_time).as_secs_f32();
        last_time = current_time;

        framebuffer.clear();

        match state {
            GameState::Menu => {
                let mut d = window.begin_drawing(&raylib_thread);
                d.clear_background(Color::BLACK);
                
                d.draw_text(
                    "TOM HIDDLESTON'S GREAT ESCAPE",
                    window_width / 2 - 400,
                    150,
                    48,
                    Color::WHITE,
                );
                
                d.draw_text(
                    "¡Escapa de Taylor Swift!",
                    window_width / 2 - 200,
                    220,
                    28,
                    Color::YELLOW,
                );
                
                d.draw_text(
                    "SELECCIONA NIVEL:",
                    window_width / 2 - 140,
                    300,
                    32,
                    Color::GREEN,
                );
                
                d.draw_text(
                    "1 - Hollywood Studio (Fácil)",
                    window_width / 2 - 200,
                    360,
                    24,
                    Color::WHITE,
                );
                
                d.draw_text(
                    "2 - Recording Studio (Medio)",
                    window_width / 2 - 210,
                    400,
                    24,
                    Color::WHITE,
                );
                
                d.draw_text(
                    "3 - Concert Venue (Difícil)",
                    window_width / 2 - 190,
                    440,
                    24,
                    Color::WHITE,
                );
                
                d.draw_text(
                    "WASD - Moverse, Mouse - Mirar",
                    window_width / 2 - 180,
                    520,
                    20,
                    Color::LIGHTGRAY,
                );
                
                d.draw_text(
                    "E - Abrir puerta (con 3 bidones)",
                    window_width / 2 - 190,
                    550,
                    20,
                    Color::LIGHTGRAY,
                );

                d.draw_text(
                    "Gamepad: O-Nivel1, △-Nivel2, □-Nivel3",
                    window_width / 2 - 200,
                    580,
                    20,
                    Color::LIGHTGRAY,
                );
                            
                if d.is_key_pressed(KeyboardKey::KEY_ONE) {
                    game_data.current_level = 0;
                    state = GameState::Playing;
                    start_level(&mut maze, &mut sprite_manager, &mut game_data, &mut player, block_size, &stream_handle, &mut current_sink);
                }
                if d.is_key_pressed(KeyboardKey::KEY_TWO) {
                    game_data.current_level = 1;
                    state = GameState::Playing;
                    start_level(&mut maze, &mut sprite_manager, &mut game_data, &mut player, block_size, &stream_handle, &mut current_sink);
                }
                if d.is_key_pressed(KeyboardKey::KEY_THREE) {
                    game_data.current_level = 2;
                    state = GameState::Playing;
                    start_level(&mut maze, &mut sprite_manager, &mut game_data, &mut player, block_size, &stream_handle, &mut current_sink);
                }

                if d.is_gamepad_available(0) {
                    if d.is_gamepad_button_pressed(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_RIGHT) {
                        game_data.current_level = 0;
                        state = GameState::Playing;
                        start_level(&mut maze, &mut sprite_manager, &mut game_data, &mut player, block_size, &stream_handle, &mut current_sink);
                    }
                    if d.is_gamepad_button_pressed(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_UP) { 
                        game_data.current_level = 1;
                        state = GameState::Playing;
                        start_level(&mut maze, &mut sprite_manager, &mut game_data, &mut player, block_size, &stream_handle, &mut current_sink);
                    }
                    if d.is_gamepad_button_pressed(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_LEFT) { 
                        game_data.current_level = 2;
                        state = GameState::Playing;
                        start_level(&mut maze, &mut sprite_manager, &mut game_data, &mut player, block_size, &stream_handle, &mut current_sink);
                    }
                }
            }

            GameState::Playing => {
                game_data.game_timer += delta_time;
                
                let dx = game_data.taylor_position.x - player.pos.x;
                let dy = game_data.taylor_position.y - player.pos.y;
                let taylor_distance = (dx * dx + dy * dy).sqrt();
                taylor_sprite.update(delta_time, taylor_distance);
                
                update_taylor_ai(&mut game_data, &player, &maze, block_size, delta_time);
                
                if check_taylor_collision(&game_data, &player) {
                    state = GameState::GameOver;
                    if let Some(sink) = &current_sink {
                        sink.stop();
                    }
                    
                    if let Ok(death_file) = File::open("audio/caught.mp3") {
                        if let Ok(death_source) = Decoder::new(BufReader::new(death_file)) {
                            if let Ok(effect_sink) = Sink::try_new(&stream_handle) {
                                effect_sink.append(death_source);
                                effect_sink.set_volume(0.8);
                                effect_sink.play();
                                effect_sink.detach();
                            }
                        }
                    }
                    continue;
                }

                process_events(&mut player, &window, &maze, block_size);
                
                let mouse_delta = window.get_mouse_delta();
                const MOUSE_SENSITIVITY: f32 = 0.003;
                player.a += mouse_delta.x * MOUSE_SENSITIVITY;

                if window.is_gamepad_available(0) {
                    let right_stick_x = window.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_RIGHT_X);
                    if right_stick_x.abs() > 0.2 {
                        player.a += right_stick_x * 0.05; 
                    }
                }

                if player.a < 0.0 { player.a += 2.0 * PI; }
                else if player.a > 2.0 * PI { player.a -= 2.0 * PI; }

                sprite_manager.update(delta_time);

                if let Some(_) = sprite_manager.check_collision(&player, 30.0) {
                    game_data.gasoline_collected += 1;
                    
                    if let Ok(collect_file) = File::open("audio/gasoline_pickup.mp3") {
                        if let Ok(collect_source) = Decoder::new(BufReader::new(collect_file)) {
                            if let Ok(effect_sink) = Sink::try_new(&stream_handle) {
                                effect_sink.append(collect_source);
                                effect_sink.set_volume(0.8);
                                effect_sink.play();
                                effect_sink.detach();
                            }
                        }
                    }
                    
                    println!("¡Gasolina recolectada! {}/{}", 
                            game_data.gasoline_collected, 
                            game_data.get_current_level().required_cans);
                }

                if game_data.gasoline_collected >= game_data.get_current_level().required_cans {
                    if window.is_key_down(KeyboardKey::KEY_E) || 
                    (window.is_gamepad_available(0) && window.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN)) {
                        let car_pos = Vector2::new(6400.0, 150.0);
                        let dx = player.pos.x - car_pos.x;
                        let dy = player.pos.y - car_pos.y;
                        let distance = (dx * dx + dy * dy).sqrt();
                        
                        if distance < 80.0 {
                            if game_data.current_level >= game_data.levels.len() - 1 {
                                state = GameState::Victory;
                            } else {
                                state = GameState::LevelComplete;
                            }
                            
                            if let Some(sink) = &current_sink {
                                sink.stop();
                            }
                            
                            if let Ok(success_file) = File::open("audio/level_complete.mp3") {
                                if let Ok(success_source) = Decoder::new(BufReader::new(success_file)) {
                                    if let Ok(effect_sink) = Sink::try_new(&stream_handle) {
                                        effect_sink.append(success_source);
                                        effect_sink.set_volume(0.8);
                                        effect_sink.play();
                                        effect_sink.detach();
                                    }
                                }
                            }
                            continue;
                        }
                    }
                }

                render_world_with_textures_sprites_and_taylor(
                    &mut framebuffer,
                    &maze,
                    block_size,
                    &player,
                    &texture_manager,
                    &sprite_manager,
                    &taylor_sprite,
                    game_data.taylor_position,
                );

                let framebuffer_texture = framebuffer.get_texture(&mut window, &raylib_thread);

                let mut d = window.begin_drawing(&raylib_thread);
                d.clear_background(Color::BLACK);
                
                if let Ok(texture) = &framebuffer_texture {
                    d.draw_texture(texture, 0, 0, Color::WHITE);
                }
                
                d.draw_text(
                    &format!("Nivel: {}", game_data.get_current_level().level_name),
                    20, 20, 28, Color::WHITE,
                );
                
                d.draw_text(
                    &format!("Gasolina: {}/{}", 
                            game_data.gasoline_collected, 
                            game_data.get_current_level().required_cans),
                    20, 55, 28, Color::YELLOW,
                );
                
                d.draw_text(
                    &format!("Tiempo: {:.1}s", game_data.game_timer),
                    20, 90, 28, Color::WHITE,
                );
                
                if taylor_distance < 150.0 {
                    d.draw_text(
                        "¡TAYLOR ESTÁ CERCA!",
                        window_width / 2 - 200, 160,
                        36, Color::RED,
                    );
                }
                
                let minimap_size = 300;
                let map_rows = maze.len();
                let map_cols = maze[0].len();
                let scale_x = minimap_size as f32 / (map_cols * block_size) as f32;
                let scale_y = minimap_size as f32 / (map_rows * block_size) as f32;
                let offset_x = window_width - minimap_size - 20;
                let offset_y = 20;

                d.draw_rectangle(offset_x, offset_y, minimap_size, minimap_size, Color::new(0, 0, 0, 150));

                for (row_index, row) in maze.iter().enumerate() {
                    for (col_index, &cell) in row.iter().enumerate() {
                        if cell != ' ' {
                            let x = offset_x + ((col_index * block_size) as f32 * scale_x) as i32;
                            let y = offset_y + ((row_index * block_size) as f32 * scale_y) as i32;
                            let cell_width = (block_size as f32 * scale_x) as i32;
                            let cell_height = (block_size as f32 * scale_y) as i32;
                            d.draw_rectangle(x, y, cell_width, cell_height, Color::GRAY);
                        }
                    }
                }

                for sprite in &sprite_manager.sprites {
                    if !sprite.collected {
                        let sx = offset_x + (sprite.x * scale_x) as i32;
                        let sy = offset_y + (sprite.y * scale_y) as i32;
                        d.draw_circle(sx, sy, 6.0, Color::ORANGE);
                    }
                }

                let exit_x = offset_x + (6400.0 * scale_x) as i32;
                let exit_y = offset_y + (150.0 * scale_y) as i32;
                d.draw_rectangle(exit_x - 8, exit_y - 8, 16, 16, Color::BLUE);
                d.draw_text("E", exit_x - 4, exit_y - 6, 12, Color::WHITE);


                let tx = offset_x + (game_data.taylor_position.x * scale_x) as i32;
                let ty = offset_y + (game_data.taylor_position.y * scale_y) as i32;
                d.draw_circle(tx, ty, 8.0, Color::RED);

                let px = offset_x + (player.pos.x * scale_x) as i32;
                let py = offset_y + (player.pos.y * scale_y) as i32;
                d.draw_circle(px, py, 6.0, Color::GREEN);

                d.draw_rectangle_lines_ex(Rectangle::new(offset_x as f32, offset_y as f32, minimap_size as f32, minimap_size as f32), 3.0, Color::WHITE);

                d.draw_fps(20, window_height - 60);

                d.draw_text("WASD para moverse", 20, window_height - 120, 24, Color::LIGHTGRAY);
                
                if game_data.gasoline_collected >= game_data.get_current_level().required_cans {
                    d.draw_text(
                        "¡Presiona E cerca de la salida para escapar!",
                        window_width / 2 - 300, 240,
                        28, Color::GREEN,
                    );
                }
            }

            GameState::LevelComplete => {
                let mut d = window.begin_drawing(&raylib_thread);
                d.clear_background(Color::BLACK);
                
                d.draw_text(
                    "¡NIVEL COMPLETO!",
                    window_width / 2 - 200,
                    window_height / 2 - 140,
                    48,
                    Color::GREEN,
                );
                
                d.draw_text(
                    &format!("Tiempo: {:.1} segundos", game_data.game_timer),
                    window_width / 2 - 160,
                    window_height / 2 - 70,
                    28,
                    Color::WHITE,
                );
                
                d.draw_text(
                    "Presiona ESPACIO para continuar",
                    window_width / 2 - 200,
                    window_height / 2,
                    28,
                    Color::YELLOW,
                );

                if d.is_key_pressed(KeyboardKey::KEY_SPACE) {
                    game_data.next_level();
                    
                    maze = load_maze(&game_data.get_current_level().maze_file);
                    let sprite_positions = extract_sprite_positions(&maze, block_size);
                    clean_maze(&mut maze);
                    sprite_manager.initialize_gasoline_cans(&sprite_positions);
                    
                    player.pos = Vector2::new(150.0, 150.0);
                    game_data.game_timer = 0.0;
                    
                    state = GameState::Playing;
                    
                    if let Ok(game_file) = File::open("audio/getaway_car.mp3") {
                        if let Ok(game_source) = Decoder::new(BufReader::new(game_file)) {
                            let sink = Sink::try_new(&stream_handle).unwrap();
                            sink.append(game_source.repeat_infinite());
                            sink.set_volume(0.4);
                            sink.play();
                            current_sink = Some(sink);
                        }
                    }
                }
            }

            GameState::GameOver => {
                let mut d = window.begin_drawing(&raylib_thread);
                d.clear_background(Color::BLACK);
                
                d.draw_text(
                    "¡GAME OVER!",
                    window_width / 2 - 160,
                    window_height / 2 - 140,
                    48,
                    Color::RED,
                );
                
                d.draw_text(
                    "¡Taylor Swift te atrapó!",
                    window_width / 2 - 180,
                    window_height / 2 - 70,
                    28,
                    Color::WHITE,
                );
                
                d.draw_text(
                    "Presiona R para reintentar",
                    window_width / 2 - 160,
                    window_height / 2,
                    28,
                    Color::YELLOW,
                );

                if d.is_key_pressed(KeyboardKey::KEY_R) {
                    game_data.reset_level();
                    player.pos = Vector2::new(150.0, 150.0);
                    game_data.game_timer = 0.0;
                    
                    maze = load_maze(&game_data.get_current_level().maze_file);
                    let sprite_positions = extract_sprite_positions(&maze, block_size);
                    clean_maze(&mut maze);
                    sprite_manager.initialize_gasoline_cans(&sprite_positions);
                    
                    state = GameState::Playing;
                    
                    if let Ok(game_file) = File::open("audio/getaway_car.mp3") {
                        if let Ok(game_source) = Decoder::new(BufReader::new(game_file)) {
                            let sink = Sink::try_new(&stream_handle).unwrap();
                            sink.append(game_source.repeat_infinite());
                            sink.set_volume(0.4);
                            sink.play();
                            current_sink = Some(sink);
                        }
                    }
                }
            }

            GameState::Victory => {
                let mut d = window.begin_drawing(&raylib_thread);
                d.clear_background(Color::BLACK);
                
                d.draw_text(
                    "¡FELICIDADES!",
                    window_width / 2 - 200,
                    window_height / 2 - 140,
                    48,
                    Color::GREEN,
                );
                
                d.draw_text(
                    "¡Tom Hiddleston escapó exitosamente!",
                    window_width / 2 - 280,
                    window_height / 2 - 70,
                    28,
                    Color::WHITE,
                );
                
                d.draw_text(
                    "Presiona ESC para salir",
                    window_width / 2 - 140,
                    window_height / 2,
                    28,
                    Color::YELLOW,
                );

                if d.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    break;
                }
            }
        }

        thread::sleep(Duration::from_millis(16));
    }

    if let Some(sink) = current_sink {
        sink.stop();
    }
}