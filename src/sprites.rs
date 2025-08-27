use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use crate::maze::{Maze, SpritePosition};
use crate::player::Player;
use crate::texture::ImageTexture;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SpriteType {
    GasolineCan,
    Car,
}

#[derive(Clone, Debug)]
pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub texture_frames: Vec<ImageTexture>,
    pub scale: f32,
    pub animation_frame: usize,
    pub animation_timer: f32,
    pub collected: bool,
    pub sprite_type: SpriteType,
}

pub struct SpriteManager {
    pub sprites: Vec<Sprite>,
    pub animation_frame_duration: f32,
    pub gasoline_can_textures: Vec<ImageTexture>,
}

impl SpriteManager {
    pub fn new() -> Self {
        let gasoline_can_textures = vec![
            ImageTexture::from_file("assets/gasoline_can_1.png"),
            ImageTexture::from_file("assets/gasoline_can_2.png"),
            ImageTexture::from_file("assets/gasoline_can_3.png"),
        ];
        
        Self {
            sprites: Vec::new(),
            animation_frame_duration: 0.8,
            gasoline_can_textures,
        }
    }

    pub fn initialize_gasoline_cans(&mut self, positions: &[SpritePosition]) {
        self.sprites.clear();
        
        for position in positions {
            let sprite = Sprite {
                x: position.x,
                y: position.y,
                texture_frames: self.gasoline_can_textures.clone(),
                scale: 0.7,
                animation_frame: 0,
                animation_timer: 0.0,
                collected: false,
                sprite_type: SpriteType::GasolineCan,
            };
            self.sprites.push(sprite);
            println!("Bidón de gasolina con textura creado en ({}, {})", position.x, position.y);
        }
        
        println!("Total de bidones de gasolina: {}", self.sprites.len());
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
}