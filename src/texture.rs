// src/texture.rs - Sistema de texturas para el juego de escape
pub struct ImageTexture {
    pub data: Vec<Vec<[u8; 3]>>,
    pub width: usize,
    pub height: usize,
}

impl ImageTexture {
    pub fn from_file(file_path: &str) -> Self {
        match image::open(file_path) {
            Ok(img) => {
                let img = img.to_rgb8();
                let (width, height) = img.dimensions();
                let mut data = vec![vec![[0, 0, 0]; width as usize]; height as usize];
                
                for y in 0..height {
                    for x in 0..width {
                        let pixel = img.get_pixel(x, y);
                        data[y as usize][x as usize] = [pixel[0], pixel[1], pixel[2]];
                    }
                }
                
                println!("Textura cargada exitosamente: {} ({}x{})", file_path, width, height);
                Self { data, width: width as usize, height: height as usize }
            },
            Err(e) => {
                println!("Error cargando {}: {}", file_path, e);
                // Textura de fallback con patrón de Hollywood
                let size = 64;
                let mut data = vec![vec![[0, 0, 0]; size]; size];
                
                for y in 0..size {
                    for x in 0..size {
                        if (x/8 + y/8) % 2 == 0 {
                            // Patrón de damero dorado/negro (Hollywood glamour)
                            data[y][x] = [255, 215, 0]; // Dorado
                        } else {
                            data[y][x] = [20, 20, 20]; // Negro
                        }
                    }
                }
                
                println!("Usando textura de fallback para: {}", file_path);
                Self { data, width: size, height: size }
            }
        }
    }
    
    pub fn get_color(&self, tex_x: f32, tex_y: f32) -> [u8; 3] {
        // Asegurar que las coordenadas están en el rango [0, 1]
        let tex_x = tex_x.clamp(0.0, 1.0);
        let tex_y = tex_y.clamp(0.0, 1.0);
        
        let x = ((tex_x * (self.width - 1) as f32) as usize).min(self.width - 1);
        let y = ((tex_y * (self.height - 1) as f32) as usize).min(self.height - 1);
        
        self.data[y][x]
    }
}

pub struct TextureManager {
    pub wall_texture: ImageTexture,
    pub floor_texture: ImageTexture,
    pub ceiling_texture: ImageTexture,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            wall_texture: ImageTexture::from_file("assets/wall.png"),
            floor_texture: ImageTexture::from_file("assets/floor.png"),
            ceiling_texture: ImageTexture::from_file("assets/ceiling.png"),
        }
    }
    
    pub fn get_texture(&self, wall_char: char) -> &ImageTexture {
        // Usar diferentes texturas según el tipo de pared si se desea
        match wall_char {
            '+' | '-' | '|' => &self.wall_texture,
            '1' => &self.wall_texture, // Pared tipo 1
            '2' => &self.wall_texture, // Pared tipo 2
            '3' => &self.wall_texture, // Pared tipo 3
            _ => &self.wall_texture,
        }
    }
}