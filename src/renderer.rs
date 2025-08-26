// src/renderer.rs
use raylib::prelude::*;
use crate::player::Player;

pub struct RayHit {
    pub distance: f32,
    pub wall_type: char,
}

pub struct Renderer {
    animation_offset: f32,
}

impl Renderer {
    pub fn new(_rl: &mut RaylibHandle, _thread: &RaylibThread) -> Self {
        Self {
            animation_offset: 0.0,
        }
    }
    
    pub fn render(&mut self, d: &mut RaylibDrawHandle, player: &Player, 
                  level: &Vec<Vec<char>>, enemies: &Vec<(f32, f32)>, 
                  goal: &(f32, f32)) {
        let width = 1024;
        let height = 768;
        
        self.animation_offset += 0.02;
        if self.animation_offset > 1.0 {
            self.animation_offset = 0.0;
        }
        
        d.draw_rectangle(0, 0, width, height / 2, Color::SKYBLUE);
        d.draw_rectangle(0, height / 2, width, height / 2, 
                        Color::new(50, 50, 50, 255));
        
        let num_rays = width;
        let angle_step = player.fov / num_rays as f32;
        let start_angle = player.a - player.fov / 2.0;
        
        for i in 0..num_rays {
            let ray_angle = start_angle + i as f32 * angle_step;
            
            if let Some(hit) = self.cast_ray(player, ray_angle, level) {
                let corrected_distance = hit.distance * (ray_angle - player.a).cos();
                let wall_height = (height as f32 / corrected_distance) as i32;
                let wall_top = (height / 2) - (wall_height / 2);
                
                let color = match hit.wall_type {
                    '+' => Color::DARKGRAY,
                    '-' => Color::GRAY,
                    '|' => Color::LIGHTGRAY,
                    '#' => Color::BROWN,
                    _ => Color::WHITE,
                };
                
                let shade = (1.0 - (hit.distance / 20.0).min(0.8)) * 255.0;
                let shaded_color = Color::new(
                    (color.r as f32 * shade / 255.0) as u8,
                    (color.g as f32 * shade / 255.0) as u8,
                    (color.b as f32 * shade / 255.0) as u8,
                    255
                );
                
                d.draw_line(i as i32, wall_top, i as i32, wall_top + wall_height, shaded_color);
            }
        }
        
        self.render_sprites(d, player, enemies, goal, height);
    }
    
    fn cast_ray(&self, player: &Player, angle: f32, level: &Vec<Vec<char>>) -> Option<RayHit> {
        let mut distance = 0.0;
        let step_size = 0.1;
        let max_distance = 20.0;
        
        let dx = angle.cos() * step_size;
        let dy = angle.sin() * step_size;
        
        let mut ray_x = player.pos.x;
        let mut ray_y = player.pos.y;
        
        while distance < max_distance {
            ray_x += dx;
            ray_y += dy;
            distance += step_size;
            
            let grid_x = ray_x as usize;
            let grid_y = ray_y as usize;
            
            if grid_y >= level.len() || grid_x >= level[0].len() {
                break;
            }
            
            let cell = level[grid_y][grid_x];
            if cell != ' ' {
                return Some(RayHit {
                    distance,
                    wall_type: cell,
                });
            }
        }
        
        None
    }
    
    fn render_sprites(&mut self, d: &mut RaylibDrawHandle, player: &Player,
                      enemies: &Vec<(f32, f32)>, goal: &(f32, f32), screen_height: i32) {
        for enemy in enemies {
            self.render_sprite(d, player, enemy.0, enemy.1, Color::PINK, 
                             screen_height, true);
        }
        
        self.render_sprite(d, player, goal.0, goal.1, Color::GREEN, 
                         screen_height, false);
    }
    
    fn render_sprite(&mut self, d: &mut RaylibDrawHandle, player: &Player,
                     sprite_x: f32, sprite_y: f32, color: Color, 
                     screen_height: i32, animate: bool) {
        let dx = sprite_x - player.pos.x;
        let dy = sprite_y - player.pos.y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        if distance < 0.5 || distance > 20.0 {
            return;
        }
        
        let sprite_angle = dy.atan2(dx);
        let mut angle_diff = sprite_angle - player.a;
        
        while angle_diff > std::f32::consts::PI {
            angle_diff -= 2.0 * std::f32::consts::PI;
        }
        while angle_diff < -std::f32::consts::PI {
            angle_diff += 2.0 * std::f32::consts::PI;
        }
        
        if angle_diff.abs() < player.fov / 2.0 {
            let screen_x = 512.0 + (angle_diff / player.fov) * 1024.0;
            let sprite_height = (screen_height as f32 / distance) as i32;
            let sprite_width = sprite_height;
            let sprite_top = (screen_height / 2) - (sprite_height / 2);
            
            let y_offset = if animate {
                (self.animation_offset * 10.0).sin() * 5.0
            } else {
                0.0
            };
            
            d.draw_rectangle(
                screen_x as i32 - sprite_width / 2,
                sprite_top + y_offset as i32,
                sprite_width,
                sprite_height,
                color
            );
            
            if animate {
                d.draw_circle(
                    screen_x as i32,
                    sprite_top + (sprite_height / 4) + y_offset as i32,
                    sprite_width as f32 / 3.0,
                    Color::GOLD
                );
            } else {
                d.draw_circle(
                    screen_x as i32 - sprite_width / 3,
                    sprite_top + sprite_height - 10 + y_offset as i32,
                    8.0,
                    Color::BLACK
                );
                d.draw_circle(
                    screen_x as i32 + sprite_width / 3,
                    sprite_top + sprite_height - 10 + y_offset as i32,
                    8.0,
                    Color::BLACK
                );
            }
        }
    }
}