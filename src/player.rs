// src/player.rs - Sistema de jugador para Tom Hiddleston
use raylib::prelude::*;
use std::f32::consts::PI;
use crate::maze::Maze;

pub struct Player {
    pub pos: Vector2,
    pub a: f32,   // ángulo de rotación
    pub fov: f32, // field of view
}

// Función para verificar si una posición es válida (no es una pared)
fn is_valid_position(maze: &Maze, x: f32, y: f32, block_size: usize) -> bool {
    let grid_x = (x / block_size as f32) as usize;
    let grid_y = (y / block_size as f32) as usize;
    
    // Verificar límites del mapa
    if grid_y >= maze.len() || grid_x >= maze[0].len() {
        return false;
    }
    
    // Solo permitir movimiento en espacios vacíos
    maze[grid_y][grid_x] == ' '
}

pub fn process_events(player: &mut Player, rl: &RaylibHandle, maze: &Maze, block_size: usize) {
    const MOVE_SPEED: f32 = 10.0; // Aumentado para escapar mejor de Taylor
    const ROTATION_SPEED: f32 = PI / 12.0;
    
    // Rotación con teclas de flecha
    if rl.is_key_down(KeyboardKey::KEY_LEFT) {
        player.a += ROTATION_SPEED;
    }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        player.a -= ROTATION_SPEED;
    }
    
    // Normalizar el ángulo
    if player.a < 0.0 {
        player.a += 2.0 * PI;
    } else if player.a > 2.0 * PI {
        player.a -= 2.0 * PI;
    }
    
    // Movimiento con detección de colisiones
    let mut new_x = player.pos.x;
    let mut new_y = player.pos.y;
    
    // Movimiento hacia adelante (W o flecha arriba)
    if rl.is_key_down(KeyboardKey::KEY_UP) || rl.is_key_down(KeyboardKey::KEY_W) {
        new_x += MOVE_SPEED * player.a.cos();
        new_y += MOVE_SPEED * player.a.sin();
    }
    
    // Movimiento hacia atrás (S o flecha abajo)
    if rl.is_key_down(KeyboardKey::KEY_DOWN) || rl.is_key_down(KeyboardKey::KEY_S) {
        new_x -= MOVE_SPEED * player.a.cos();
        new_y -= MOVE_SPEED * player.a.sin();
    }
    
    // Movimiento lateral izquierda (A - strafe)
    if rl.is_key_down(KeyboardKey::KEY_A) {
        new_x += MOVE_SPEED * (player.a - PI/2.0).cos();
        new_y += MOVE_SPEED * (player.a - PI/2.0).sin();
    }
    
    // Movimiento lateral derecha (D - strafe)
    if rl.is_key_down(KeyboardKey::KEY_D) {
        new_x += MOVE_SPEED * (player.a + PI/2.0).cos();
        new_y += MOVE_SPEED * (player.a + PI/2.0).sin();
    }
    
    // Aplicar movimiento solo si la nueva posición es válida
    // Verificar X e Y por separado para permitir "deslizamiento" en las paredes
    if is_valid_position(maze, new_x, player.pos.y, block_size) {
        player.pos.x = new_x;
    }
    if is_valid_position(maze, player.pos.x, new_y, block_size) {
        player.pos.y = new_y;
    }
}