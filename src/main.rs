// src/main.rs - Tom Hiddleston's Great Escape Game
mod line;
mod framebuffer;
mod maze;
mod caster;
mod player;
mod texture;
mod sprites;
mod taylor_sprite;

use line::line;
use maze::{Maze, load_maze, extract_sprite_positions, clean_maze};
use caster::{cast_ray, render_world_with_textures_sprites_and_taylor};
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
}

impl GameData {
    fn new() -> Self {
        let levels = vec![
            GameLevel {
                maze_file: "mazes/level1.txt".to_string(),
                level_name: "Hollywood Studio".to_string(),
                required_cans: 4,
                taylor_speed: 2.0,
                taylor_spawn_x: 250.0,  // Posición dentro del mapa
                taylor_spawn_y: 950.0,
            },
            GameLevel {
                maze_file: "mazes/level2.txt".to_string(),
                level_name: "Recording Studio".to_string(),
                required_cans: 4,
                taylor_speed: 2.5,
                taylor_spawn_x: 250.0,
                taylor_spawn_y: 950.0,
            },
            GameLevel {
                maze_file: "mazes/level3.txt".to_string(),
                level_name: "Concert Venue".to_string(),
                required_cans: 4,
                taylor_speed: 3.0,
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
    // Obtener los valores necesarios antes de cualquier mutación
    let taylor_speed = game_data.get_current_level().taylor_speed;
    let current_level_index = game_data.current_level;
    
    game_data.taylor_last_move_time += delta_time;

    let update_frequency = match current_level_index {
        0 => 2.0,  
        1 => 1.5,  
        2 => 1.0,  
        _ => 1.0,
    };

    if game_data.taylor_last_move_time >= update_frequency {
        let prediction_time = 1.0;
        let predicted_x = player.pos.x + (player.a.cos() * 50.0 * prediction_time);
        let predicted_y = player.pos.y + (player.a.sin() * 50.0 * prediction_time);
        
        game_data.taylor_target = Vector2::new(predicted_x, predicted_y);
        game_data.taylor_last_move_time = 0.0;
    }

    let dx = game_data.taylor_target.x - game_data.taylor_position.x;
    let dy = game_data.taylor_target.y - game_data.taylor_position.y;
    let distance = (dx * dx + dy * dy).sqrt();

    if distance > 5.0 {
        let speed_multiplier = match current_level_index {
            0 => 1.0,   
            1 => 1.2,   
            2 => 1.5,   
            _ => 1.0,
        };
        
        let effective_speed = taylor_speed * speed_multiplier;
        let move_x = (dx / distance) * effective_speed * delta_time * 60.0;
        let move_y = (dy / distance) * effective_speed * delta_time * 60.0;

        let new_x = game_data.taylor_position.x + move_x;
        let new_y = game_data.taylor_position.y + move_y;

        if is_valid_position(maze, new_x, game_data.taylor_position.y, block_size) {
            game_data.taylor_position.x = new_x;
        }
        if is_valid_position(maze, game_data.taylor_position.x, new_y, block_size) {
            game_data.taylor_position.y = new_y;
        }

        if distance < 10.0 && game_data.taylor_last_move_time > 5.0 {
            for _ in 0..10 {
                let angle = (game_data.game_timer * 2.0) % (2.0 * PI);
                let dist = 200.0 + (game_data.game_timer % 1.0) * 300.0;
                let new_x = player.pos.x + angle.cos() * dist;
                let new_y = player.pos.y + angle.sin() * dist;
                
                if is_valid_position(maze, new_x, new_y, block_size) {
                    game_data.taylor_position.x = new_x;
                    game_data.taylor_position.y = new_y;
                    break;
                }
            }
        }
    }
}

fn is_valid_position(maze: &Maze, x: f32, y: f32, block_size: usize) -> bool {
    let grid_x = (x / block_size as f32) as usize;
    let grid_y = (y / block_size as f32) as usize;
    
    if grid_y >= maze.len() || grid_x >= maze[0].len() {
        return false;
    }
    
    maze[grid_y][grid_x] == ' '
}

fn check_taylor_collision(game_data: &GameData, player: &Player) -> bool {
    let dx = game_data.taylor_position.x - player.pos.x;
    let dy = game_data.taylor_position.y - player.pos.y;
    let distance = (dx * dx + dy * dy).sqrt();
    
    let capture_distance = match game_data.current_level {
        0 => 50.0,  
        1 => 45.0,  
        2 => 40.0,  
        _ => 50.0,
    };
    
    distance < capture_distance
}

fn render_minimap_with_entities(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    game_data: &GameData,
    sprite_manager: &SpriteManager,
    minimap_size: usize,
    block_size: usize,
) {
    let map_rows = maze.len();
    let map_cols = maze[0].len();
    let scale_x = minimap_size as f32 / (map_cols * block_size) as f32;
    let scale_y = minimap_size as f32 / (map_rows * block_size) as f32;
    let offset_x = framebuffer.width as usize - minimap_size - 10;
    let offset_y = 10;

    framebuffer.set_current_color(Color::new(20, 20, 20, 255));
    for x in offset_x..offset_x + minimap_size {
        for y in offset_y..offset_y + minimap_size {
            if x < framebuffer.width as usize && y < framebuffer.height as usize {
                framebuffer.set_pixel(x as u32, y as u32);
            }
        }
    }

    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            if cell != ' ' {
                let x = offset_x + ((col_index * block_size) as f32 * scale_x) as usize;
                let y = offset_y + ((row_index * block_size) as f32 * scale_y) as usize;
                let cell_width = (block_size as f32 * scale_x) as usize;
                let cell_height = (block_size as f32 * scale_y) as usize;
                
                framebuffer.set_current_color(Color::GRAY);
                for dx in 0..cell_width {
                    for dy in 0..cell_height {
                        let px = x + dx;
                        let py = y + dy;
                        if px < framebuffer.width as usize && py < framebuffer.height as usize {
                            framebuffer.set_pixel(px as u32, py as u32);
                        }
                    }
                }
            }
        }
    }

    sprite_manager.render_minimap_sprites(framebuffer, maze, minimap_size, block_size);

    let px = offset_x as f32 + player.pos.x * scale_x;
    let py = offset_y as f32 + player.pos.y * scale_y;
    if px >= offset_x as f32 && px < (offset_x + minimap_size) as f32 
       && py >= offset_y as f32 && py < (offset_y + minimap_size) as f32 {
        framebuffer.set_current_color(Color::GREEN);
        let radius = 3;
        for dy in -(radius as i32)..=(radius as i32) {
            for dx in -(radius as i32)..=(radius as i32) {
                if dx * dx + dy * dy <= radius * radius {
                    let x = (px as i32 + dx) as u32;
                    let y = (py as i32 + dy) as u32;
                    if x < framebuffer.width && y < framebuffer.height {
                        framebuffer.set_pixel(x, y);
                    }
                }
            }
        }
    }

    let tx = offset_x as f32 + game_data.taylor_position.x * scale_x;
    let ty = offset_y as f32 + game_data.taylor_position.y * scale_y;
    if tx >= offset_x as f32 && tx < (offset_x + minimap_size) as f32 
       && ty >= offset_y as f32 && ty < (offset_y + minimap_size) as f32 {
        framebuffer.set_current_color(Color::RED);
        let radius = 4;
        for dy in -(radius as i32)..=(radius as i32) {
            for dx in -(radius as i32)..=(radius as i32) {
                if dx * dx + dy * dy <= radius * radius {
                    let x = (tx as i32 + dx) as u32;
                    let y = (ty as i32 + dy) as u32;
                    if x < framebuffer.width && y < framebuffer.height {
                        framebuffer.set_pixel(x, y);
                    }
                }
            }
        }
    }

    framebuffer.set_current_color(Color::WHITE);
    for thickness in 0..2 {
        for x in offset_x..offset_x + minimap_size {
            if x < framebuffer.width as usize {
                framebuffer.set_pixel(x as u32, (offset_y + thickness) as u32);
                framebuffer.set_pixel(x as u32, (offset_y + minimap_size - 1 - thickness) as u32);
            }
        }
        for y in offset_y..offset_y + minimap_size {
            if y < framebuffer.height as usize {
                framebuffer.set_pixel((offset_x + thickness) as u32, y as u32);
                framebuffer.set_pixel((offset_x + minimap_size - 1 - thickness) as u32, y as u32);
            }
        }
    }
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

fn main() {
    let window_width = 1000;
    let window_height = 650;
    let block_size = 100;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Tom Hiddleston's Great Escape")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    window.hide_cursor();

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
                    window_width / 2 - 200,
                    80,
                    24,
                    Color::WHITE,
                );
                
                d.draw_text(
                    "¡Escapa de Taylor Swift!",
                    window_width / 2 - 120,
                    130,
                    16,
                    Color::YELLOW,
                );
                
                d.draw_text(
                    "SELECCIONA NIVEL:",
                    window_width / 2 - 80,
                    180,
                    18,
                    Color::GREEN,
                );
                
                d.draw_text(
                    "1 - Hollywood Studio (Fácil)",
                    window_width / 2 - 120,
                    220,
                    16,
                    Color::WHITE,
                );
                
                d.draw_text(
                    "2 - Recording Studio (Medio)",
                    window_width / 2 - 120,
                    250,
                    16,
                    Color::WHITE,
                );
                
                d.draw_text(
                    "3 - Concert Venue (Difícil)",
                    window_width / 2 - 120,
                    280,
                    16,
                    Color::WHITE,
                );
                
                d.draw_text(
                    "WASD - Moverse, Mouse - Mirar",
                    window_width / 2 - 100,
                    330,
                    12,
                    Color::LIGHTGRAY,
                );
                
                d.draw_text(
                    "E - Abrir puerta (con 3 bidones)",
                    window_width / 2 - 100,
                    350,
                    12,
                    Color::LIGHTGRAY,
                );
            
                // Selección de nivel
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
                
                const MOUSE_SENSITIVITY: f32 = 0.005;
                let mouse_delta_x = window.get_mouse_delta().x;
                player.a += mouse_delta_x * MOUSE_SENSITIVITY;
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
            
                // Verificar si puede abrir la puerta
                if game_data.gasoline_collected >= game_data.get_current_level().required_cans {
                    if window.is_key_down(KeyboardKey::KEY_E) {
                        let car_pos = Vector2::new(6400.0, 150.0); // Posición de la E en los mapas grandes
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
            
                // USAR SOLO RAYLIB PARA RENDERIZAR
                let mut d = window.begin_drawing(&raylib_thread);
                d.clear_background(Color::new(20, 20, 40, 255));
                
                // Dibujar un mundo 3D simple
                let num_rays = 200; // Más rayos para mejor calidad
                let hh = window_height as f32 / 2.0;
                
                for i in 0..num_rays {
                    let current_ray = i as f32 / num_rays as f32;
                    let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
                    let intersect = cast_ray(&mut framebuffer, &maze, &player, a, block_size, false);
                    
                    if intersect.impact != ' ' {
                        let corrected_distance = intersect.distance * (a - player.a).cos();
                        let distance_to_projection_plane = 250.0;
                        let stake_height = (hh / corrected_distance) * distance_to_projection_plane;
                        let stake_top = (hh - (stake_height / 2.0)).max(0.0) as i32;
                        let stake_bottom = (hh + (stake_height / 2.0)).min(window_height as f32) as i32;
                        
                        let x = (i as f32 * (window_width as f32 / num_rays as f32)) as i32;
                        
                        // Color basado en la distancia
                        let color_intensity = (255.0 * (1.0 - (corrected_distance / 500.0))).max(50.0) as u8;
                        let wall_color = Color::new(color_intensity, color_intensity / 2, 0, 255);
                        
                        // Dibujar columna de pared
                        d.draw_line(x, stake_top, x, stake_bottom, wall_color);
                    }
                }

                                // Dibujar la salida (E) con color especial
                for (row_index, row) in maze.iter().enumerate() {
                    for (col_index, &cell) in row.iter().enumerate() {
                        if cell == 'E' {
                            let exit_x = (col_index * block_size) as f32 + (block_size as f32 / 2.0);
                            let exit_y = (row_index * block_size) as f32 + (block_size as f32 / 2.0);
                            
                            let dx = exit_x - player.pos.x;
                            let dy = exit_y - player.pos.y;
                            let distance = (dx * dx + dy * dy).sqrt();
                            
                            if distance > 50.0 && distance < 400.0 {
                                let angle_to_exit = dy.atan2(dx);
                                let angle_diff = angle_to_exit - player.a;
                                let normalized_angle = ((angle_diff + PI) % (2.0 * PI)) - PI;
                                
                                if normalized_angle.abs() < player.fov / 2.0 {
                                    let screen_x = window_width / 2 + (normalized_angle * window_width as f32 / player.fov) as i32;
                                    let exit_size = (60.0 / distance * 120.0) as i32;
                                    
                                    let exit_color = if game_data.gasoline_collected >= game_data.get_current_level().required_cans {
                                        Color::GREEN
                                    } else {
                                        Color::DARKGRAY
                                    };
                                    
                                    d.draw_rectangle(screen_x - exit_size/2, window_height / 2 - exit_size/2, exit_size, exit_size, exit_color);
                                }
                            }
                        }
                    }
                }
                
                // Dibujar sprites (bidones de gasolina)
                for sprite in &sprite_manager.sprites {
                    if !sprite.collected {
                        let dx = sprite.x - player.pos.x;
                        let dy = sprite.y - player.pos.y;
                        let distance = (dx * dx + dy * dy).sqrt();
                        
                        if distance > 50.0 && distance < 400.0 {
                            let angle_to_sprite = dy.atan2(dx);
                            let angle_diff = angle_to_sprite - player.a;
                            let normalized_angle = ((angle_diff + PI) % (2.0 * PI)) - PI;
                            
                            if normalized_angle.abs() < player.fov / 2.0 {
                                let screen_x = window_width / 2 + (normalized_angle * window_width as f32 / player.fov) as i32;
                                let sprite_size = (50.0 / distance * 100.0) as i32;
                                
                                d.draw_circle(screen_x, window_height / 2, sprite_size as f32, Color::ORANGE);
                            }
                        }
                    }
                }
                
                // Dibujar Taylor Swift
                if taylor_distance > 50.0 && taylor_distance < 600.0 {
                    let angle_to_taylor = dy.atan2(dx);
                    let angle_diff = angle_to_taylor - player.a;
                    let normalized_angle = ((angle_diff + PI) % (2.0 * PI)) - PI;
                    
                    if normalized_angle.abs() < player.fov / 2.0 {
                        let screen_x = window_width / 2 + (normalized_angle * window_width as f32 / player.fov) as i32;
                        let taylor_size = (80.0 / taylor_distance * 150.0) as i32;
                        
                        let taylor_color = if taylor_distance < 100.0 { Color::DARKRED } else { Color::RED };
                        d.draw_circle(screen_x, window_height / 2, taylor_size as f32, taylor_color);
                    }
                }
                
                // UI del juego
                d.draw_text(
                    &format!("Nivel: {}", game_data.get_current_level().level_name),
                    10, 10, 16, Color::WHITE,
                );
                
                d.draw_text(
                    &format!("Gasolina: {}/{}", 
                             game_data.gasoline_collected, 
                             game_data.get_current_level().required_cans),
                    10, 30, 16, Color::YELLOW,
                );
                
                d.draw_text(
                    &format!("Tiempo: {:.1}s", game_data.game_timer),
                    10, 50, 16, Color::WHITE,
                );
                
                if taylor_distance < 150.0 {
                    d.draw_text(
                        "¡TAYLOR ESTÁ CERCA!",
                        window_width / 2 - 100, 100,
                        20, Color::RED,
                    );
                }
                
                // MINIMAPA
                let minimap_size = 150;
                let map_rows = maze.len();
                let map_cols = maze[0].len();
                let scale_x = minimap_size as f32 / (map_cols * block_size) as f32;
                let scale_y = minimap_size as f32 / (map_rows * block_size) as f32;
                let offset_x = window_width - minimap_size - 10;
                let offset_y = 10;
            
                // Fondo minimapa
                d.draw_rectangle(offset_x, offset_y, minimap_size, minimap_size, Color::new(0, 0, 0, 150));
            
                // Paredes en minimapa
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
            
                // Bidones en minimapa
                for sprite in &sprite_manager.sprites {
                    if !sprite.collected {
                        let sx = offset_x + (sprite.x * scale_x) as i32;
                        let sy = offset_y + (sprite.y * scale_y) as i32;
                        d.draw_circle(sx, sy, 3.0, Color::ORANGE);
                    }
                }
            
                // Taylor en minimapa
                let tx = offset_x + (game_data.taylor_position.x * scale_x) as i32;
                let ty = offset_y + (game_data.taylor_position.y * scale_y) as i32;
                d.draw_circle(tx, ty, 4.0, Color::RED);
            
                // Jugador en minimapa
                let px = offset_x + (player.pos.x * scale_x) as i32;
                let py = offset_y + (player.pos.y * scale_y) as i32;
                d.draw_circle(px, py, 3.0, Color::GREEN);
            
                // Borde minimapa
                d.draw_rectangle_lines_ex(Rectangle::new(offset_x as f32, offset_y as f32, minimap_size as f32, minimap_size as f32), 2.0, Color::WHITE);
            
                // FPS
                d.draw_fps(10, window_height - 30);
            
                // Instrucciones
                d.draw_text("WASD para moverse", 10, window_height - 60, 12, Color::LIGHTGRAY);
                
                // Instrucción para abrir puerta
                if game_data.gasoline_collected >= game_data.get_current_level().required_cans {
                    d.draw_text(
                        "¡Presiona E cerca de la salida para escapar!",
                        window_width / 2 - 150, 160,
                        16, Color::GREEN,
                    );
                }
            }

            GameState::LevelComplete => {
                let mut d = window.begin_drawing(&raylib_thread);
                d.clear_background(Color::BLACK);
                
                d.draw_text(
                    "¡NIVEL COMPLETO!",
                    window_width / 2 - 100, 200,
                    24, Color::GREEN,
                );
                
                d.draw_text(
                    &format!("Tiempo: {:.1} segundos", game_data.game_timer),
                    window_width / 2 - 80, 250,
                    16, Color::WHITE,
                );
                
                d.draw_text(
                    "Presiona ESPACIO para continuar",
                    window_width / 2 - 120, 300,
                    16, Color::YELLOW,
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
                    window_width / 2 - 80, 200,
                    24, Color::RED,
                );
                
                d.draw_text(
                    "¡Taylor Swift te atrapó!",
                    window_width / 2 - 90, 250,
                    16, Color::WHITE,
                );
                
                d.draw_text(
                    "Presiona R para reintentar",
                    window_width / 2 - 90, 300,
                    16, Color::YELLOW,
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
                    window_width / 2 - 100, 200,
                    24, Color::GREEN,
                );
                
                d.draw_text(
                    "¡Tom Hiddleston escapó exitosamente!",
                    window_width / 2 - 150, 250,
                    16, Color::WHITE,
                );
                
                d.draw_text(
                    "Presiona ESC para salir",
                    window_width / 2 - 80, 300,
                    16, Color::YELLOW,
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