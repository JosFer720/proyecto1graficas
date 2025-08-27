use raylib::color::Color;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::texture::TextureManager;
use crate::sprites::SpriteManager;
use crate::taylor_sprite::TaylorSprite;

pub struct Intersect {
    pub distance: f32,
    pub impact: char,
    pub hit_x: f32,
    pub hit_y: f32,
    pub side: bool,
}

pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    a: f32,
    block_size: usize,
    draw_line: bool,
) -> Intersect {
    let mut d = 0.0;
    let step = 0.5;
    framebuffer.set_current_color(Color::WHITESMOKE);
    
    loop {
        let cos = d * a.cos();
        let sin = d * a.sin();
        let x = (player.pos.x + cos) as usize;
        let y = (player.pos.y + sin) as usize;
        let i = x / block_size;
        let j = y / block_size;
        
        if maze.is_empty() || j >= maze.len() {
            return Intersect {
                distance: d,
                impact: '+',
                hit_x: player.pos.x + cos,
                hit_y: player.pos.y + sin,
                side: false,
            };
        }
        
        if i >= maze.get(j).map_or(0, |row| row.len()) {
            return Intersect {
                distance: d,
                impact: '+',
                hit_x: player.pos.x + cos,
                hit_y: player.pos.y + sin,
                side: false,
            };
        }
        
        let cell = maze.get(j).and_then(|row| row.get(i)).unwrap_or(&'+');
        if *cell != ' ' {
            let hit_x = player.pos.x + cos;
            let hit_y = player.pos.y + sin;
            
            let cell_x = (hit_x / block_size as f32).fract();
            let cell_y = (hit_y / block_size as f32).fract();
            
            let side = cell_x.min(1.0 - cell_x) < cell_y.min(1.0 - cell_y);
            
            return Intersect {
                distance: d,
                impact: *cell,
                hit_x,
                hit_y,
                side,
            };
        }
        
        if draw_line {
            framebuffer.set_pixel(x as u32, y as u32);
        }
        
        d += step;
        
        if d > 2000.0 {
            return Intersect {
                distance: d,
                impact: '+',
                hit_x: player.pos.x + cos,
                hit_y: player.pos.y + sin,
                side: false,
            };
        }
    }
}

fn render_single_sprite_with_zbuffer_textured(
    framebuffer: &mut Framebuffer,
    sprite: &crate::sprites::Sprite,
    player: &Player,
    z_buffer: &[f32],
    distance: f32
) {
    if distance < 20.0 || distance > 800.0 { return; }
    
    let dx = sprite.x - player.pos.x;
    let dy = sprite.y - player.pos.y;
    
    let cos_angle = player.a.cos();
    let sin_angle = player.a.sin();
    
    let transformed_x = -dx * sin_angle + dy * cos_angle;
    let transformed_y = dx * cos_angle + dy * sin_angle;

    if transformed_y <= 10.0 { return; }
    
    let hh = framebuffer.height as f32 / 2.0;
    
    let actual_distance = (dx * dx + dy * dy).sqrt();
    let distance_to_projection_plane = 200.0;
    let sprite_height = (hh / actual_distance) * distance_to_projection_plane;
    let wall_height_factor = 0.5; 
    let adjusted_sprite_height = sprite_height * wall_height_factor;
    
    let fov = player.fov;
    let screen_center = framebuffer.width as f32 / 2.0;
    
    let sprite_angle = (transformed_x / transformed_y).atan();
    let angle_from_center = sprite_angle;
    
    let pixels_per_radian = framebuffer.width as f32 / fov;
    let screen_x = screen_center + (angle_from_center * pixels_per_radian);
    
    let sprite_width = adjusted_sprite_height * 0.8;
    let start_x = (screen_x - sprite_width / 2.0) as i32;
    let end_x = (screen_x + sprite_width / 2.0) as i32;
    let start_y = (hh - adjusted_sprite_height / 2.0).max(0.0) as i32;
    let end_y = (hh + adjusted_sprite_height / 2.0).min(framebuffer.height as f32) as i32;
    
    if start_x >= framebuffer.width as i32 || end_x < 0 ||
       start_y >= framebuffer.height as i32 || end_y < 0 { return; }
    
    let current_texture = &sprite.texture_frames[sprite.animation_frame];
    
    for screen_y in start_y.max(0)..end_y.min(framebuffer.height as i32) {
        for screen_x in start_x.max(0)..end_x.min(framebuffer.width as i32) {
            let buffer_index = screen_x as usize + screen_y as usize * framebuffer.width as usize;
            
            if buffer_index < z_buffer.len() && actual_distance < z_buffer[buffer_index] {
                let tex_x = if end_x > start_x {
                    (screen_x - start_x) as f32 / (end_x - start_x) as f32
                } else { 0.5 };
                
                let tex_y = if end_y > start_y {
                    (screen_y - start_y) as f32 / (end_y - start_y) as f32
                } else { 0.5 };
                
                let texture_color = current_texture.get_color(tex_x, tex_y);
                
                if texture_color[0] > 15 || texture_color[1] > 15 || texture_color[2] > 15 {
                    let mut final_color = Color::new(texture_color[0], texture_color[1], texture_color[2], 255);
                    
                    let distance_factor = (1.0 - (actual_distance / 600.0)).max(0.3);
                    final_color.r = (final_color.r as f32 * distance_factor) as u8;
                    final_color.g = (final_color.g as f32 * distance_factor) as u8;
                    final_color.b = (final_color.b as f32 * distance_factor) as u8;
                    
                    let glow_factor = 1.0 + (sprite.animation_timer * 6.28).sin() * 0.15;
                    final_color.r = (final_color.r as f32 * glow_factor).min(255.0) as u8;
                    final_color.g = (final_color.g as f32 * glow_factor).min(255.0) as u8;
                    final_color.b = (final_color.b as f32 * glow_factor).min(255.0) as u8;
                    
                    framebuffer.set_current_color(final_color);
                    framebuffer.set_pixel(screen_x as u32, screen_y as u32);
                }
            }
        }
    }
}

fn render_sprites_with_zbuffer_textured(
    framebuffer: &mut Framebuffer,
    player: &Player,
    _maze: &Maze,
    _block_size: usize,
    sprite_manager: &SpriteManager,
    z_buffer: &[f32],
) {
    let mut sprites_with_distance: Vec<(usize, f32)> = Vec::new();
    for (index, sprite) in sprite_manager.sprites.iter().enumerate() {
        if !sprite.collected {
            let dx = sprite.x - player.pos.x;
            let dy = sprite.y - player.pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            sprites_with_distance.push((index, distance));
        }
    }
    sprites_with_distance.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    for (sprite_index, distance) in sprites_with_distance {
        let sprite = &sprite_manager.sprites[sprite_index];
        render_single_sprite_with_zbuffer_textured(framebuffer, sprite, player, z_buffer, distance);
    }
}

pub fn render_world_with_textures_sprites_and_taylor_textured(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
    player: &Player,
    texture_manager: &TextureManager,
    sprite_manager: &SpriteManager,
    taylor_sprite: &TaylorSprite,
    taylor_position: raylib::prelude::Vector2,
) {
    let num_rays = framebuffer.width;
    let hh = framebuffer.height as f32 / 2.0;
    
    let mut z_buffer = vec![f32::INFINITY; (framebuffer.width * framebuffer.height) as usize];
    
    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            if y < hh as u32 {
                // Sky gradient
                let gradient_factor = y as f32 / hh;
                framebuffer.set_current_color(Color::new(
                    (20.0 + gradient_factor * 30.0) as u8,
                    (20.0 + gradient_factor * 50.0) as u8,
                    (40.0 + gradient_factor * 80.0) as u8,
                    255
                ));
            } else {
                let floor_pattern = ((x / 4) + (y / 4)) % 2;
                if floor_pattern == 0 {
                    framebuffer.set_current_color(Color::new(40, 20, 20, 255));
                } else {
                    framebuffer.set_current_color(Color::new(35, 18, 18, 255));
                }
            }
            framebuffer.set_pixel(x, y);
        }
    }
    
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        let wall_intersect = cast_ray(framebuffer, &maze, &player, a, block_size, false);
        
        if wall_intersect.impact == ' ' {
            continue;
        }
        
        let corrected_distance = wall_intersect.distance * (a - player.a).cos();
        
        let distance_to_projection_plane = 250.0;
        let stake_height = (hh / corrected_distance) * distance_to_projection_plane;
        let wall_height_factor = 0.65;
        let adjusted_stake_height = stake_height * wall_height_factor;
        let stake_top = (hh - (adjusted_stake_height / 2.0)).max(0.0) as usize;
        let stake_bottom = (hh + (adjusted_stake_height / 2.0)).min(framebuffer.height as f32) as usize;
        
        if stake_bottom > stake_top {
            let texture = texture_manager.get_texture(wall_intersect.impact);
            
            let tex_x = if wall_intersect.side {
                (wall_intersect.hit_y / block_size as f32).fract()
            } else {
                (wall_intersect.hit_x / block_size as f32).fract()
            };
            
            for y in stake_top..stake_bottom {
                let tex_y = if stake_bottom > stake_top {
                    (y - stake_top) as f32 / (stake_bottom - stake_top) as f32
                } else {
                    0.0
                };
                let color = texture.get_color(tex_x, tex_y);
                
                let final_color = if wall_intersect.side {
                    Color::new(
                        (color[0] as f32 * 0.7) as u8,
                        (color[1] as f32 * 0.7) as u8,
                        (color[2] as f32 * 0.7) as u8,
                        255
                    )
                } else {
                    Color::new(color[0], color[1], color[2], 255)
                };
                
                framebuffer.set_current_color(final_color);
                framebuffer.set_pixel(i as u32, y as u32);
                
                let buffer_index = i as usize + y * framebuffer.width as usize;
                if buffer_index < z_buffer.len() {
                    z_buffer[buffer_index] = corrected_distance;
                }
            }
        }
    }
    
    render_sprites_with_zbuffer_textured(framebuffer, player, maze, block_size, sprite_manager, &z_buffer);
    
    taylor_sprite.render_3d(framebuffer, taylor_position, player, &z_buffer, maze, block_size);
}

pub fn render_world_with_textures_sprites_and_taylor(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
    player: &Player,
    texture_manager: &TextureManager,
    sprite_manager: &SpriteManager,
    taylor_sprite: &TaylorSprite,
    taylor_position: raylib::prelude::Vector2,
) {
    render_world_with_textures_sprites_and_taylor_textured(
        framebuffer,
        maze,
        block_size,
        player,
        texture_manager,
        sprite_manager,
        taylor_sprite,
        taylor_position,
    );
}