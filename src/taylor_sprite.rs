use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use crate::player::Player;
use crate::texture::ImageTexture;
use crate::maze::Maze;
use crate::caster::cast_ray;

pub struct TaylorSprite {
    pub texture: ImageTexture,
    pub animation_timer: f32,
    pub menacing_mode: bool,
}

impl TaylorSprite {
    pub fn new() -> Self {
        Self {
            texture: ImageTexture::from_file("assets/taylor.png"),
            animation_timer: 0.0,
            menacing_mode: false,
        }
    }

    pub fn update(&mut self, delta_time: f32, distance_to_player: f32) {
        self.animation_timer += delta_time;
        
        self.menacing_mode = distance_to_player < 200.0 || (self.animation_timer % 1.0) < 0.5;
    }

    fn is_visible_from_player(
        &self,
        taylor_pos: Vector2,
        player: &Player,
        maze: &Maze,
        block_size: usize,
    ) -> bool {
        let dx = taylor_pos.x - player.pos.x;
        let dy = taylor_pos.y - player.pos.y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        let angle_to_taylor = dy.atan2(dx);
        
        let intersect = cast_ray(
            &mut crate::framebuffer::Framebuffer::new(1, 1), 
            maze,
            player,
            angle_to_taylor,
            block_size,
            false,
        );
        
        intersect.distance >= distance - 20.0 
    }

    pub fn render_3d(
        &self,
        framebuffer: &mut Framebuffer,
        taylor_pos: Vector2,
        player: &Player,
        z_buffer: &[f32],
        maze: &Maze,
        block_size: usize,
    ) {
        let dx = taylor_pos.x - player.pos.x;
        let dy = taylor_pos.y - player.pos.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < 30.0 || distance > 1000.0 { 
            return; 
        }

        if !self.is_visible_from_player(taylor_pos, player, maze, block_size) {
            return; 
        }

        let cos_angle = player.a.cos();
        let sin_angle = player.a.sin();
        let transformed_x = -dx * sin_angle + dy * cos_angle;
        let transformed_y = dx * cos_angle + dy * sin_angle;

        if transformed_y <= 10.0 { 
            return; 
        }

        let hh = framebuffer.height as f32 / 2.0;
        let distance_to_projection_plane = 300.0;
        let sprite_height = (hh / distance) * distance_to_projection_plane;
        let wall_height_factor = 1.5;
        let adjusted_sprite_height = sprite_height * wall_height_factor;

        let fov = player.fov;
        let screen_center = framebuffer.width as f32 / 2.0;
        let sprite_angle = (transformed_x / transformed_y).atan();
        let angle_from_center = sprite_angle;
        let pixels_per_radian = framebuffer.width as f32 / fov;
        let screen_x = screen_center + (angle_from_center * pixels_per_radian);

        let sprite_width = adjusted_sprite_height * 0.6;
        let start_x = (screen_x - sprite_width / 2.0) as i32;
        let end_x = (screen_x + sprite_width / 2.0) as i32;
        let start_y = (hh - adjusted_sprite_height / 2.0).max(0.0) as i32;
        let end_y = (hh + adjusted_sprite_height / 2.0).min(framebuffer.height as f32) as i32;

        if start_x >= framebuffer.width as i32 || end_x < 0 ||
           start_y >= framebuffer.height as i32 || end_y < 0 { 
            return; 
        }

        for screen_y in start_y.max(0)..end_y.min(framebuffer.height as i32) {
            for screen_x in start_x.max(0)..end_x.min(framebuffer.width as i32) {
                let buffer_index = screen_x as usize + screen_y as usize * framebuffer.width as usize;
                
                if buffer_index < z_buffer.len() && distance < z_buffer[buffer_index] {
                    let tex_x = if end_x > start_x {
                        (screen_x - start_x) as f32 / (end_x - start_x) as f32
                    } else { 0.5 };
                    
                    let tex_y = if end_y > start_y {
                        (screen_y - start_y) as f32 / (end_y - start_y) as f32
                    } else { 0.5 };
                    
                    let color = self.texture.get_color(tex_x, tex_y);
                    
                    if color[0] > 10 || color[1] > 10 || color[2] > 10 {
                        let mut final_color = Color::new(color[0], color[1], color[2], 255);
                        
                        if self.menacing_mode {
                            final_color.r = (final_color.r as u16 + 50).min(255) as u8;
                            final_color.g = (final_color.g as u16).max(20) as u8;
                            final_color.b = (final_color.b as u16).max(20) as u8;
                        }
                        
                        let distance_factor = (1.0 - (distance / 600.0)).max(0.4);
                        final_color.r = (final_color.r as f32 * distance_factor) as u8;
                        final_color.g = (final_color.g as f32 * distance_factor) as u8;
                        final_color.b = (final_color.b as f32 * distance_factor) as u8;
                        
                        framebuffer.set_current_color(final_color);
                        framebuffer.set_pixel(screen_x as u32, screen_y as u32);
                    }
                }
            }
        }
        
        if distance < 100.0 {
            self.render_menacing_aura(framebuffer, start_x, end_x, start_y, end_y, distance);
        }
    }
    
    fn render_menacing_aura(
        &self,
        framebuffer: &mut Framebuffer,
        start_x: i32,
        end_x: i32,
        start_y: i32,
        end_y: i32,
        distance: f32,
    ) {
        let aura_intensity = ((100.0 - distance) / 100.0 * 100.0) as u8;
        let aura_color = Color::new(255, 0, 0, aura_intensity.min(80));
        
        framebuffer.set_current_color(aura_color);
        
        let expand = 5;
        for y in (start_y - expand)..(end_y + expand) {
            for x in (start_x - expand)..(end_x + expand) {
                if x >= 0 && x < framebuffer.width as i32 && y >= 0 && y < framebuffer.height as i32 {
                    if (x < start_x || x >= end_x || y < start_y || y >= end_y) && 
                       (x + y) % 3 == 0 { 
                        framebuffer.set_pixel(x as u32, y as u32);
                    }
                }
            }
        }
    }
}