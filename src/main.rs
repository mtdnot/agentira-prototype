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

#[derive(Clone, Copy)]
enum FlockMode {
    Wandering,    // 自由行動
    Following,    // リーダー追従
    Formation,    // フォーメーション維持
    Gathering,    // 集結
}

#[derive(Clone, Copy)]
enum FormationType {
    VFormation,   // V字フォーメーション
    Circle,       // 円形フォーメーション
    Line,         // 一列フォーメーション
}

// AI Agent structure
#[derive(Clone)]
struct AIAgent {
    position: Vec3,
    direction: Vec3,
    speed: f32,
    color: Color,
    agent_type: AgentType,
    change_timer: f32,
    change_interval: f32,
    // 群れ行動用
    flock_mode: FlockMode,
    is_leader: bool,
    target_position: Option<Vec3>,
    formation_offset: Vec3,
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
            // 群れ行動初期化
            flock_mode: FlockMode::Wandering,
            is_leader: false,
            target_position: None,
            formation_offset: Vec3::ZERO,
        }
    }

    fn update(&mut self, delta_time: f32, agents: &[AIAgent]) {
        // Update timer for behavior changes
        self.change_timer += delta_time;
        
        // 群れ行動計算
        let desired_direction = self.calculate_flock_behavior(agents);
        
        // スムーズに方向転換
        self.direction = self.direction.lerp(desired_direction, 2.0 * delta_time).normalize();
        
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
    
    fn calculate_flock_behavior(&mut self, agents: &[AIAgent]) -> Vec3 {
        match self.flock_mode {
            FlockMode::Wandering => self.calculate_wandering(),
            FlockMode::Following => self.calculate_following(agents),
            FlockMode::Formation => self.calculate_formation(agents),
            FlockMode::Gathering => self.calculate_gathering(agents),
        }
    }
    
    fn calculate_wandering(&mut self) -> Vec3 {
        // 定期的にランダムな方向に変更
        if self.change_timer >= self.change_interval {
            self.change_timer = 0.0;
            self.change_interval = rand::gen_range(1.0, 3.0);
            let angle = rand::gen_range(0.0, 2.0 * PI);
            Vec3::new(angle.sin(), 0.0, angle.cos()).normalize()
        } else {
            self.direction
        }
    }
    
    fn calculate_following(&self, agents: &[AIAgent]) -> Vec3 {
        // リーダーを探して追従
        if let Some(leader) = agents.iter().find(|a| a.is_leader) {
            let to_leader = (leader.position - self.position).normalize();
            let distance = (leader.position - self.position).length();
            
            // 適切な距離を保つ
            if distance > 3.0 {
                to_leader // リーダーに近づく
            } else if distance < 1.5 {
                -to_leader // 少し離れる
            } else {
                leader.direction // リーダーと同じ方向
            }
        } else {
            self.direction
        }
    }
    
    fn calculate_formation(&self, agents: &[AIAgent]) -> Vec3 {
        // フォーメーション位置を計算
        if let Some(target) = self.target_position {
            let to_target = (target - self.position);
            if to_target.length() > 0.5 {
                to_target.normalize()
            } else {
                self.direction // 位置についたら現在方向を維持
            }
        } else {
            self.direction
        }
    }
    
    fn calculate_gathering(&self, agents: &[AIAgent]) -> Vec3 {
        // 中心点に向かって集結
        let center = Vec3::ZERO;
        let to_center = (center - self.position);
        
        // 他のエージェントとの距離を考慮
        let mut separation = Vec3::ZERO;
        let mut neighbor_count = 0;
        
        for other in agents {
            if std::ptr::eq(self, other) { continue; }
            
            let distance = (self.position - other.position).length();
            if distance < 2.0 && distance > 0.1 {
                separation += (self.position - other.position).normalize() / distance;
                neighbor_count += 1;
            }
        }
        
        if neighbor_count > 0 {
            separation = separation / neighbor_count as f32;
        }
        
        // 中心への移動と他エージェントからの分離をバランス
        (to_center.normalize() * 0.7 + separation * 0.3).normalize()
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

    // Scoutをリーダーに設定
    agents[1].is_leader = true;
    agents[1].flock_mode = FlockMode::Wandering;
    
    // 他のエージェントをFollowingモードに
    for i in 0..agents.len() {
        if i != 1 { // Scout以外
            agents[i].flock_mode = FlockMode::Following;
        }
    }

    let mut camera_angle = 0.0;
    let camera_radius = 15.0;
    let mut flock_mode_timer = 0.0;

    loop {
        let delta_time = get_frame_time();
        flock_mode_timer += delta_time;
        
        // キーボード入力で群れ行動モード切り替え
        if is_key_pressed(KeyCode::Key1) {
            set_all_agents_mode(&mut agents, FlockMode::Wandering);
        } else if is_key_pressed(KeyCode::Key2) {
            set_follow_mode(&mut agents);
        } else if is_key_pressed(KeyCode::Key3) {
            set_formation_mode(&mut agents, FormationType::VFormation);
        } else if is_key_pressed(KeyCode::Key4) {
            set_formation_mode(&mut agents, FormationType::Circle);
        } else if is_key_pressed(KeyCode::Key5) {
            set_all_agents_mode(&mut agents, FlockMode::Gathering);
        }
        
        // 自動モード切り替え (15秒ごと)
        if flock_mode_timer > 15.0 {
            flock_mode_timer = 0.0;
            let mode_index = (get_time() as i32) % 4;
            match mode_index {
                0 => set_follow_mode(&mut agents),
                1 => set_formation_mode(&mut agents, FormationType::VFormation),
                2 => set_formation_mode(&mut agents, FormationType::Circle),
                _ => set_all_agents_mode(&mut agents, FlockMode::Gathering),
            }
        }
        
        // Update camera rotation
        camera_angle += 0.3 * delta_time;
        camera.position = Vec3::new(
            camera_angle.sin() * camera_radius,
            8.0,
            camera_angle.cos() * camera_radius,
        );
        camera.target = Vec3::ZERO;

        // Update AI agents (群れ行動対応)
        // 借用問題を避けるため、2段階でupdate
        let agents_snapshot: Vec<AIAgent> = agents.clone();
        for i in 0..agents.len() {
            agents[i].update(delta_time, &agents_snapshot);
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
        draw_text("Agentira Prototype - Flocking AI Agents 🤖", 20.0, 30.0, 20.0, DARKGRAY);
        draw_text(&format!("FPS: {:.0}", get_fps()), 20.0, 50.0, 16.0, DARKGRAY);
        
        // 群れ行動制御UI
        draw_text("🎮 Flock Controls:", 20.0, 80.0, 18.0, BLUE);
        draw_text("[1] Wandering  [2] Follow Leader  [3] V-Formation  [4] Circle  [5] Gather", 20.0, 100.0, 14.0, DARKGRAY);
        draw_text("Auto mode changes every 15 seconds", 20.0, 120.0, 12.0, GRAY);
        
        // エージェント状態表示
        let current_mode = if !agents.is_empty() {
            match agents[0].flock_mode {
                FlockMode::Wandering => "🚶 Wandering",
                FlockMode::Following => "👥 Following Leader",
                FlockMode::Formation => "📐 Formation",
                FlockMode::Gathering => "🎯 Gathering",
            }
        } else {
            "Unknown"
        };
        draw_text(&format!("Current Mode: {}", current_mode), 20.0, 140.0, 16.0, GREEN);
        draw_text(&format!("Mode Timer: {:.1}s", flock_mode_timer), 20.0, 160.0, 14.0, GRAY);

        next_frame().await
    }
}

// 群れ行動制御関数群
fn set_all_agents_mode(agents: &mut [AIAgent], mode: FlockMode) {
    for agent in agents.iter_mut() {
        agent.flock_mode = mode;
        agent.is_leader = false;
        agent.target_position = None;
    }
    
    if matches!(mode, FlockMode::Wandering) {
        // Wanderingモードでは全員がリーダー（自由行動）
        for agent in agents.iter_mut() {
            agent.change_timer = agent.change_interval; // 即座に方向転換
        }
    }
}

fn set_follow_mode(agents: &mut [AIAgent]) {
    for (i, agent) in agents.iter_mut().enumerate() {
        agent.flock_mode = FlockMode::Following;
        agent.is_leader = i == 1; // Scout(index 1)をリーダーに
        agent.target_position = None;
    }
}

fn set_formation_mode(agents: &mut [AIAgent], formation: FormationType) {
    for agent in agents.iter_mut() {
        agent.flock_mode = FlockMode::Formation;
        agent.is_leader = false;
    }
    
    // フォーメーション位置を計算
    let center = Vec3::ZERO;
    let formation_positions = calculate_formation_positions(formation, agents.len());
    
    for (i, agent) in agents.iter_mut().enumerate() {
        if i < formation_positions.len() {
            agent.target_position = Some(center + formation_positions[i]);
            agent.formation_offset = formation_positions[i];
        }
    }
}

fn calculate_formation_positions(formation: FormationType, agent_count: usize) -> Vec<Vec3> {
    let mut positions = Vec::new();
    
    match formation {
        FormationType::VFormation => {
            // V字フォーメーション
            positions.push(Vec3::new(0.0, 0.0, 2.0)); // リーダー
            for i in 1..agent_count {
                let side = if i % 2 == 1 { 1.0 } else { -1.0 };
                let row = (i + 1) / 2;
                positions.push(Vec3::new(
                    side * (row as f32) * 1.5,
                    0.0,
                    2.0 - (row as f32) * 1.0
                ));
            }
        },
        FormationType::Circle => {
            // 円形フォーメーション
            let radius = 3.0;
            for i in 0..agent_count {
                let angle = (i as f32) * 2.0 * PI / (agent_count as f32);
                positions.push(Vec3::new(
                    angle.cos() * radius,
                    0.0,
                    angle.sin() * radius
                ));
            }
        },
        FormationType::Line => {
            // 一列フォーメーション
            for i in 0..agent_count {
                positions.push(Vec3::new(
                    (i as f32 - (agent_count as f32 - 1.0) / 2.0) * 2.0,
                    0.0,
                    0.0
                ));
            }
        }
    }
    
    positions
}