use macroquad::prelude::*;
use std::f32::consts::PI;

// AI Agent structure
struct AIAgent {
    position: Vec3,
    direction: Vec3,
    speed: f32,
    color: Color,
    change_timer: f32,
    change_interval: f32,
}

impl AIAgent {
    fn new(x: f32, z: f32, color: Color) -> Self {
        Self {
            position: Vec3::new(x, 0.5, z),
            direction: Vec3::new(
                (rand::gen_range(0.0, 2.0 * PI)).sin(),
                0.0,
                (rand::gen_range(0.0, 2.0 * PI)).cos(),
            ).normalize(),
            speed: rand::gen_range(1.5, 3.5),
            color,
            change_timer: 0.0,
            change_interval: rand::gen_range(1.0, 3.0),
        }
    }

    fn update(&mut self, delta_time: f32) {
        // Update timer for direction changes
        self.change_timer += delta_time;
        
        // Change direction randomly
        if self.change_timer >= self.change_interval {
            let angle = rand::gen_range(0.0, 2.0 * PI);
            self.direction = Vec3::new(angle.sin(), 0.0, angle.cos()).normalize();
            self.change_timer = 0.0;
            self.change_interval = rand::gen_range(1.0, 3.0);
        }
        
        // Move the agent
        self.position += self.direction * self.speed * delta_time;
        
        // Keep within bounds
        let bounds = 8.0;
        if self.position.x.abs() > bounds {
            self.direction.x *= -1.0;
            self.position.x = self.position.x.clamp(-bounds, bounds);
        }
        if self.position.z.abs() > bounds {
            self.direction.z *= -1.0;
            self.position.z = self.position.z.clamp(-bounds, bounds);
        }
    }

    fn draw(&self) {
        draw_cube(self.position, Vec3::ONE, None, self.color);
        
        // Draw a small indicator showing direction
        let indicator_pos = self.position + self.direction * 0.8;
        draw_cube(indicator_pos, Vec3::new(0.2, 0.2, 0.2), None, WHITE);
    }
}

#[macroquad::main("Agentira Prototype")]
async fn main() {
    let mut camera = Camera3D {
        position: Vec3::new(10.0, 8.0, 10.0),
        up: Vec3::Y,
        target: Vec3::ZERO,
        ..Default::default()
    };

    // Create AI agents
    let mut agents = vec![
        AIAgent::new(-4.0, -2.0, RED),
        AIAgent::new(-2.0, -1.0, GREEN),
        AIAgent::new(0.0, 0.0, BLUE),
        AIAgent::new(2.0, 1.0, YELLOW),
        AIAgent::new(4.0, 2.0, MAGENTA),
    ];

    let mut camera_angle = 0.0;
    let camera_radius = 15.0;

    loop {
        let delta_time = get_frame_time();
        
        // Update camera rotation
        camera_angle += 0.3 * delta_time;
        camera.position = Vec3::new(
            camera_angle.sin() * camera_radius,
            8.0,
            camera_angle.cos() * camera_radius,
        );
        camera.target = Vec3::ZERO;

        // Update AI agents
        for agent in &mut agents {
            agent.update(delta_time);
        }

        // Draw
        clear_background(SKYBLUE);

        set_camera(&camera);

        // Draw ground
        draw_plane(Vec3::new(0.0, -0.1, 0.0), Vec2::new(20.0, 20.0), None, DARKGREEN);
        
        // Draw grid
        for i in -10..=10 {
            let line_color = if i == 0 { WHITE } else { GRAY };
            draw_line_3d(
                Vec3::new(i as f32, 0.0, -10.0),
                Vec3::new(i as f32, 0.0, 10.0),
                line_color,
            );
            draw_line_3d(
                Vec3::new(-10.0, 0.0, i as f32),
                Vec3::new(10.0, 0.0, i as f32),
                line_color,
            );
        }

        // Draw AI agents
        for agent in &agents {
            agent.draw();
        }

        // Reset camera for UI
        set_default_camera();

        // Draw UI
        draw_text("Agentira Prototype - AI Agents Moving Around", 20.0, 30.0, 20.0, DARKGRAY);
        draw_text(&format!("FPS: {:.0}", get_fps()), 20.0, 50.0, 16.0, DARKGRAY);
        draw_text("Camera auto-rotating • Pixel AI agents moving randomly", 20.0, 70.0, 16.0, DARKGRAY);

        next_frame().await
    }
}