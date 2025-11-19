use crate::triangle::Vertex;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Face {
    pub vertex_indices: Vec<usize>,
}

pub struct Model {
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Face>,
}

impl Model {
    pub fn new() -> Self {
        Model {
            vertices: Vec::new(),
            faces: Vec::new(),
        }
    }

    pub fn load_from_file(filename: &str) -> Result<Self, std::io::Error> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        
        let mut model = Model::new();

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "v" => {
                    // Vertex line
                    if parts.len() >= 4 {
                        let x: f32 = parts[1].parse().unwrap_or(0.0);
                        let y: f32 = parts[2].parse().unwrap_or(0.0);
                        let z: f32 = parts[3].parse().unwrap_or(0.0);
                        model.vertices.push(Vertex::new(x, y, z));
                    }
                }
                "f" => {
                    // Face line
                    let mut vertex_indices = Vec::new();
                    for i in 1..parts.len() {
                        // Handle faces with format "v/vt/vn" or "v//vn" or just "v"
                        let index_str = parts[i].split('/').next().unwrap();
                        if let Ok(index) = index_str.parse::<usize>() {
                            // OBJ indices are 1-based, convert to 0-based
                            vertex_indices.push(index - 1);
                        }
                    }
                    if !vertex_indices.is_empty() {
                        model.faces.push(Face { vertex_indices });
                    }
                }
                _ => {
                    // Ignore other lines (vn, vt, mtllib, usemtl, etc.)
                }
            }
        }

        Ok(model)
    }
}
