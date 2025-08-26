// src/maze.rs
use std::fs::File;
use std::io::{BufRead, BufReader};

pub type Maze = Vec<Vec<char>>;

#[derive(Debug, Clone)]
pub struct SpritePosition {
    pub x: f32,
    pub y: f32,
}

pub fn load_maze(filename: &str) -> Maze {
    match File::open(filename) {
        Ok(file) => {
            let reader = BufReader::new(file);
            reader
                .lines()
                .map(|line| line.unwrap().chars().collect())
                .collect()
        }
        Err(e) => {
            println!("Error cargando {}: {}", filename, e);
            println!("Usando maze de fallback...");
            // Maze de fallback
            vec![
                "+++++++++++++++++".chars().collect(),
                "+  .    +     . +".chars().collect(),
                "+   +   +   +   +".chars().collect(),
                "+       +       +".chars().collect(),
                "+   .   +   .   +".chars().collect(),
                "+++++++++++++++++".chars().collect(),
            ]
        }
    }
}

// Función para extraer posiciones de sprites desde el maze
pub fn extract_sprite_positions(maze: &Maze, block_size: usize) -> Vec<SpritePosition> {
    let mut sprite_positions = Vec::new();
    
    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            if cell == '.' {
                // Calcular la posición central del bloque
                let x = (col_index * block_size) as f32 + (block_size as f32 / 2.0);
                let y = (row_index * block_size) as f32 + (block_size as f32 / 2.0);
                
                sprite_positions.push(SpritePosition { x, y });
                println!("Sprite encontrado en posición: ({}, {}) - grid({}, {})", x, y, col_index, row_index);
            }
        }
    }
    
    println!("Total de sprites encontrados: {}", sprite_positions.len());
    sprite_positions
}

// Función para limpiar el maze (convertir '.' a ' ' después de extraer posiciones)
pub fn clean_maze(maze: &mut Maze) {
    for row in maze.iter_mut() {
        for cell in row.iter_mut() {
            if *cell == '.' {
                *cell = ' '; // Convertir a espacio vacío
            }
        }
    }
}