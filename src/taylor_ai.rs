use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;
use raylib::prelude::Vector2;
use crate::maze::Maze;
use crate::player::Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    
    pub fn from_world_pos(world_x: f32, world_y: f32, block_size: usize) -> Self {
        Self {
            x: (world_x / block_size as f32) as i32,
            y: (world_y / block_size as f32) as i32,
        }
    }
    
    pub fn to_world_pos(&self, block_size: usize) -> Vector2 {
        Vector2::new(
            (self.x * block_size as i32) as f32 + (block_size as f32 / 2.0),
            (self.y * block_size as i32) as f32 + (block_size as f32 / 2.0),
        )
    }
    
    pub fn manhattan_distance(&self, other: &GridPos) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
    
    pub fn get_neighbors(&self) -> Vec<GridPos> {
        vec![
            GridPos::new(self.x + 1, self.y),  
            GridPos::new(self.x - 1, self.y),     
            GridPos::new(self.x, self.y + 1),    
            GridPos::new(self.x, self.y - 1),     
            GridPos::new(self.x + 1, self.y + 1), 
            GridPos::new(self.x - 1, self.y + 1),
            GridPos::new(self.x + 1, self.y - 1), 
            GridPos::new(self.x - 1, self.y - 1), 
        ]
    }
}

#[derive(Debug, Clone)]
pub struct PathNode {
    pub pos: GridPos,
    pub g_cost: i32, 
    pub h_cost: i32, 
    pub parent: Option<GridPos>,
}

impl PathNode {
    pub fn f_cost(&self) -> i32 {
        self.g_cost + self.h_cost
    }
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl Eq for PathNode {}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_cost().cmp(&self.f_cost())
            .then_with(|| other.h_cost.cmp(&self.h_cost))
    }
}

pub struct TaylorAI {
    pub current_path: Vec<GridPos>,
    pub path_index: usize,
    pub last_pathfind_time: f32,
    pub pathfind_frequency: f32,
    pub stuck_timer: f32,
    pub last_position: Vector2,
}

impl TaylorAI {
    pub fn new() -> Self {
        Self {
            current_path: Vec::new(),
            path_index: 0,
            last_pathfind_time: 0.0,
            pathfind_frequency: 1.0, 
            stuck_timer: 0.0,
            last_position: Vector2::new(0.0, 0.0),
        }
    }
    
    pub fn is_walkable(&self, pos: &GridPos, maze: &Maze) -> bool {
        if pos.y < 0 || pos.y >= maze.len() as i32 {
            return false;
        }
        
        let row = &maze[pos.y as usize];
        if pos.x < 0 || pos.x >= row.len() as i32 {
            return false;
        }
        
        row[pos.x as usize] == ' '
    }
    
    pub fn find_path_astar(&self, start: GridPos, goal: GridPos, maze: &Maze) -> Vec<GridPos> {
        if !self.is_walkable(&start, maze) || !self.is_walkable(&goal, maze) {
            return Vec::new();
        }
        
        let mut open_set = BinaryHeap::new();
        let mut closed_set = HashSet::new();
        let mut came_from: HashMap<GridPos, GridPos> = HashMap::new();
        let mut g_scores: HashMap<GridPos, i32> = HashMap::new();
        
        g_scores.insert(start, 0);
        open_set.push(PathNode {
            pos: start,
            g_cost: 0,
            h_cost: start.manhattan_distance(&goal),
            parent: None,
        });
        
        while let Some(current_node) = open_set.pop() {
            let current_pos = current_node.pos;
            
            if current_pos == goal {
                return self.reconstruct_path(&came_from, current_pos);
            }
            
            closed_set.insert(current_pos);
            
            for neighbor_pos in current_pos.get_neighbors() {
                if !self.is_walkable(&neighbor_pos, maze) || closed_set.contains(&neighbor_pos) {
                    continue;
                }
                
                let is_diagonal = (neighbor_pos.x - current_pos.x).abs() == 1_i32 && 
                                (neighbor_pos.y - current_pos.y).abs() == 1_i32;
                let move_cost = if is_diagonal { 14 } else { 10 }; 
                
                let tentative_g_score = current_node.g_cost + move_cost;
                
                if let Some(&existing_g_score) = g_scores.get(&neighbor_pos) {
                    if tentative_g_score >= existing_g_score {
                        continue;
                    }
                }
                
                came_from.insert(neighbor_pos, current_pos);
                g_scores.insert(neighbor_pos, tentative_g_score);
                
                open_set.push(PathNode {
                    pos: neighbor_pos,
                    g_cost: tentative_g_score,
                    h_cost: neighbor_pos.manhattan_distance(&goal),
                    parent: Some(current_pos),
                });
            }
        }
        
        Vec::new()
    }
    
    fn reconstruct_path(&self, came_from: &HashMap<GridPos, GridPos>, mut current: GridPos) -> Vec<GridPos> {
        let mut path = vec![current];
        
        while let Some(&parent) = came_from.get(&current) {
            current = parent;
            path.push(current);
        }
        
        path.reverse();
        path.remove(0); 
        path
    }
    
    pub fn update_ai(
        &mut self,
        taylor_position: &mut Vector2,
        player: &Player,
        maze: &Maze,
        block_size: usize,
        delta_time: f32,
        taylor_speed: f32,
    ) {
        self.last_pathfind_time += delta_time;
        self.stuck_timer += delta_time;
        
        let distance_moved = (taylor_position.x - self.last_position.x).powi(2) + 
                           (taylor_position.y - self.last_position.y).powi(2);
        if distance_moved < 0.5 {
            self.stuck_timer += delta_time;
        } else {
            self.stuck_timer = 0.0;
            self.last_position = *taylor_position;
        }
        
        let should_recalculate = self.last_pathfind_time >= self.pathfind_frequency ||
                                self.current_path.is_empty() ||
                                self.stuck_timer > 2.0;
        
        if should_recalculate {
            let taylor_grid_pos = GridPos::from_world_pos(taylor_position.x, taylor_position.y, block_size);
            let player_grid_pos = GridPos::from_world_pos(player.pos.x, player.pos.y, block_size);
            
            self.current_path = self.find_path_astar(taylor_grid_pos, player_grid_pos, maze);
            self.path_index = 0;
            self.last_pathfind_time = 0.0;
            self.stuck_timer = 0.0;
            
            if self.current_path.is_empty() {
                self.find_approximate_path(taylor_grid_pos, player_grid_pos, maze);
            }
        }
        
        if !self.current_path.is_empty() && self.path_index < self.current_path.len() {
            let target_grid_pos = self.current_path[self.path_index];
            let target_world_pos = target_grid_pos.to_world_pos(block_size);
            
            let dx = target_world_pos.x - taylor_position.x;
            let dy = target_world_pos.y - taylor_position.y;
            let distance_to_target = (dx * dx + dy * dy).sqrt();
            
            if distance_to_target < block_size as f32 * 0.3 {
                self.path_index += 1;
            } else {
                let move_distance = taylor_speed * delta_time * 60.0;
                let dir_x = dx / distance_to_target;
                let dir_y = dy / distance_to_target;
                
                let new_x = taylor_position.x + dir_x * move_distance;
                let new_y = taylor_position.y + dir_y * move_distance;
                
                if self.is_position_valid(new_x, new_y, maze, block_size) {
                    taylor_position.x = new_x;
                    taylor_position.y = new_y;
                } else {
                    self.try_alternative_movement(taylor_position, dir_x, dir_y, move_distance, maze, block_size);
                }
            }
        } else {
            self.direct_movement_fallback(taylor_position, player, taylor_speed, delta_time, maze, block_size);
        }
    }
    
    fn find_approximate_path(&mut self, start: GridPos, goal: GridPos, maze: &Maze) {
        let mut best_distance = i32::MAX;
        let mut best_target = goal;
        
        for radius in 1i32..=5 {
            for dx in -radius..=radius {
                for dy in -radius..=radius {
                    if dx.abs() != radius && dy.abs() != radius {
                        continue;
            }
                    
                    let candidate = GridPos::new(goal.x + dx, goal.y + dy);
                    if self.is_walkable(&candidate, maze) {
                        let distance = start.manhattan_distance(&candidate);
                        if distance < best_distance {
                            best_distance = distance;
                            best_target = candidate;
                        }
                    }
                }
            }
            
            if best_target != goal {
                self.current_path = self.find_path_astar(start, best_target, maze);
                if !self.current_path.is_empty() {
                    break;
                }
            }
        }
    }
    
    fn is_position_valid(&self, x: f32, y: f32, maze: &Maze, block_size: usize) -> bool {
        const TAYLOR_RADIUS: f32 = 15.0;
        
        let check_positions = [
            (x, y),
            (x + TAYLOR_RADIUS, y),
            (x - TAYLOR_RADIUS, y),
            (x, y + TAYLOR_RADIUS),
            (x, y - TAYLOR_RADIUS),
        ];
        
        for &(check_x, check_y) in &check_positions {
            let grid_pos = GridPos::from_world_pos(check_x, check_y, block_size);
            if !self.is_walkable(&grid_pos, maze) {
                return false;
            }
        }
        
        true
    }
    
    fn try_alternative_movement(
        &self,
        taylor_position: &mut Vector2,
        desired_dir_x: f32,
        desired_dir_y: f32,
        move_distance: f32,
        maze: &Maze,
        block_size: usize,
    ) {
        let alternatives = [
            (desired_dir_y, -desired_dir_x),
            (-desired_dir_y, desired_dir_x), 
            (desired_dir_x * 0.7 + desired_dir_y * 0.7, desired_dir_y * 0.7 - desired_dir_x * 0.7),
            (desired_dir_x * 0.7 - desired_dir_y * 0.7, desired_dir_y * 0.7 + desired_dir_x * 0.7), 
        ];
        
        for &(alt_x, alt_y) in &alternatives {
            let new_x = taylor_position.x + alt_x * move_distance * 0.5;
            let new_y = taylor_position.y + alt_y * move_distance * 0.5;
            
            if self.is_position_valid(new_x, new_y, maze, block_size) {
                taylor_position.x = new_x;
                taylor_position.y = new_y;
                break;
            }
        }
    }
    
    fn direct_movement_fallback(
        &self,
        taylor_position: &mut Vector2,
        player: &Player,
        taylor_speed: f32,
        delta_time: f32,
        maze: &Maze,
        block_size: usize,
    ) {
        let dx = player.pos.x - taylor_position.x;
        let dy = player.pos.y - taylor_position.y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        if distance > 10.0 {
            let move_distance = taylor_speed * delta_time * 60.0;
            let dir_x = dx / distance;
            let dir_y = dy / distance;
            
            let new_x = taylor_position.x + dir_x * move_distance;
            let new_y = taylor_position.y + dir_y * move_distance;
            
            if self.is_position_valid(new_x, new_y, maze, block_size) {
                taylor_position.x = new_x;
                taylor_position.y = new_y;
            }
        }
    }
}