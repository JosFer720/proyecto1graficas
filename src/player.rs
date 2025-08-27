use raylib::prelude::*;
use std::f32::consts::PI;
use crate::maze::Maze;

pub struct Player {
    pub pos: Vector2,
    pub a: f32,  
    pub fov: f32,
}

fn is_valid_position(maze: &Maze, x: f32, y: f32, block_size: usize) -> bool {
    let grid_x = (x / block_size as f32) as usize;
    let grid_y = (y / block_size as f32) as usize;
    
    if grid_y >= maze.len() || grid_x >= maze[0].len() {
        return false;
    }
    
    maze[grid_y][grid_x] == ' '
}

pub fn process_events(player: &mut Player, rl: &RaylibHandle, maze: &Maze, block_size: usize) {
    const MOVE_SPEED: f32 = 10.0;
    const ROTATION_SPEED: f32 = PI / 12.0;
    const GAMEPAD_SENSITIVITY: f32 = 0.8;
    
    if rl.is_key_down(KeyboardKey::KEY_LEFT) {
        player.a += ROTATION_SPEED;
    }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        player.a -= ROTATION_SPEED;
    }
    
    if rl.is_gamepad_available(0) {
        let right_stick_x = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_RIGHT_X);
        if right_stick_x.abs() > 0.2 {
            player.a -= right_stick_x * ROTATION_SPEED * GAMEPAD_SENSITIVITY;
        }
    }
    
    if player.a < 0.0 {
        player.a += 2.0 * PI;
    } else if player.a > 2.0 * PI {
        player.a -= 2.0 * PI;
    }
    
    let mut new_x = player.pos.x;
    let mut new_y = player.pos.y;
    
    let mut gamepad_move_forward = 0.0;
    let mut gamepad_move_strafe = 0.0;
    
    if rl.is_gamepad_available(0) {
        let left_stick_y = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_Y);
        let left_stick_x = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_X);
        
        if left_stick_y.abs() > 0.2 {
            gamepad_move_forward = -left_stick_y; 
        }
        if left_stick_x.abs() > 0.2 {
            gamepad_move_strafe = left_stick_x;
        }
    }
    
    if rl.is_key_down(KeyboardKey::KEY_UP) || rl.is_key_down(KeyboardKey::KEY_W) || gamepad_move_forward > 0.0 {
        let move_amount = if gamepad_move_forward > 0.0 { 
            MOVE_SPEED * gamepad_move_forward 
        } else { 
            MOVE_SPEED 
        };
        new_x += move_amount * player.a.cos();
        new_y += move_amount * player.a.sin();
    }
    
    if rl.is_key_down(KeyboardKey::KEY_DOWN) || rl.is_key_down(KeyboardKey::KEY_S) || gamepad_move_forward < 0.0 {
        let move_amount = if gamepad_move_forward < 0.0 { 
            MOVE_SPEED * (-gamepad_move_forward) 
        } else { 
            MOVE_SPEED 
        };
        new_x -= move_amount * player.a.cos();
        new_y -= move_amount * player.a.sin();
    }
    
    if rl.is_key_down(KeyboardKey::KEY_A) || gamepad_move_strafe < 0.0 {
        let move_amount = if gamepad_move_strafe < 0.0 { 
            MOVE_SPEED * (-gamepad_move_strafe) 
        } else { 
            MOVE_SPEED 
        };
        new_x += move_amount * (player.a - PI/2.0).cos();
        new_y += move_amount * (player.a - PI/2.0).sin();
    }
    
    if rl.is_key_down(KeyboardKey::KEY_D) || gamepad_move_strafe > 0.0 {
        let move_amount = if gamepad_move_strafe > 0.0 { 
            MOVE_SPEED * gamepad_move_strafe 
        } else { 
            MOVE_SPEED 
        };
        new_x += move_amount * (player.a + PI/2.0).cos();
        new_y += move_amount * (player.a + PI/2.0).sin();
    }
    
    if is_valid_position(maze, new_x, player.pos.y, block_size) {
        player.pos.x = new_x;
    }
    if is_valid_position(maze, player.pos.x, new_y, block_size) {
        player.pos.y = new_y;
    }
}