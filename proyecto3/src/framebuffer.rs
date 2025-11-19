use crate::color::Color;
use std::fs::File;
use std::io::Write;

pub const SCREEN_WIDTH: usize = 800;
pub const SCREEN_HEIGHT: usize = 600;

pub struct Framebuffer {
    buffer: Vec<Color>,
    width: usize,
    height: usize,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            buffer: vec![Color::black(); width * height],
            width,
            height,
        }
    }

    pub fn clear(&mut self, color: Color) {
        for pixel in self.buffer.iter_mut() {
            *pixel = color;
        }
    }

    pub fn point(&mut self, x: usize, y: usize, color: Color) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.buffer[index] = color;
        }
    }

    pub fn save_as_ppm(&self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;
        
        // Write PPM header
        writeln!(file, "P3")?;
        writeln!(file, "{} {}", self.width, self.height)?;
        writeln!(file, "255")?;

        // Write pixel data
        for pixel in &self.buffer {
            write!(file, "{} {} {} ", pixel.r, pixel.g, pixel.b)?;
        }

        Ok(())
    }
}
