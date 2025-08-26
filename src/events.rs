use raylib::prelude::*;
use crate::player::Player;

pub struct EventHandler {
    last_mouse_x: f32,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            last_mouse_x: 512.0,
        }
    }
    
    pub fn handle_input(&mut self, rl: &mut RaylibHandle, player: &mut Player, level: &Vec<Vec<char>>) {
        let delta = rl.get_frame_time();
        
        if rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP) {
            player.move_forward(delta, level);
        }
        if rl.is_key_down(KeyboardKey::KEY_S) || rl.is_key_down(KeyboardKey::KEY_DOWN) {
            player.move_backward(delta, level);
        }
        if rl.is_key_down(KeyboardKey::KEY_A) || rl.is_key_down(KeyboardKey::KEY_LEFT) {
            player.rotate(-delta * 2.0);
        }
        if rl.is_key_down(KeyboardKey::KEY_D) || rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            player.rotate(delta * 2.0);
        }
        
        let mouse_x = rl.get_mouse_x() as f32;
        let mouse_delta = mouse_x - self.last_mouse_x;
        
        if mouse_delta.abs() > 0.1 {
            player.rotate(mouse_delta * 0.001);
            self.last_mouse_x = mouse_x;
        }
        
        if mouse_x < 100.0 || mouse_x > 924.0 {
            rl.set_mouse_position(Vector2::new(512.0, 384.0));
            self.last_mouse_x = 512.0;
        }
    }
}