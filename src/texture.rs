use image::GenericImageView;

#[derive(Clone, Debug)]
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
                println!("Error cargando {}: {}, generando textura de fallback...", file_path, e);
                
                if file_path.contains("gasoline_can") {
                    Self::generate_gasoline_can_texture(file_path)
                } else if file_path.contains("taylor") {
                    Self::generate_taylor_fallback()
                } else if file_path.contains("exit") {
                    Self::generate_exit_texture()
                } else {
                    Self::generate_wall_texture()
                }
            }
        }
    }
    
    fn generate_gasoline_can_texture(file_path: &str) -> Self {
        let size = 64;
        let mut data = vec![vec![[0, 0, 0]; size]; size];
        
        let (base_color, effect_color, glow_intensity) = if file_path.contains("_1") {
            ([200, 50, 50], [140, 140, 140], 0.0) 
        } else if file_path.contains("_2") {
            ([220, 70, 70], [160, 160, 160], 0.3)
        } else {
            ([255, 100, 50], [200, 180, 100], 0.6) 
        };
        
        for y in 0..size {
            for x in 0..size {
                let center_x = size as f32 / 2.0;
                let center_y = size as f32 * 0.6; 
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                
                if dx.abs() <= 20.0 && dy >= -25.0 && dy <= 15.0 {
                    if y <= size / 4 {
                        let mut cap_color = effect_color;
                        if glow_intensity > 0.0 {
                            cap_color[0] = (cap_color[0] as f32 * (1.0 + glow_intensity)).min(255.0) as u8;
                            cap_color[1] = (cap_color[1] as f32 * (1.0 + glow_intensity)).min(255.0) as u8;
                            cap_color[2] = (cap_color[2] as f32 * (1.0 + glow_intensity * 0.5)).min(255.0) as u8;
                        }
                        data[y][x] = cap_color;
                    } else {
                        let mut body_color = base_color;
                        if glow_intensity > 0.0 {
                            body_color[0] = (body_color[0] as f32 * (1.0 + glow_intensity * 0.3)).min(255.0) as u8;
                            body_color[1] = (body_color[1] as f32 * (1.0 + glow_intensity * 0.8)).min(255.0) as u8;
                            body_color[2] = (body_color[2] as f32 * (1.0 + glow_intensity)).min(255.0) as u8;
                        }
                        data[y][x] = body_color;
                    }
                }
                
                if x >= 18 && x <= 25 && y >= 8 && y <= 15 {
                    data[y][x] = [120, 120, 120];
                }
                
                if x >= 50 && x <= 55 && y >= 25 && y <= 40 {
                    let mut handle_color = [100, 100, 100];
                    if glow_intensity > 0.0 {
                        handle_color[0] = (handle_color[0] as f32 * (1.0 + glow_intensity * 0.5)).min(255.0) as u8;
                        handle_color[1] = (handle_color[1] as f32 * (1.0 + glow_intensity * 0.5)).min(255.0) as u8;
                        handle_color[2] = (handle_color[2] as f32 * (1.0 + glow_intensity * 0.2)).min(255.0) as u8;
                    }
                    data[y][x] = handle_color;
                }
                
                if dx.abs() <= 12.0 && dy >= -5.0 && dy <= 8.0 {
                    let mut label_color = [255, 255, 255];
                    if glow_intensity > 0.4 {
                        label_color = [255, 215, 0]; 
                    }
                    data[y][x] = label_color;
                }
                
                if dx.abs() <= 20.0 && dy >= -25.0 && dy <= 15.0 {
                    if dx >= -20.0 && dx <= -15.0 {
                        let current = data[y][x];
                        data[y][x] = [
                            (current[0] as u16 + 30).min(255) as u8,
                            (current[1] as u16 + 30).min(255) as u8,
                            (current[2] as u16 + 30).min(255) as u8,
                        ];
                    }
                    if dx >= 15.0 && dx <= 20.0 {
                        let current = data[y][x];
                        data[y][x] = [
                            (current[0] as f32 * 0.7) as u8,
                            (current[1] as f32 * 0.7) as u8,
                            (current[2] as f32 * 0.7) as u8,
                        ];
                    }
                }
            }
        }
        
        println!("Textura de bidón generada: {}", file_path);
        Self { data, width: size, height: size }
    }
    
    fn generate_taylor_fallback() -> Self {
        let size = 64;
        let mut data = vec![vec![[0, 0, 0]; size]; size];
        
        for y in 0..size {
            for x in 0..size {
                let center_x = size as f32 / 2.0;
                let center_y = size as f32 / 2.0;
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                
                if dy >= -25.0 && dy <= -15.0 && dx.abs() <= 8.0 {
                    data[y][x] = [255, 220, 177];
                }
                

                if dy >= -15.0 && dy <= 10.0 && dx.abs() <= 12.0 {
                    if y % 3 == 0 { 
                        data[y][x] = [200, 0, 100];
                    } else {
                        data[y][x] = [150, 0, 75]; 
                    }
                }
                
                if dy >= 10.0 && dy <= 25.0 {
                    if (dx >= -8.0 && dx <= -2.0) || (dx >= 2.0 && dx <= 8.0) {
                        data[y][x] = [50, 50, 150];
                    }
                }
            }
        }
        
        println!("Textura de Taylor Swift generada (fallback)");
        Self { data, width: size, height: size }
    }
    
    fn generate_exit_texture() -> Self {
        let size = 64;
        let mut data = vec![vec![[0, 0, 0]; size]; size];
        
        for y in 0..size {
            for x in 0..size {
                let center_x = size as f32 / 2.0;
                let center_y = size as f32 / 2.0;
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                
                if x <= 2 || x >= size - 3 || y <= 2 || y >= size - 3 {
                    data[y][x] = [100, 60, 30]; 
                }
                else if (x >= 6 && x <= 28 && y >= 6 && y <= 25) || 
                        (x >= 35 && x <= 57 && y >= 6 && y <= 25) {
                    let grain = ((x + y * 3) % 8) as f32 / 8.0;
                    data[y][x] = [
                        (120.0 + grain * 40.0) as u8,
                        (80.0 + grain * 20.0) as u8,
                        (40.0 + grain * 10.0) as u8,
                    ];
                }
                else if (x >= 26 && x <= 30 && y >= 15 && y <= 19) || 
                        (x >= 33 && x <= 37 && y >= 15 && y <= 19) {
                    data[y][x] = [200, 180, 120]; 
                }
                else if x >= 20 && x <= 43 && y >= 30 && y <= 40 {
                    data[y][x] = [0, 150, 0];
                }
                else if x >= 22 && x <= 41 && y >= 32 && y <= 38 {
                    let text_pattern = match x {
                        22..=24 | 26..=28 | 30..=32 | 34..=36 | 38..=41 => {
                            if y >= 33 && y <= 37 { [255, 255, 255] } else { [0, 150, 0] }
                        }
                        _ => [0, 150, 0]
                    };
                    data[y][x] = text_pattern;
                }
                else if x >= 29 && x <= 34 && y >= 6 && y <= 25 {
                    data[y][x] = [60, 40, 20]; 
                }
                else if y >= 50 {
                    data[y][x] = [80, 80, 75];
                }
                else {
                    data[y][x] = [90, 90, 85];
                }
            }
        }
        
        println!("Textura de puerta de salida generada (fallback)");
        Self { data, width: size, height: size }
    }
    
    fn generate_wall_texture() -> Self {
        let size = 64;
        let mut data = vec![vec![[0, 0, 0]; size]; size];
        
        for y in 0..size {
            for x in 0..size {
                let brick_width = 16;
                let brick_height = 8;
                let _brick_x = x / brick_width;
                let brick_y = y / brick_height;
                
                let offset = if brick_y % 2 == 0 { 0 } else { brick_width / 2 };
                let adjusted_x = (x + offset) % (brick_width * 2);
                
                if adjusted_x % brick_width < brick_width - 1 && y % brick_height < brick_height - 1 {
                    let variation = ((x + y) % 16) as f32 / 16.0;
                    data[y][x] = [
                        (120.0 + variation * 40.0) as u8,
                        (60.0 + variation * 20.0) as u8,
                        (40.0 + variation * 15.0) as u8,
                    ];
                } else {
                    data[y][x] = [90, 90, 85];
                }
            }
        }
        
        println!("Textura de pared generada (fallback)");
        Self { data, width: size, height: size }
    }
    
    pub fn get_color(&self, tex_x: f32, tex_y: f32) -> [u8; 3] {
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
    pub exit_texture: ImageTexture,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            wall_texture: ImageTexture::from_file("assets/wall.png"),
            floor_texture: ImageTexture::from_file("assets/floor.png"),
            ceiling_texture: ImageTexture::from_file("assets/ceiling.png"),
            exit_texture: ImageTexture::from_file("assets/exit.png"),
        }
    }
    
    pub fn get_texture(&self, wall_char: char) -> &ImageTexture {
        match wall_char {
            'E' => &self.exit_texture,   
            '+' | '-' | '|' => &self.wall_texture,
            '1' => &self.wall_texture,     
            '2' => &self.wall_texture,    
            '3' => &self.wall_texture,     
            _ => &self.wall_texture,
        }
    }
}