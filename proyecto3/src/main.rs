use minifb::{Key, Window, WindowOptions};
use std::f32::consts::PI;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;
const FOV: f32 = PI / 2.5;

#[derive(Clone, Copy, Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    fn from_float(r: f32, g: f32, b: f32) -> Self {
        Color {
            r: (r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }

    fn to_u32(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    fn mul(&self, factor: f32) -> Color {
        Color::from_float(
            self.r as f32 / 255.0 * factor,
            self.g as f32 / 255.0 * factor,
            self.b as f32 / 255.0 * factor,
        )
    }

    fn lerp(&self, other: &Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color::from_float(
            (self.r as f32 / 255.0) * (1.0 - t) + (other.r as f32 / 255.0) * t,
            (self.g as f32 / 255.0) * (1.0 - t) + (other.g as f32 / 255.0) * t,
            (self.b as f32 / 255.0) * (1.0 - t) + (other.b as f32 / 255.0) * t,
        )
    }
}

#[derive(Clone, Copy, Debug)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn normalize(&self) -> Vec3 {
        let len = self.length();
        if len > 0.0001 {
            Vec3::new(self.x / len, self.y / len, self.z / len)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        }
    }

    fn add(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    fn sub(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    fn mul(&self, scalar: f32) -> Vec3 {
        Vec3::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }

    fn rotate_y(&self, angle: f32) -> Vec3 {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Vec3::new(
            self.x * cos_a + self.z * sin_a,
            self.y,
            -self.x * sin_a + self.z * cos_a,
        )
    }

    fn rotate_x(&self, angle: f32) -> Vec3 {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Vec3::new(
            self.x,
            self.y * cos_a - self.z * sin_a,
            self.y * sin_a + self.z * cos_a,
        )
    }

    fn rotate_z(&self, angle: f32) -> Vec3 {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Vec3::new(
            self.x * cos_a - self.y * sin_a,
            self.x * sin_a + self.y * cos_a,
            self.z,
        )
    }
}

struct Spaceship {
    position: Vec3,
    velocity: Vec3,
    yaw: f32,
    pitch: f32,
    roll: f32,
    target_roll: f32,
}

impl Spaceship {
    fn new() -> Self {
        Spaceship {
            position: Vec3::new(0.0, 5.0, 25.0),
            velocity: Vec3::new(0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            target_roll: 0.0,
        }
    }

    fn get_forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        )
    }

    fn get_right(&self) -> Vec3 {
        let forward = self.get_forward();
        let up = Vec3::new(0.0, 1.0, 0.0);
        forward.cross(&up).normalize()
    }

    fn update(&mut self, dt: f32, planets: &[Planet]) {
        let new_position = self.position.add(&self.velocity.mul(dt));
        
        if !check_collision(&new_position, planets) {
            self.position = new_position;
        } else {
            self.velocity = self.velocity.mul(0.5);
        }
        
        self.velocity = self.velocity.mul(0.95);
        self.roll += (self.target_roll - self.roll) * 5.0 * dt;
    }

    fn accelerate(&mut self, direction: Vec3, speed: f32) {
        self.velocity = self.velocity.add(&direction.mul(speed));
        let max_speed = 2.5;
        let vel_len = self.velocity.length();
        if vel_len > max_speed {
            self.velocity = self.velocity.mul(max_speed / vel_len);
        }
    }

    fn warp_to(&mut self, target: Vec3, target_yaw: f32, target_pitch: f32) {
        self.position = target;
        self.yaw = target_yaw;
        self.pitch = target_pitch;
        self.velocity = Vec3::new(0.0, 0.0, 0.0);
        self.roll = 0.0;
        self.target_roll = 0.0;
    }
}

struct Camera {
    distance: f32,
    height: f32,
    smoothed_position: Vec3,
    smoothed_yaw: f32,
    smoothed_pitch: f32,
}

impl Camera {
    fn new() -> Self {
        Camera {
            distance: 10.0,
            height: 4.0,
            smoothed_position: Vec3::new(0.0, 5.0, 25.0),
            smoothed_yaw: 0.0,
            smoothed_pitch: 0.0,
        }
    }

    fn update(&mut self, spaceship: &Spaceship, dt: f32) {
        let smooth_factor = 5.0 * dt;
        
        self.smoothed_position = Vec3::new(
            self.smoothed_position.x + (spaceship.position.x - self.smoothed_position.x) * smooth_factor,
            self.smoothed_position.y + (spaceship.position.y - self.smoothed_position.y) * smooth_factor,
            self.smoothed_position.z + (spaceship.position.z - self.smoothed_position.z) * smooth_factor,
        );
        
        self.smoothed_yaw += angle_difference(spaceship.yaw, self.smoothed_yaw) * smooth_factor;
        self.smoothed_pitch += (spaceship.pitch - self.smoothed_pitch) * smooth_factor;
    }

    fn get_position(&self) -> Vec3 {
        let offset = Vec3::new(
            -self.smoothed_yaw.sin() * self.smoothed_pitch.cos() * self.distance,
            self.height - self.smoothed_pitch.sin() * self.distance * 0.5,
            -self.smoothed_yaw.cos() * self.smoothed_pitch.cos() * self.distance,
        );
        self.smoothed_position.add(&offset)
    }

    fn get_forward(&self) -> Vec3 {
        self.smoothed_position.sub(&self.get_position()).normalize()
    }

    fn get_right(&self) -> Vec3 {
        let forward = self.get_forward();
        let up = Vec3::new(0.0, 1.0, 0.0);
        forward.cross(&up).normalize()
    }
}

fn angle_difference(target: f32, current: f32) -> f32 {
    let mut diff = target - current;
    while diff > PI {
        diff -= 2.0 * PI;
    }
    while diff < -PI {
        diff += 2.0 * PI;
    }
    diff
}

#[derive(Clone, Copy)]
enum ShaderType {
    Sun,
    Earth,
    GasGiant,
    Ice,
    Desert,
    Lava,
    Purple,
    Moon,
}

struct Planet {
    position: Vec3,
    orbit_radius: f32,
    orbit_speed: f32,
    rotation_speed: f32,
    scale: f32,
    shader: ShaderType,
    rotation: f32,
    orbit_angle: f32,
    has_rings: bool,
    ring_color: Color,
    moons: Vec<Moon>,
}

struct Moon {
    orbit_radius: f32,
    orbit_speed: f32,
    size: f32,
    angle: f32,
}

impl Moon {
    fn update(&mut self, dt: f32) {
        self.angle += self.orbit_speed * dt;
    }
    
    fn get_position(&self, planet_pos: &Vec3) -> Vec3 {
        Vec3::new(
            planet_pos.x + self.orbit_radius * self.angle.cos(),
            planet_pos.y,
            planet_pos.z + self.orbit_radius * self.angle.sin(),
        )
    }
}

impl Planet {
    fn update(&mut self, dt: f32) {
        self.orbit_angle += self.orbit_speed * dt;
        self.rotation += self.rotation_speed * dt;
        
        self.position = Vec3::new(
            self.orbit_radius * self.orbit_angle.cos(),
            0.0,
            self.orbit_radius * self.orbit_angle.sin(),
        );
        
        // Update moons
        for moon in &mut self.moons {
            moon.update(dt);
        }
    }
}

fn check_collision(pos: &Vec3, planets: &[Planet]) -> bool {
    for planet in planets {
        let dist = pos.sub(&planet.position).length();
        if dist < planet.scale + 2.0 {
            return true;
        }
    }
    false
}

fn noise(x: f32, y: f32, z: f32) -> f32 {
    (x.sin() * 43758.5453 + y.sin() * 22578.1459 + z.cos() * 19134.3872).fract()
}

fn fbm(p: &Vec3, octaves: i32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;
    
    for _ in 0..octaves {
        value += noise(p.x * frequency, p.y * frequency, p.z * frequency) * amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    value
}

fn apply_planet_shader(normal: &Vec3, light_intensity: f32, shader: ShaderType, time: f32) -> Color {
    match shader {
        ShaderType::Sun => {
            let glow = 0.9 + (time * 2.0).sin() * 0.1;
            let core = Color::new(255, 240, 200);
            let corona = Color::new(255, 180, 80);
            let t = (normal.y * 0.5 + 0.5) * glow;
            core.lerp(&corona, t)
        },
        ShaderType::Earth => {
            let ocean = Color::new(30, 80, 180);
            let land = Color::new(60, 150, 80);
            let clouds = Color::new(220, 220, 240);
            
            let continent = fbm(&normal.mul(3.0), 3);
            let cloud_pattern = fbm(&normal.mul(8.0).add(&Vec3::new(time * 0.1, 0.0, 0.0)), 2);
            
            let mut base = if continent > 0.5 { land } else { ocean };
            if cloud_pattern > 0.6 {
                base = base.lerp(&clouds, 0.7);
            }
            base.mul(light_intensity.max(0.2))
        },
        ShaderType::GasGiant => {
            let base1 = Color::new(220, 180, 120);
            let base2 = Color::new(180, 140, 90);
            let band = ((normal.y + time * 0.05).sin() * 10.0).fract();
            let turbulence = fbm(&Vec3::new(normal.x * 5.0, normal.y * 15.0, normal.z * 5.0), 2);
            let color = base1.lerp(&base2, band + turbulence * 0.3);
            color.mul(light_intensity.max(0.15))
        },
        ShaderType::Ice => {
            let ice1 = Color::new(180, 220, 255);
            let ice2 = Color::new(120, 180, 240);
            let cracks = fbm(&normal.mul(8.0), 3);
            let color = ice1.lerp(&ice2, cracks);
            color.mul(light_intensity.max(0.3))
        },
        ShaderType::Desert => {
            let sand1 = Color::new(220, 160, 100);
            let sand2 = Color::new(180, 120, 60);
            let dunes = fbm(&normal.mul(6.0), 3);
            let color = sand1.lerp(&sand2, dunes);
            color.mul(light_intensity.max(0.2))
        },
        ShaderType::Lava => {
            let dark = Color::new(80, 30, 20);
            let hot = Color::new(255, 80, 30);
            let glow = Color::new(255, 200, 100);
            
            let pattern = fbm(&normal.mul(4.0).add(&Vec3::new(time * 0.2, 0.0, 0.0)), 3);
            let pulse = (time * 3.0 + pattern * 10.0).sin() * 0.5 + 0.5;
            
            let base = dark.lerp(&hot, pattern);
            let final_color = base.lerp(&glow, pulse * pattern);
            final_color.mul(light_intensity.max(0.3))
        },
        ShaderType::Purple => {
            let base1 = Color::new(150, 100, 200);
            let base2 = Color::new(100, 60, 160);
            let bands = (normal.y * 8.0 + time * 0.1).sin() * 0.5 + 0.5;
            let color = base1.lerp(&base2, bands);
            color.mul(light_intensity.max(0.15))
        },
        ShaderType::Moon => {
            let gray1 = Color::new(180, 180, 180);
            let gray2 = Color::new(120, 120, 120);
            let craters = fbm(&normal.mul(10.0), 4);
            let color = gray1.lerp(&gray2, craters);
            color.mul(light_intensity.max(0.1))
        },
    }
}

fn project_vertex(
    vertex: &Vec3,
    camera_pos: &Vec3,
    camera_forward: &Vec3,
    camera_right: &Vec3,
) -> Option<(f32, f32, f32)> {
    let relative = vertex.sub(camera_pos);
    let camera_up = camera_right.cross(camera_forward).normalize();
    
    let x = relative.dot(camera_right);
    let y = relative.dot(&camera_up);
    let z = relative.dot(camera_forward);
    
    if z <= 0.1 {
        return None;
    }
    
    let aspect = WIDTH as f32 / HEIGHT as f32;
    let fov_factor = (FOV / 2.0).tan();
    
    let screen_x = (WIDTH as f32 / 2.0) * (1.0 + x / (z * fov_factor * aspect));
    let screen_y = (HEIGHT as f32 / 2.0) * (1.0 - y / (z * fov_factor));
    
    Some((screen_x, screen_y, z))
}

fn render_sphere(
    buffer: &mut [u32],
    z_buffer: &mut [f32],
    center: &Vec3,
    radius: f32,
    shader: ShaderType,
    rotation: f32,
    camera: &Camera,
    time: f32,
) {
    let camera_pos = camera.get_position();
    let camera_forward = camera.get_forward();
    let camera_right = camera.get_right();
    
    if let Some((cx, cy, depth)) = project_vertex(center, &camera_pos, &camera_forward, &camera_right) {
        let dist = center.sub(&camera_pos).length();
        if dist > 250.0 {
            return;
        }
        
        let screen_radius = (radius * WIDTH as f32 / (2.0 * dist * (FOV / 2.0).tan())) as i32;
        let light_dir = Vec3::new(0.0, 0.0, 0.0).sub(center).normalize();
        
        let x_min = ((cx - screen_radius as f32).max(0.0) as i32).max(0).min(WIDTH as i32 - 1);
        let x_max = ((cx + screen_radius as f32).min(WIDTH as f32) as i32).max(0).min(WIDTH as i32 - 1);
        let y_min = ((cy - screen_radius as f32).max(0.0) as i32).max(0).min(HEIGHT as i32 - 1);
        let y_max = ((cy + screen_radius as f32).min(HEIGHT as f32) as i32).max(0).min(HEIGHT as i32 - 1);
        
        // FIXED: Changed y_min..x_max to y_min..=y_max
        for y in y_min..=y_max {
            for x in x_min..=x_max {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let dist_sq = dx * dx + dy * dy;
                let r_sq = screen_radius as f32 * screen_radius as f32;
                
                if dist_sq <= r_sq {
                    let sphere_z_sq = r_sq - dist_sq;
                    if sphere_z_sq >= 0.0 {
                        let sphere_z = sphere_z_sq.sqrt();
                        let pixel_depth = depth - sphere_z / (WIDTH as f32);
                        
                        let idx = y as usize * WIDTH + x as usize;
                        if idx < buffer.len() && pixel_depth > z_buffer[idx] {
                            z_buffer[idx] = pixel_depth;
                            
                            let nx = dx / screen_radius as f32;
                            let ny = dy / screen_radius as f32;
                            let nz = sphere_z / screen_radius as f32;
                            
                            let normal = Vec3::new(nx, -ny, nz).normalize();
                            let light_intensity = normal.dot(&light_dir).max(0.0);
                            
                            let rotated_normal = normal.rotate_y(rotation);
                            let color = apply_planet_shader(&rotated_normal, light_intensity, shader, time);
                            
                            buffer[idx] = color.to_u32();
                        }
                    }
                }
            }
        }
    }
}

fn render_orbit(buffer: &mut [u32], radius: f32, camera: &Camera, color: u32) {
    let camera_pos = camera.get_position();
    let camera_forward = camera.get_forward();
    let camera_right = camera.get_right();
    
    let segments = 150;
    for i in 0..segments {
        let angle = 2.0 * PI * i as f32 / segments as f32;
        let v = Vec3::new(radius * angle.cos(), 0.0, radius * angle.sin());
        
        if let Some((sx, sy, _)) = project_vertex(&v, &camera_pos, &camera_forward, &camera_right) {
            let x = sx as i32;
            let y = sy as i32;
            if x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32 {
                let idx = y as usize * WIDTH + x as usize;
                buffer[idx] = color;
            }
        }
    }
}

fn draw_line(buffer: &mut [u32], z_buffer: &mut [f32], x0: i32, y0: i32, z0: f32, x1: i32, y1: i32, z1: f32, color: u32) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = x0;
    let mut y = y0;
    let steps = dx.max(dy).max(1);
    
    for step in 0..=steps {
        if x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32 {
            let t = step as f32 / steps as f32;
            let z = z0 + (z1 - z0) * t;
            let idx = y as usize * WIDTH + x as usize;
            if z > z_buffer[idx] {
                z_buffer[idx] = z;
                buffer[idx] = color;
            }
        }
        
        if x == x1 && y == y1 { break; }
        
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

fn render_spaceship(
    buffer: &mut [u32],
    z_buffer: &mut [f32],
    spaceship: &Spaceship,
    camera: &Camera,
) {
    let camera_pos = camera.get_position();
    let camera_forward = camera.get_forward();
    let camera_right = camera.get_right();
    
    // X-WING STYLE SPACESHIP - Star Wars inspired!
    let vertices = vec![
        // Nose cone (pointed like X-Wing)
        Vec3::new(0.0, 0.0, 2.2),
        Vec3::new(-0.15, 0.1, 1.5),
        Vec3::new(0.15, 0.1, 1.5),
        Vec3::new(-0.15, -0.1, 1.5),
        Vec3::new(0.15, -0.1, 1.5),
        
        // Main fuselage
        Vec3::new(-0.2, 0.15, 0.8),
        Vec3::new(0.2, 0.15, 0.8),
        Vec3::new(-0.2, -0.15, 0.8),
        Vec3::new(0.2, -0.15, 0.8),
        
        Vec3::new(-0.2, 0.15, -0.5),
        Vec3::new(0.2, 0.15, -0.5),
        Vec3::new(-0.2, -0.15, -0.5),
        Vec3::new(0.2, -0.15, -0.5),
        
        // Cockpit canopy
        Vec3::new(-0.12, 0.25, 0.5),
        Vec3::new(0.12, 0.25, 0.5),
        Vec3::new(-0.12, 0.25, 0.0),
        Vec3::new(0.12, 0.25, 0.0),
        
        // TOP-LEFT S-FOIL (wing)
        Vec3::new(-0.3, 0.3, 0.3),
        Vec3::new(-1.6, 0.8, 0.0),
        Vec3::new(-1.6, 0.8, -0.7),
        Vec3::new(-0.3, 0.3, -0.6),
        
        // TOP-RIGHT S-FOIL (wing)
        Vec3::new(0.3, 0.3, 0.3),
        Vec3::new(1.6, 0.8, 0.0),
        Vec3::new(1.6, 0.8, -0.7),
        Vec3::new(0.3, 0.3, -0.6),
        
        // BOTTOM-LEFT S-FOIL (wing)
        Vec3::new(-0.3, -0.3, 0.3),
        Vec3::new(-1.6, -0.8, 0.0),
        Vec3::new(-1.6, -0.8, -0.7),
        Vec3::new(-0.3, -0.3, -0.6),
        
        // BOTTOM-RIGHT S-FOIL (wing)
        Vec3::new(0.3, -0.3, 0.3),
        Vec3::new(1.6, -0.8, 0.0),
        Vec3::new(1.6, -0.8, -0.7),
        Vec3::new(0.3, -0.3, -0.6),
        
        // Engine nacelles (4 engines!)
        // Top-left engine
        Vec3::new(-1.4, 0.75, -0.8),
        Vec3::new(-1.4, 0.75, -1.1),
        // Top-right engine
        Vec3::new(1.4, 0.75, -0.8),
        Vec3::new(1.4, 0.75, -1.1),
        // Bottom-left engine
        Vec3::new(-1.4, -0.75, -0.8),
        Vec3::new(-1.4, -0.75, -1.1),
        // Bottom-right engine
        Vec3::new(1.4, -0.75, -0.8),
        Vec3::new(1.4, -0.75, -1.1),
    ];
    
    // Transform vertices
    let mut transformed = Vec::new();
    for v in &vertices {
        let rotated = v.rotate_z(spaceship.roll)
            .rotate_x(spaceship.pitch)
            .rotate_y(spaceship.yaw);
        transformed.push(rotated.add(&spaceship.position));
    }
    
    // Project all vertices
    let mut projected = Vec::new();
    for v in &transformed {
        projected.push(project_vertex(v, &camera_pos, &camera_forward, &camera_right));
    }
    
    // Colors
    let body_color = 0xD8D8D8;    // Light gray
    let wing_color = 0xA0A0A0;    // Medium gray
    let cockpit_color = 0x4080FF; // Blue cockpit
    let engine_color = 0xFF3030;  // Red engines
    let accent_color = 0xFFFFFF;  // White accents
    
    let edges = vec![
        // Nose cone
        (0, 1, accent_color), (0, 2, accent_color), (0, 3, accent_color), (0, 4, accent_color),
        (1, 2, body_color), (2, 4, body_color), (4, 3, body_color), (3, 1, body_color),
        
        // Connect nose to fuselage
        (1, 5, body_color), (2, 6, body_color), (3, 7, body_color), (4, 8, body_color),
        
        // Front fuselage frame
        (5, 6, body_color), (6, 8, body_color), (8, 7, body_color), (7, 5, body_color),
        
        // Rear fuselage frame
        (9, 10, body_color), (10, 12, body_color), (12, 11, body_color), (11, 9, body_color),
        
        // Connect front to rear fuselage
        (5, 9, body_color), (6, 10, body_color), (7, 11, body_color), (8, 12, body_color),
        
        // Cockpit canopy
        (13, 14, cockpit_color), (14, 16, cockpit_color), (16, 15, cockpit_color), (15, 13, cockpit_color),
        (5, 13, cockpit_color), (6, 14, cockpit_color), (9, 15, cockpit_color), (10, 16, cockpit_color),
        
        // TOP-LEFT S-FOIL
        (17, 18, wing_color), (18, 19, wing_color), (19, 20, wing_color), (20, 17, wing_color),
        (5, 17, wing_color), (9, 20, wing_color),
        (18, 19, accent_color), // Wing leading edge
        
        // TOP-RIGHT S-FOIL
        (21, 22, wing_color), (22, 23, wing_color), (23, 24, wing_color), (24, 21, wing_color),
        (6, 21, wing_color), (10, 24, wing_color),
        (22, 23, accent_color), // Wing leading edge
        
        // BOTTOM-LEFT S-FOIL
        (25, 26, wing_color), (26, 27, wing_color), (27, 28, wing_color), (28, 25, wing_color),
        (7, 25, wing_color), (11, 28, wing_color),
        (26, 27, accent_color), // Wing leading edge
        
        // BOTTOM-RIGHT S-FOIL
        (29, 30, wing_color), (30, 31, wing_color), (31, 32, wing_color), (32, 29, wing_color),
        (8, 29, wing_color), (12, 32, wing_color),
        (30, 31, accent_color), // Wing leading edge
        
        // Engine nacelles (glowing red!)
        (33, 34, engine_color), // Top-left engine
        (35, 36, engine_color), // Top-right engine
        (37, 38, engine_color), // Bottom-left engine
        (39, 40, engine_color), // Bottom-right engine
        
        // Connect engines to wings
        (19, 33, engine_color), (19, 34, engine_color),
        (23, 35, engine_color), (23, 36, engine_color),
        (27, 37, engine_color), (27, 38, engine_color),
        (31, 39, engine_color), (31, 40, engine_color),
    ];
    
    // Draw all edges with proper depth
    for (i, j, color) in edges {
        if let (Some((x0, y0, z0)), Some((x1, y1, z1))) = (projected[i], projected[j]) {
            draw_line(buffer, z_buffer, x0 as i32, y0 as i32, z0, x1 as i32, y1 as i32, z1, color);
        }
    }
}

fn render_skybox(buffer: &mut [u32]) {
    let mut rng_state = 12345u32;
    
    for _ in 0..800 {
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let x = (rng_state % WIDTH as u32) as usize;
        
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let y = (rng_state % HEIGHT as u32) as usize;
        
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let brightness = 120 + (rng_state % 136) as u8;
        
        let idx = y * WIDTH + x;
        if idx < buffer.len() {
            buffer[idx] = ((brightness as u32) << 16) 
                        | ((brightness as u32) << 8) 
                        | (brightness as u32);
        }
    }
}

fn main() {
    let mut window = Window::new(
        "Solar System Explorer - WASD:Move | Arrows:Look | Q/E:Up/Down | Shift:Boost | 1-7:Warp",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();
    
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    
    let mut spaceship = Spaceship::new();
    let mut camera = Camera::new();
    let mut time = 0.0f32;
    let mut show_orbits = true;
    
    let mut buffer = vec![0u32; WIDTH * HEIGHT];
    let mut z_buffer = vec![f32::NEG_INFINITY; WIDTH * HEIGHT];
    
    let mut planets = vec![
        Planet {
            position: Vec3::new(0.0, 0.0, 0.0),
            orbit_radius: 0.0,
            orbit_speed: 0.0,
            rotation_speed: 0.05,
            scale: 5.0,
            shader: ShaderType::Sun,
            rotation: 0.0,
            orbit_angle: 0.0,
            has_rings: false,
            ring_color: Color::new(0, 0, 0),
            moons: vec![],
        },
        Planet {
            position: Vec3::new(20.0, 0.0, 0.0),
            orbit_radius: 20.0,
            orbit_speed: 0.3,
            rotation_speed: 0.5,
            scale: 2.0,
            shader: ShaderType::Earth,
            rotation: 0.0,
            orbit_angle: 0.0,
            has_rings: false,
            ring_color: Color::new(0, 0, 0),
            moons: vec![
                Moon {
                    orbit_radius: 4.0,
                    orbit_speed: 2.0,
                    size: 0.5,
                    angle: 0.0,
                }
            ],
        },
        Planet {
            position: Vec3::new(35.0, 0.0, 0.0),
            orbit_radius: 35.0,
            orbit_speed: 0.2,
            rotation_speed: 0.3,
            scale: 4.0,
            shader: ShaderType::GasGiant,
            rotation: 0.0,
            orbit_angle: 1.5,
            has_rings: false,
            ring_color: Color::new(200, 170, 130),
            moons: vec![
                Moon {
                    orbit_radius: 7.0,
                    orbit_speed: 1.5,
                    size: 0.8,
                    angle: 0.0,
                },
                Moon {
                    orbit_radius: 9.0,
                    orbit_speed: 1.2,
                    size: 0.6,
                    angle: PI,
                }
            ],
        },
        Planet {
            position: Vec3::new(50.0, 0.0, 0.0),
            orbit_radius: 50.0,
            orbit_speed: 0.15,
            rotation_speed: 0.4,
            scale: 3.0,
            shader: ShaderType::Ice,
            rotation: 0.0,
            orbit_angle: 3.0,
            has_rings: false,
            ring_color: Color::new(0, 0, 0),
            moons: vec![],
        },
        Planet {
            position: Vec3::new(65.0, 0.0, 0.0),
            orbit_radius: 65.0,
            orbit_speed: 0.12,
            rotation_speed: 0.6,
            scale: 2.5,
            shader: ShaderType::Desert,
            rotation: 0.0,
            orbit_angle: 4.5,
            has_rings: false,
            ring_color: Color::new(0, 0, 0),
            moons: vec![
                Moon {
                    orbit_radius: 5.0,
                    orbit_speed: 1.8,
                    size: 0.6,
                    angle: PI / 2.0,
                }
            ],
        },
        Planet {
            position: Vec3::new(80.0, 0.0, 0.0),
            orbit_radius: 80.0,
            orbit_speed: 0.1,
            rotation_speed: 0.35,
            scale: 2.8,
            shader: ShaderType::Lava,
            rotation: 0.0,
            orbit_angle: 5.5,
            has_rings: false,
            ring_color: Color::new(0, 0, 0),
            moons: vec![
                Moon {
                    orbit_radius: 5.5,
                    orbit_speed: 2.0,
                    size: 0.7,
                    angle: 0.0,
                }
            ],
        },
        Planet {
            position: Vec3::new(95.0, 0.0, 0.0),
            orbit_radius: 95.0,
            orbit_speed: 0.08,
            rotation_speed: 0.25,
            scale: 3.5,
            shader: ShaderType::Purple,
            rotation: 0.0,
            orbit_angle: 0.5,
            has_rings: false,
            ring_color: Color::new(140, 100, 180),
            moons: vec![
                Moon {
                    orbit_radius: 6.0,
                    orbit_speed: 1.6,
                    size: 0.5,
                    angle: 0.0,
                },
                Moon {
                    orbit_radius: 8.5,
                    orbit_speed: 1.1,
                    size: 0.7,
                    angle: PI / 3.0,
                }
            ],
        },
    ];
    
    println!("\n╔═══════════════════════════════════════╗");
    println!("║   SOLAR SYSTEM EXPLORER - ARWING     ║");
    println!("╚═══════════════════════════════════════╝");
    println!("\n🚀 Flight Controls:");
    println!("  W/S        - Accelerate Forward/Back");
    println!("  A/D        - Strafe Left/Right");
    println!("  Q/E        - Altitude Up/Down");
    println!("  Arrow Keys - Pitch & Roll");
    println!("  Shift      - Afterburner Boost");
    println!("\n🌍 Navigation:");
    println!("  1-7 - Warp to Planets");
    println!("  O   - Toggle Orbit Lines");
    println!("  ESC - Exit\n");
    
    let mut last_time = std::time::Instant::now();
    
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let current_time = std::time::Instant::now();
        let dt = (current_time - last_time).as_secs_f32().min(0.033);
        last_time = current_time;
        
        time += dt;
        
        for planet in &mut planets {
            planet.update(dt);
        }
        
        let rotation_speed = 2.0 * dt;
        let mut roll_input = 0.0;
        
        if window.is_key_down(Key::Left) {
            spaceship.yaw -= rotation_speed;
            roll_input = -0.4;
        }
        if window.is_key_down(Key::Right) {
            spaceship.yaw += rotation_speed;
            roll_input = 0.4;
        }
        if window.is_key_down(Key::Up) {
            spaceship.pitch += rotation_speed;
            spaceship.pitch = spaceship.pitch.min(PI / 3.0);
        }
        if window.is_key_down(Key::Down) {
            spaceship.pitch -= rotation_speed;
            spaceship.pitch = spaceship.pitch.max(-PI / 3.0);
        }
        
        spaceship.target_roll = roll_input;
        
        let boost = if window.is_key_down(Key::LeftShift) { 2.5 } else { 1.0 };
        let accel_force = 0.18 * boost;
        
        if window.is_key_down(Key::W) {
            spaceship.accelerate(spaceship.get_forward(), accel_force);
        }
        if window.is_key_down(Key::S) {
            spaceship.accelerate(spaceship.get_forward(), -accel_force);
        }
        if window.is_key_down(Key::A) {
            spaceship.accelerate(spaceship.get_right().mul(-1.0), accel_force * 0.7);
        }
        if window.is_key_down(Key::D) {
            spaceship.accelerate(spaceship.get_right(), accel_force * 0.7);
        }
        if window.is_key_down(Key::Q) {
            spaceship.accelerate(Vec3::new(0.0, -1.0, 0.0), accel_force * 0.7);
        }
        if window.is_key_down(Key::E) {
            spaceship.accelerate(Vec3::new(0.0, 1.0, 0.0), accel_force * 0.7);
        }
        
        if window.is_key_pressed(Key::Key1, minifb::KeyRepeat::No) {
            spaceship.warp_to(Vec3::new(15.0, 8.0, 0.0), PI, -0.2);
        }
        if window.is_key_pressed(Key::Key2, minifb::KeyRepeat::No) && planets.len() > 1 {
            let p = &planets[1];
            let angle = p.orbit_angle;
            spaceship.warp_to(
                Vec3::new(p.position.x + 8.0 * angle.cos(), 5.0, p.position.z + 8.0 * angle.sin()),
                angle + PI, -0.15
            );
        }
        if window.is_key_pressed(Key::Key3, minifb::KeyRepeat::No) && planets.len() > 2 {
            let p = &planets[2];
            let angle = p.orbit_angle;
            spaceship.warp_to(
                Vec3::new(p.position.x + 12.0 * angle.cos(), 8.0, p.position.z + 12.0 * angle.sin()),
                angle + PI, -0.2
            );
        }
        if window.is_key_pressed(Key::Key4, minifb::KeyRepeat::No) && planets.len() > 3 {
            let p = &planets[3];
            let angle = p.orbit_angle;
            spaceship.warp_to(
                Vec3::new(p.position.x + 10.0 * angle.cos(), 6.0, p.position.z + 10.0 * angle.sin()),
                angle + PI, -0.15
            );
        }
        if window.is_key_pressed(Key::Key5, minifb::KeyRepeat::No) && planets.len() > 4 {
            let p = &planets[4];
            let angle = p.orbit_angle;
            spaceship.warp_to(
                Vec3::new(p.position.x + 9.0 * angle.cos(), 5.5, p.position.z + 9.0 * angle.sin()),
                angle + PI, -0.15
            );
        }
        if window.is_key_pressed(Key::Key6, minifb::KeyRepeat::No) && planets.len() > 5 {
            let p = &planets[5];
            let angle = p.orbit_angle;
            spaceship.warp_to(
                Vec3::new(p.position.x + 10.0 * angle.cos(), 6.0, p.position.z + 10.0 * angle.sin()),
                angle + PI, -0.15
            );
        }
        if window.is_key_pressed(Key::Key7, minifb::KeyRepeat::No) && planets.len() > 6 {
            let p = &planets[6];
            let angle = p.orbit_angle;
            spaceship.warp_to(
                Vec3::new(p.position.x + 12.0 * angle.cos(), 7.0, p.position.z + 12.0 * angle.sin()),
                angle + PI, -0.2
            );
        }
        
        if window.is_key_pressed(Key::O, minifb::KeyRepeat::No) {
            show_orbits = !show_orbits;
        }
        
        spaceship.update(dt, &planets);
        camera.update(&spaceship, dt);
        
        buffer.fill(0x000000);
        z_buffer.fill(f32::NEG_INFINITY);
        
        render_skybox(&mut buffer);
        
        if show_orbits {
            for planet in &planets {
                if planet.orbit_radius > 0.0 {
                    render_orbit(&mut buffer, planet.orbit_radius, &camera, 0x505050);
                }
            }
        }
        
        for planet in &planets {
            render_sphere(
                &mut buffer,
                &mut z_buffer,
                &planet.position,
                planet.scale,
                planet.shader,
                planet.rotation,
                &camera,
                time,
            );
            
            // Render moons
            for moon in &planet.moons {
                let moon_pos = moon.get_position(&planet.position);
                render_sphere(
                    &mut buffer,
                    &mut z_buffer,
                    &moon_pos,
                    moon.size,
                    ShaderType::Moon, // Gray rocky moons
                    0.0,
                    &camera,
                    time,
                );
            }
        }
        
        render_spaceship(&mut buffer, &mut z_buffer, &spaceship, &camera);
        
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
