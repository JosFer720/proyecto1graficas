// src/caster.rs - Sistema de raycasting con Taylor Swift
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
        
        if j >= maze.len() || i >= maze[0].len() {
            return Intersect {
                distance: d,
                impact: '+',
                hit_x: player.pos.x + cos,
                hit_y: player.pos.y + sin,
                side: false,
            };
        }
        
        if maze[j][i] != ' ' {
            let hit_x = player.pos.x + cos;
            let hit_y = player.pos.y + sin;
            
            let cell_x = (hit_x / block_size as f32).fract();
            let cell_y = (hit_y / block_size as f32).fract();
            
            let side = cell_x.min(1.0 - cell_x) < cell_y.min(1.0 - cell_y);
            
            return Intersect {
                distance: d,
                impact: maze[j][i],
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
    let num_rays = framebuffer.width;
    let hh = framebuffer.height as f32 / 2.0;
    
    // Crear z-buffer para toda la pantalla
    let mut z_buffer = vec![f32::INFINITY; (framebuffer.width * framebuffer.height) as usize];
    
    // --- CIELO Y SUELO ---
    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            if y < hh as u32 {
                framebuffer.set_current_color(Color::new(20, 20, 40, 255)); // Cielo más oscuro/dramático
            } else {
                framebuffer.set_current_color(Color::new(40, 20, 20, 255)); // Suelo rojizo/amenazante
            }
            framebuffer.set_pixel(x, y);
        }
    }
    
    // PASO 1: Renderizar todas las paredes y actualizar z-buffer
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
            // Renderizar pared
            let texture = texture_manager.get_texture(wall_intersect.impact);
            
            let tex_x = if wall_intersect.side {
                (wall_intersect.hit_y / block_size as f32).fract()
            } else {
                (wall_intersect.hit_x / block_size as f32).fract()
            };
            
            for y in stake_top..stake_bottom {
                let tex_y = (y - stake_top) as f32 / (stake_bottom - stake_top) as f32;
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
                
                // Actualizar z-buffer
                let buffer_index = i as usize + y * framebuffer.width as usize;
                if buffer_index < z_buffer.len() {
                    z_buffer[buffer_index] = corrected_distance;
                }
            }
        }
    }
    
    // PASO 2: Renderizar bidones de gasolina usando el z-buffer
    render_sprites_with_zbuffer(framebuffer, player, maze, block_size, sprite_manager, &z_buffer);
    
    // PASO 3: Renderizar a Taylor Swift (¡El momento más dramático!)
    taylor_sprite.render_3d(framebuffer, taylor_position, player, &z_buffer);
}

// Función auxiliar para renderizar sprites con z-buffer
fn render_sprites_with_zbuffer(
    framebuffer: &mut Framebuffer,
    player: &Player,
    _maze: &Maze,
    _block_size: usize,
    sprite_manager: &SpriteManager,
    z_buffer: &[f32],
) {
    // Ordenar sprites por distancia (más lejanos primero)
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
        render_single_sprite_with_zbuffer(framebuffer, sprite, player, z_buffer, distance);
    }
}

// Función para renderizar un solo sprite con z-buffer
fn render_single_sprite_with_zbuffer(
    framebuffer: &mut Framebuffer,
    sprite: &crate::sprites::Sprite,
    player: &Player,
    z_buffer: &[f32],
    distance: f32
) {
    if distance < 20.0 || distance > 800.0 { return; }
    
    let dx = sprite.x - player.pos.x;
    let dy = sprite.y - player.pos.y;
    
    // Transformación correcta a coordenadas de cámara
    let cos_angle = player.a.cos();
    let sin_angle = player.a.sin();
    
    let transformed_x = -dx * sin_angle + dy * cos_angle;
    let transformed_y = dx * cos_angle + dy * sin_angle;

    // Si el sprite está detrás del jugador, no renderizar
    if transformed_y <= 10.0 { return; }
    
    let hh = framebuffer.height as f32 / 2.0;
    
    // Usar la distancia real para el scaling
    let actual_distance = (dx * dx + dy * dy).sqrt();
    let distance_to_projection_plane = 200.0;
    let sprite_height = (hh / actual_distance) * distance_to_projection_plane;
    let wall_height_factor = 0.4;
    let adjusted_sprite_height = sprite_height * wall_height_factor * 0.6;
    
    // Calcular posición horizontal en pantalla
    let fov = player.fov;
    let screen_center = framebuffer.width as f32 / 2.0;
    
    // Ángulo relativo del sprite respecto al centro de la vista
    let sprite_angle = (transformed_x / transformed_y).atan();
    let angle_from_center = sprite_angle;
    
    // Convertir ángulo a posición en pantalla
    let pixels_per_radian = framebuffer.width as f32 / fov;
    let screen_x = screen_center + (angle_from_center * pixels_per_radian);
    
    let sprite_width = adjusted_sprite_height * 0.8;
    let start_x = (screen_x - sprite_width / 2.0) as i32;
    let end_x = (screen_x + sprite_width / 2.0) as i32;
    let start_y = (hh - adjusted_sprite_height / 2.0).max(0.0) as i32;
    let end_y = (hh + adjusted_sprite_height / 2.0).min(framebuffer.height as f32) as i32;
    
    // Verificar que el sprite esté en pantalla
    if start_x >= framebuffer.width as i32 || end_x < 0 ||
       start_y >= framebuffer.height as i32 || end_y < 0 { return; }
    
    let texture = &sprite.texture_frames[sprite.animation_frame];
    
    for screen_y in start_y.max(0)..end_y.min(framebuffer.height as i32) {
        for screen_x in start_x.max(0)..end_x.min(framebuffer.width as i32) {
            let buffer_index = screen_x as usize + screen_y as usize * framebuffer.width as usize;
            
            // Solo renderizar si el sprite está más cerca que la pared en esa posición
            if buffer_index < z_buffer.len() && actual_distance < z_buffer[buffer_index] {
                let tex_x = if end_x > start_x {
                    ((screen_x - start_x) * 32 / (end_x - start_x)).clamp(0, 31) as usize
                } else { 16 };
                
                let tex_y = if end_y > start_y {
                    ((screen_y - start_y) * 32 / (end_y - start_y)).clamp(0, 31) as usize
                } else { 16 };
                
                let pixel_index = tex_y * 32 + tex_x;
                
                if pixel_index < texture.len() {
                    let pixel = texture[pixel_index];
                    if pixel.a > 128 { // Solo píxeles suficientemente opacos
                        framebuffer.set_current_color(pixel);
                        framebuffer.set_pixel(screen_x as u32, screen_y as u32);
                    }
                }
            }
        }
    }
}