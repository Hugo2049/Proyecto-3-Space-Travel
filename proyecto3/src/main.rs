mod color;
mod framebuffer;
mod triangle;
mod obj;

use color::Color;
use framebuffer::{Framebuffer, SCREEN_WIDTH, SCREEN_HEIGHT};
use triangle::{triangle, Vertex};
use obj::Model;

fn main() {
    println!("Starting spaceship renderer...");

    // Create framebuffer
    let mut framebuffer = Framebuffer::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    // Load OBJ model
    let model = match Model::load_from_file("spaceship.obj") {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error loading model: {}", e);
            return;
        }
    };

    println!("Loaded model with {} vertices and {} faces", 
             model.vertices.len(), model.faces.len());

    // Calculate model bounds for centering and scaling
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;

    for vertex in &model.vertices {
        min_x = min_x.min(vertex.x);
        max_x = max_x.max(vertex.x);
        min_y = min_y.min(vertex.y);
        max_y = max_y.max(vertex.y);
        min_z = min_z.min(vertex.z);
        max_z = max_z.max(vertex.z);
    }

    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    let center_z = (min_z + max_z) / 2.0;

    let width = max_x - min_x;
    let height = max_y - min_y;
    let depth = max_z - min_z;
    let max_dimension = width.max(height).max(depth);

    // Scale to fit nicely on screen (use about 60% of screen)
    let scale = (SCREEN_WIDTH.min(SCREEN_HEIGHT) as f32 * 0.6) / max_dimension;

    println!("Model bounds: x[{:.2}, {:.2}], y[{:.2}, {:.2}], z[{:.2}, {:.2}]", 
             min_x, max_x, min_y, max_y, min_z, max_z);
    println!("Center: ({:.2}, {:.2}, {:.2})", center_x, center_y, center_z);
    println!("Scale: {:.2}", scale);

    // Transform vertices to screen coordinates
    let mut screen_vertices = Vec::new();
    for vertex in &model.vertices {
        // Center and scale the model
        let x = (vertex.x - center_x) * scale;
        let y = (vertex.y - center_y) * scale;
        let z = (vertex.z - center_z) * scale;

        // Project to screen (simple orthographic projection)
        // Flip Y because screen coordinates have origin at top-left
        let screen_x = x + (SCREEN_WIDTH as f32 / 2.0);
        let screen_y = -y + (SCREEN_HEIGHT as f32 / 2.0);

        screen_vertices.push(Vertex::new(screen_x, screen_y, z));
    }

    // Set the drawing color
    let current_color = Color::new(255, 255, 0); // Yellow

    // Clear the framebuffer
    framebuffer.clear(Color::black());

    println!("Rendering {} faces...", model.faces.len());

    // Render all faces
    for face in &model.faces {
        // Handle triangular and quad faces
        if face.vertex_indices.len() >= 3 {
            // Draw triangles for each face (triangulate if needed)
            for i in 1..face.vertex_indices.len() - 1 {
                let v1 = &screen_vertices[face.vertex_indices[0]];
                let v2 = &screen_vertices[face.vertex_indices[i]];
                let v3 = &screen_vertices[face.vertex_indices[i + 1]];
                
                triangle(&mut framebuffer, v1, v2, v3, current_color);
            }
        }
    }

    // Save the rendered image
    println!("Saving rendered image...");
    match framebuffer.save_as_ppm("spaceship_render.ppm") {
        Ok(_) => println!("Image saved as spaceship_render.ppm"),
        Err(e) => eprintln!("Error saving image: {}", e),
    }

    println!("Rendering complete!");
}

