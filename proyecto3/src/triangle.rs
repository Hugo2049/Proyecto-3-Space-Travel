use crate::color::Color;
use crate::framebuffer::Framebuffer;

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vertex { x, y, z }
    }
}

fn line(framebuffer: &mut Framebuffer, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: Color) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        if x0 >= 0 && x0 < 800 && y0 >= 0 && y0 < 600 {
            framebuffer.point(x0 as usize, y0 as usize, color);
        }

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x0 += sx;
        }
        if e2 < dx {
            err += dx;
            y0 += sy;
        }
    }
}

pub fn triangle(framebuffer: &mut Framebuffer, v1: &Vertex, v2: &Vertex, v3: &Vertex, color: Color) {
    let x1 = v1.x as i32;
    let y1 = v1.y as i32;
    let x2 = v2.x as i32;
    let y2 = v2.y as i32;
    let x3 = v3.x as i32;
    let y3 = v3.y as i32;

    // Draw the three edges of the triangle
    line(framebuffer, x1, y1, x2, y2, color);
    line(framebuffer, x2, y2, x3, y3, color);
    line(framebuffer, x3, y3, x1, y1, color);
}
