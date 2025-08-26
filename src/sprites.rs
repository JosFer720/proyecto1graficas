// src/sprites.rs - Sistema de sprites para bidones de gasolina
use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use crate::maze::{Maze, SpritePosition};
use crate::player::Player;

#[derive(Clone, Debug, PartialEq)]
pub enum SpriteType {
    GasolineCan,
    Car,
}

#[derive(Clone, Debug)]
pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub texture_frames: Vec<Vec<Color>>,
    pub scale: f32,
    pub animation_frame: usize,
    pub animation_timer: f32,
    pub collected: bool,
    pub sprite_type: SpriteType,
}

pub struct SpriteManager {
    pub sprites: Vec<Sprite>,
    pub animation_frame_duration: f32,
}

impl SpriteManager {
    pub fn new() -> Self {
        Self {
            sprites: Vec::new(),
            animation_frame_duration: 0.8,
        }
    }

    pub fn initialize_gasoline_cans(&mut self, positions: &[SpritePosition]) {
        self.sprites.clear();
        
        for position in positions {
            let sprite = Sprite {
                x: position.x,
                y: position.y,
                texture_frames: Self::create_gasoline_can_frames(),
                scale: 0.7,
                animation_frame: 0,
                animation_timer: 0.0,
                collected: false,
                sprite_type: SpriteType::GasolineCan,
            };
            self.sprites.push(sprite);
            println!("Bidón de gasolina creado en ({}, {})", position.x, position.y);
        }
        
        println!("Total de bidones de gasolina: {}", self.sprites.len());
    }

    fn create_gasoline_can_frames() -> Vec<Vec<Color>> {
        let mut frames = Vec::new();
        
        // Frame 1 - Bidón normal
        let mut texture_normal = vec![Color::new(0, 0, 0, 0); 32 * 32];
        
        for y in 0..32 {
            for x in 0..32 {
                let center_x = 16.0;
                let center_y = 18.0;
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                
                // Cuerpo principal del bidón (rectangular)
                if dx.abs() <= 10.0 && dy >= -12.0 && dy <= 10.0 {
                    if y <= 8 {
                        // Parte superior (tapa) - gris metálico
                        texture_normal[y * 32 + x] = Color::new(140, 140, 140, 255);
                    } else {
                        // Cuerpo principal - rojo
                        texture_normal[y * 32 + x] = Color::new(200, 50, 50, 255);
                    }
                }
                
                // Pico/boquilla del bidón (parte superior izquierda)
                if x >= 8 && x <= 12 && y >= 4 && y <= 8 {
                    texture_normal[y * 32 + x] = Color::new(100, 100, 100, 255);
                }
                
                // Asa del bidón (lado derecho)
                if x >= 24 && x <= 26 && y >= 12 && y <= 20 {
                    texture_normal[y * 32 + x] = Color::new(120, 120, 120, 255);
                }
                
                // Etiqueta/marca en el centro
                if dx.abs() <= 6.0 && dy >= -2.0 && dy <= 4.0 {
                    texture_normal[y * 32 + x] = Color::new(255, 255, 255, 255);
                }
            }
        }
        
        // Frame 2 - Bidón brillante (efecto de parpadeo)
        let mut texture_glow = texture_normal.clone();
        for pixel in &mut texture_glow {
            if pixel.a > 0 {
                // Hacer más brillante
                pixel.r = (pixel.r as u16 + 80).min(255) as u8;
                pixel.g = (pixel.g as u16 + 80).min(255) as u8;
                pixel.b = (pixel.b as u16 + 80).min(255) as u8;
            }
        }
        
        // Frame 3 - Bidón con brillo dorado
        let mut texture_golden = texture_normal.clone();
        for pixel in &mut texture_golden {
            if pixel.a > 0 && (pixel.r > 150 || pixel.g < 100) {
                // Efecto dorado en partes metálicas
                pixel.r = (pixel.r as u16 + 100).min(255) as u8;
                pixel.g = (pixel.g as u16 + 60).min(255) as u8;
                pixel.b = (pixel.b as u16).max(30) as u8;
            }
        }
        
        frames.push(texture_normal);
        frames.push(texture_glow);
        frames.push(texture_golden);
        frames
    }

    pub fn update(&mut self, delta_time: f32) {
        for sprite in &mut self.sprites {
            if !sprite.collected {
                sprite.animation_timer += delta_time;
                if sprite.animation_timer >= self.animation_frame_duration {
                    sprite.animation_timer = 0.0;
                    sprite.animation_frame = (sprite.animation_frame + 1) % sprite.texture_frames.len();
                }
            }
        }
    }

    pub fn render_minimap_sprites(
        &self,
        framebuffer: &mut Framebuffer,
        _maze: &Maze,
        minimap_size: usize,
        block_size: usize,
    ) {
        let map_cols = 15; // Ajustar según el tamaño del maze
        let map_rows = 11;  // Ajustar según el tamaño del maze
        let scale_x = minimap_size as f32 / (map_cols * block_size) as f32;
        let scale_y = minimap_size as f32 / (map_rows * block_size) as f32;
        let offset_x = framebuffer.width as usize - minimap_size - 10;
        let offset_y = 10;

        for sprite in &self.sprites {
            if sprite.collected { continue; }
            
            let sprite_minimap_x = offset_x + (sprite.x * scale_x) as usize;
            let sprite_minimap_y = offset_y + (sprite.y * scale_y) as usize;
            
            // Color según el tipo de sprite
            let color = match sprite.sprite_type {
                SpriteType::GasolineCan => Color::ORANGE,
                SpriteType::Car => Color::BLUE,
            };
            
            framebuffer.set_current_color(color);
            let radius = 3;
            for dy in -(radius as i32)..=(radius as i32) {
                for dx in -(radius as i32)..=(radius as i32) {
                    if dx * dx + dy * dy <= radius * radius {
                        let x = (sprite_minimap_x as i32 + dx) as u32;
                        let y = (sprite_minimap_y as i32 + dy) as u32;
                        if x < framebuffer.width && y < framebuffer.height {
                            framebuffer.set_pixel(x, y);
                        }
                    }
                }
            }
        }
    }

    pub fn check_collision(&mut self, player: &Player, collision_distance: f32) -> Option<usize> {
        for (i, sprite) in self.sprites.iter_mut().enumerate() {
            if !sprite.collected {
                let dx = sprite.x - player.pos.x;
                let dy = sprite.y - player.pos.y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance < collision_distance {
                    sprite.collected = true;
                    println!("¡Bidón de gasolina recolectado en ({}, {})!", sprite.x, sprite.y);
                    return Some(i);
                }
            }
        }
        None
    }

    pub fn get_collected_count(&self) -> usize {
        self.sprites.iter().filter(|s| s.collected).count()
    }

    pub fn get_total_count(&self) -> usize {
        self.sprites.len()
    }
}