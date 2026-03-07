use macroquad::prelude::*;
use std::f32::consts::PI;

// AI Agent types
#[derive(Clone, Copy)]
enum AgentType {
    Worker,      // 作業員型
    Scout,       // 偵察型  
    Builder,     // 建設型
    Collector,   // 収集型
    Guardian,    // 警備型
}

// AI Agent structure
struct AIAgent {
    position: Vec3,
    direction: Vec3,
    speed: f32,
    color: Color,
    agent_type: AgentType,
    change_timer: f32,
    change_interval: f32,
}

impl AIAgent {
    fn new(x: f32, z: f32, color: Color, agent_type: AgentType) -> Self {
        // エージェントタイプによって特性を調整
        let speed = match agent_type {
            AgentType::Scout => rand::gen_range(3.0, 4.5),      // 偵察型は高速
            AgentType::Worker => rand::gen_range(1.8, 2.5),     // 作業員は標準
            AgentType::Builder => rand::gen_range(1.2, 2.0),    // 建設型は低速だが安定
            AgentType::Collector => rand::gen_range(2.0, 3.0),  // 収集型は中速
            AgentType::Guardian => rand::gen_range(1.5, 2.2),   // 警備型は慎重
        };
        
        Self {
            position: Vec3::new(x, 0.5, z),
            direction: Vec3::new(
                (rand::gen_range(0.0, 2.0 * PI)).sin(),
                0.0,
                (rand::gen_range(0.0, 2.0 * PI)).cos(),
            ).normalize(),
            speed,
            color,
            agent_type,
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
        self.draw_pixelated_agent();
        
        // Draw direction indicator (floating arrow)
        let indicator_pos = self.position + self.direction * 1.2 + Vec3::new(0.0, 1.5, 0.0);
        draw_cube(indicator_pos, Vec3::new(0.15, 0.15, 0.15), None, WHITE);
    }

    fn draw_pixelated_agent(&self) {
        let pos = self.position;
        let base_color = self.color;
        
        // エージェントタイプによって外見を調整
        let (head_scale, body_scale, has_jetpack, antenna_style, core_color) = match self.agent_type {
            AgentType::Worker => (0.6, 0.8, false, YELLOW, SKYBLUE),
            AgentType::Scout => (0.5, 0.6, true, GREEN, LIME),           // 小型高速
            AgentType::Builder => (0.7, 1.0, false, ORANGE, ORANGE),     // 大型安定
            AgentType::Collector => (0.6, 0.7, true, PURPLE, VIOLET),    // 中型多機能
            AgentType::Guardian => (0.8, 1.1, false, RED, RED),          // 大型防御
        };
        
        // 頭部 (アンテナ付きロボットヘッド)
        let head_pos = pos + Vec3::new(0.0, 1.0, 0.0);
        draw_cube(head_pos, Vec3::new(head_scale, head_scale, head_scale), None, base_color);
        
        // 目 (LEDライト風)
        let eye_color = WHITE;
        let eye_offset = head_scale * 0.25;
        draw_cube(head_pos + Vec3::new(-eye_offset, 0.1, head_scale * 0.6), Vec3::new(0.1, 0.1, 0.1), None, eye_color);
        draw_cube(head_pos + Vec3::new(eye_offset, 0.1, head_scale * 0.6), Vec3::new(0.1, 0.1, 0.1), None, eye_color);
        
        // アンテナ (エージェントタイプによって色が変わる)
        draw_cube(head_pos + Vec3::new(0.0, head_scale * 0.6, 0.0), Vec3::new(0.05, 0.3, 0.05), None, antenna_style);
        draw_cube(head_pos + Vec3::new(0.0, head_scale * 0.8, 0.0), Vec3::new(0.1, 0.1, 0.1), None, core_color);
        
        // 胴体 (メインボディ)
        let body_pos = pos + Vec3::new(0.0, 0.2, 0.0);
        let body_color = Color::new(base_color.r * 0.8, base_color.g * 0.8, base_color.b * 0.8, 1.0);
        draw_cube(body_pos, Vec3::new(body_scale, 1.0, 0.4), None, body_color);
        
        // コア/チェストライト (エネルギーコア風)
        draw_cube(body_pos + Vec3::new(0.0, 0.2, 0.25), Vec3::new(0.2, 0.2, 0.1), None, core_color);
        
        // 腕部 (左右) - ビルダーは太い腕
        let arm_width = if matches!(self.agent_type, AgentType::Builder) { 0.3 } else { 0.2 };
        let arm_color = Color::new(base_color.r * 0.6, base_color.g * 0.6, base_color.b * 0.6, 1.0);
        // 左腕
        draw_cube(pos + Vec3::new(-body_scale * 0.75, 0.4, 0.0), Vec3::new(arm_width, 0.6, arm_width), None, arm_color);
        draw_cube(pos + Vec3::new(-body_scale - 0.2, -0.1, 0.0), Vec3::new(arm_width, 0.4, arm_width), None, arm_color);
        // 右腕  
        draw_cube(pos + Vec3::new(body_scale * 0.75, 0.4, 0.0), Vec3::new(arm_width, 0.6, arm_width), None, arm_color);
        draw_cube(pos + Vec3::new(body_scale + 0.2, -0.1, 0.0), Vec3::new(arm_width, 0.4, arm_width), None, arm_color);
        
        // 脚部 (左右) - ガーディアンは太い脚
        let leg_width = if matches!(self.agent_type, AgentType::Guardian) { 0.35 } else { 0.25 };
        let leg_color = DARKGRAY;
        // 左脚
        draw_cube(pos + Vec3::new(-body_scale * 0.3, -0.4, 0.0), Vec3::new(leg_width, 0.6, leg_width), None, leg_color);
        draw_cube(pos + Vec3::new(-body_scale * 0.3, -0.9, 0.0), Vec3::new(0.3, 0.2, 0.4), None, GRAY);
        // 右脚
        draw_cube(pos + Vec3::new(body_scale * 0.3, -0.4, 0.0), Vec3::new(leg_width, 0.6, leg_width), None, leg_color);
        draw_cube(pos + Vec3::new(body_scale * 0.3, -0.9, 0.0), Vec3::new(0.3, 0.2, 0.4), None, GRAY);
        
        // ジェットパック (スカウト・コレクターのみ)
        if has_jetpack {
            let jetpack_color = if matches!(self.agent_type, AgentType::Scout) { LIME } else { PURPLE };
            draw_cube(pos + Vec3::new(0.0, 0.3, -0.4), Vec3::new(0.4, 0.8, 0.2), None, jetpack_color);
            // ジェットの炎エフェクト
            draw_cube(pos + Vec3::new(0.0, 0.1, -0.6), Vec3::new(0.1, 0.1, 0.2), None, ORANGE);
        }
        
        // 肩のディテール
        let detail_color = match self.agent_type {
            AgentType::Guardian => RED,
            AgentType::Builder => ORANGE,
            _ => GOLD,
        };
        draw_cube(pos + Vec3::new(-body_scale * 0.6, 0.7, 0.0), Vec3::new(0.15, 0.15, 0.15), None, detail_color);
        draw_cube(pos + Vec3::new(body_scale * 0.6, 0.7, 0.0), Vec3::new(0.15, 0.15, 0.15), None, detail_color);
        
        // 特殊装備
        match self.agent_type {
            AgentType::Builder => {
                // 建設用ツール
                draw_cube(pos + Vec3::new(body_scale + 0.2, -0.3, 0.0), Vec3::new(0.1, 0.3, 0.1), None, BROWN);
            },
            AgentType::Collector => {
                // 収集コンテナ
                draw_cube(pos + Vec3::new(0.0, 0.8, -0.3), Vec3::new(0.3, 0.3, 0.2), None, PURPLE);
            },
            AgentType::Guardian => {
                // シールドジェネレーター
                draw_cube(pos + Vec3::new(0.0, 0.5, 0.4), Vec3::new(0.4, 0.2, 0.1), None, BLUE);
            },
            _ => {}
        }
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

    // Create AI agents with different types
    let mut agents = vec![
        AIAgent::new(-4.0, -2.0, RED, AgentType::Worker),
        AIAgent::new(-2.0, -1.0, GREEN, AgentType::Scout),
        AIAgent::new(0.0, 0.0, BLUE, AgentType::Builder),
        AIAgent::new(2.0, 1.0, YELLOW, AgentType::Collector),
        AIAgent::new(4.0, 2.0, MAGENTA, AgentType::Guardian),
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
        draw_text("Agentira Prototype - 3D Pixel AI Agents", 20.0, 30.0, 20.0, DARKGRAY);
        draw_text(&format!("FPS: {:.0}", get_fps()), 20.0, 50.0, 16.0, DARKGRAY);
        draw_text("5 Agent Types: Worker(Red) Scout(Green) Builder(Blue) Collector(Yellow) Guardian(Magenta)", 20.0, 70.0, 14.0, DARKGRAY);
        draw_text("Each agent has unique design, speed, and equipment", 20.0, 90.0, 14.0, DARKGRAY);

        next_frame().await
    }
}