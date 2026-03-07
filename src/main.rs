use macroquad::prelude::*;
use std::f32::consts::PI;

// Additional color constants
const SILVER: Color = Color::new(0.75, 0.75, 0.75, 1.0);

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

// 建物システム
#[derive(Clone, Copy, PartialEq)]
enum BuildingType {
    Mine,         // 採掘場
    Factory,      // 工場  
    Storage,      // 倉庫
    Conveyor,     // ベルトコンベア
}

#[derive(Clone)]
struct Building {
    position: Vec3,
    building_type: BuildingType,
    health: f32,
    progress: f32,        // 建設進捗 (0.0-1.0)
    is_operational: bool, // 稼働中かどうか
    input_storage: f32,   // 入力リソース
    output_storage: f32,  // 出力製品
    worker_assigned: Option<usize>, // 担当ワーカーID
}

// リソースタイプ  
#[derive(Clone, Copy, PartialEq)]
enum ResourceType {
    RawOre,       // 原鉱石
    ProcessedMetal, // 加工金属
    Energy,       // エネルギー
    Component,    // 部品
}

#[derive(Clone)]
struct Resource {
    position: Vec3,
    resource_type: ResourceType,
    amount: f32,
    is_being_collected: bool,
}

// エージェントタスク
#[derive(Clone, Copy, PartialEq)]
enum AgentTask {
    Idle,
    Exploring,
    Building(usize),      // 建物ID
    Collecting(usize),    // リソースID
    Transporting,
    Operating(usize),     // 建物稼働
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
    // 工場建設用
    current_task: AgentTask,
    carried_resource: Option<ResourceType>,
    work_efficiency: f32,        // 作業効率
    task_timer: f32,            // タスク実行時間
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
        
        // エージェントタイプごとの作業効率
        let efficiency = match agent_type {
            AgentType::Builder => 1.2,      // 建設得意
            AgentType::Collector => 1.3,    // 収集得意  
            AgentType::Worker => 1.0,       // バランス型
            AgentType::Scout => 0.8,        // 探索特化
            AgentType::Guardian => 0.9,     // 防御特化
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
            // 工場建設初期化
            current_task: AgentTask::Idle,
            carried_resource: None,
            work_efficiency: efficiency,
            task_timer: 0.0,
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
        // タスクベース移動を優先
        if !matches!(self.current_task, AgentTask::Idle) {
            if let Some(target) = self.target_position {
                let to_target = (target - self.position);
                if to_target.length() > 0.5 {
                    return to_target.normalize();
                }
            }
        }
        
        // 通常の群れ行動
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
    
    // 建設システム初期化
    let mut buildings: Vec<Building> = Vec::new();
    let mut resources: Vec<Resource> = Vec::new();
    let mut construction_mode = false;
    let mut resource_spawn_timer = 0.0;
    
    // 初期リソースを配置
    spawn_initial_resources(&mut resources);

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
        
        // 建設システム制御
        if is_key_pressed(KeyCode::B) {
            construction_mode = !construction_mode;
            if construction_mode {
                assign_construction_tasks(&mut agents, &mut buildings, &resources);
            } else {
                reset_all_tasks(&mut agents);
            }
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

        // リソース自動生成
        resource_spawn_timer += delta_time;
        if resource_spawn_timer > 5.0 {
            resource_spawn_timer = 0.0;
            spawn_random_resource(&mut resources);
        }
        
        // 建設システム更新
        if construction_mode {
            update_buildings(&mut buildings, delta_time);
            update_agent_tasks(&mut agents, &buildings, &resources, delta_time);
        }
        
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

        // Draw resources (光るエフェクト)
        for resource in &resources {
            draw_resource(resource, get_time() as f32);
        }
        
        // Draw buildings
        for building in &buildings {
            draw_building(building);
        }
        
        // Draw AI agents
        for agent in &agents {
            agent.draw();
            
            // タスク状態の可視化
            if construction_mode {
                draw_agent_task_indicator(agent);
            }
        }

        // Reset camera for UI
        set_default_camera();

        // Draw UI
        draw_text("Agentira Prototype - Factory Automation 🏭", 20.0, 30.0, 20.0, DARKGRAY);
        draw_text(&format!("FPS: {:.0}", get_fps()), 20.0, 50.0, 16.0, DARKGRAY);
        
        // 群れ行動制御UI
        draw_text("🎮 Flock Controls:", 20.0, 80.0, 16.0, BLUE);
        draw_text("[1] Wandering  [2] Follow  [3] V-Form  [4] Circle  [5] Gather", 20.0, 100.0, 12.0, DARKGRAY);
        
        // 建設システムUI
        let construction_status = if construction_mode { "🏗️ BUILDING" } else { "🚶 ROAMING" };
        let construction_color = if construction_mode { ORANGE } else { GREEN };
        
        draw_text("🏭 Factory System:", 20.0, 130.0, 16.0, ORANGE);
        draw_text("[B] Toggle Building Mode", 20.0, 150.0, 12.0, DARKGRAY);
        draw_text(&format!("Status: {}", construction_status), 20.0, 170.0, 14.0, construction_color);
        
        // 統計表示
        draw_text(&format!("Buildings: {}  Resources: {}", buildings.len(), resources.len()), 20.0, 190.0, 12.0, GRAY);
        
        // エージェント状態
        if construction_mode {
            let mut task_counts = [0; 6]; // Idle, Exploring, Building, Collecting, Transporting, Operating
            for agent in &agents {
                match agent.current_task {
                    AgentTask::Idle => task_counts[0] += 1,
                    AgentTask::Exploring => task_counts[1] += 1,
                    AgentTask::Building(_) => task_counts[2] += 1,
                    AgentTask::Collecting(_) => task_counts[3] += 1,
                    AgentTask::Transporting => task_counts[4] += 1,
                    AgentTask::Operating(_) => task_counts[5] += 1,
                }
            }
            draw_text(&format!("Tasks: Idle:{} Build:{} Collect:{} Transport:{}", 
                task_counts[0], task_counts[2], task_counts[3], task_counts[4]), 20.0, 210.0, 11.0, BLUE);
        } else {
            let current_mode = if !agents.is_empty() {
                match agents[0].flock_mode {
                    FlockMode::Wandering => "🚶 Wandering",
                    FlockMode::Following => "👥 Following Leader",
                    FlockMode::Formation => "📐 Formation",
                    FlockMode::Gathering => "🎯 Gathering",
                }
            } else { "Unknown" };
            draw_text(&format!("Flock Mode: {}", current_mode), 20.0, 210.0, 12.0, GREEN);
        }

        next_frame().await
    }
}

// 建設システム関数群
fn spawn_initial_resources(resources: &mut Vec<Resource>) {
    // 初期リソースをランダム配置
    for _ in 0..8 {
        spawn_random_resource(resources);
    }
}

fn spawn_random_resource(resources: &mut Vec<Resource>) {
    let x = rand::gen_range(-8.0, 8.0);
    let z = rand::gen_range(-8.0, 8.0);
    let resource_type = match rand::gen_range(0, 3) {
        0 => ResourceType::RawOre,
        1 => ResourceType::Energy, 
        _ => ResourceType::Component,
    };
    
    resources.push(Resource {
        position: Vec3::new(x, 0.5, z),
        resource_type,
        amount: 100.0,
        is_being_collected: false,
    });
}

fn assign_construction_tasks(agents: &mut Vec<AIAgent>, buildings: &mut Vec<Building>, resources: &Vec<Resource>) {
    // エージェントタイプに応じてタスクを割り当て
    for agent in agents.iter_mut() {
        agent.current_task = match agent.agent_type {
            AgentType::Scout => AgentTask::Exploring,
            AgentType::Builder => {
                // 建設すべき場所を探す
                if buildings.len() < 3 {
                    create_construction_site(buildings, agents.len());
                    AgentTask::Building(buildings.len() - 1)
                } else {
                    AgentTask::Idle
                }
            },
            AgentType::Collector => {
                // 最寄りのリソースを探す
                if let Some((i, _)) = find_nearest_resource(&agent.position, resources) {
                    AgentTask::Collecting(i)
                } else {
                    AgentTask::Exploring
                }
            },
            AgentType::Worker => {
                // 稼働可能な建物を探す
                if let Some((i, _)) = find_operational_building(buildings) {
                    AgentTask::Operating(i)
                } else {
                    AgentTask::Idle
                }
            },
            AgentType::Guardian => AgentTask::Exploring, // パトロール
        };
    }
}

fn create_construction_site(buildings: &mut Vec<Building>, building_id: usize) {
    let x = rand::gen_range(-6.0, 6.0);
    let z = rand::gen_range(-6.0, 6.0);
    let building_type = match building_id % 3 {
        0 => BuildingType::Mine,
        1 => BuildingType::Factory,
        _ => BuildingType::Storage,
    };
    
    buildings.push(Building {
        position: Vec3::new(x, 0.0, z),
        building_type,
        health: 100.0,
        progress: 0.0,
        is_operational: false,
        input_storage: 0.0,
        output_storage: 0.0,
        worker_assigned: None,
    });
}

fn find_nearest_resource(pos: &Vec3, resources: &Vec<Resource>) -> Option<(usize, f32)> {
    resources.iter().enumerate()
        .filter(|(_, r)| !r.is_being_collected && r.amount > 0.0)
        .map(|(i, r)| (i, (pos - &r.position).length()))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
}

fn find_operational_building(buildings: &Vec<Building>) -> Option<(usize, &Building)> {
    buildings.iter().enumerate()
        .find(|(_, b)| b.is_operational && b.worker_assigned.is_none())
}

fn reset_all_tasks(agents: &mut Vec<AIAgent>) {
    for agent in agents.iter_mut() {
        agent.current_task = AgentTask::Idle;
        agent.carried_resource = None;
        agent.task_timer = 0.0;
    }
}

fn update_buildings(buildings: &mut Vec<Building>, delta_time: f32) {
    for building in buildings.iter_mut() {
        // 建設進捗
        if building.progress < 1.0 && building.progress > 0.0 {
            building.progress += delta_time * 0.1; // 建設速度
            if building.progress >= 1.0 {
                building.progress = 1.0;
                building.is_operational = true;
            }
        }
        
        // 稼働中建物の生産
        if building.is_operational && building.input_storage > 0.0 {
            match building.building_type {
                BuildingType::Mine => {
                    building.output_storage += delta_time * 10.0; // 採掘速度
                },
                BuildingType::Factory => {
                    if building.input_storage >= 10.0 {
                        building.input_storage -= 10.0;
                        building.output_storage += 5.0;
                    }
                },
                _ => {}
            }
        }
    }
}

fn update_agent_tasks(agents: &mut Vec<AIAgent>, buildings: &mut Vec<Building>, resources: &Vec<Resource>, delta_time: f32) {
    for i in 0..agents.len() {
        let agent = &mut agents[i];
        agent.task_timer += delta_time;
        
        match agent.current_task {
            AgentTask::Building(building_id) => {
                if building_id < buildings.len() {
                    let target_pos = buildings[building_id].position;
                    let distance = (agent.position - target_pos).length();
                    
                    if distance < 2.0 {
                        // 建設作業
                        buildings[building_id].progress += delta_time * agent.work_efficiency * 0.2;
                        agent.target_position = Some(target_pos);
                    } else {
                        // 建設場所に移動
                        agent.target_position = Some(target_pos);
                    }
                }
            },
            AgentTask::Collecting(resource_id) => {
                if resource_id < resources.len() {
                    let target_pos = resources[resource_id].position;
                    let distance = (agent.position - target_pos).length();
                    
                    if distance < 1.5 {
                        // リソース収集
                        if agent.task_timer > 2.0 {
                            agent.carried_resource = Some(resources[resource_id].resource_type);
                            agent.current_task = AgentTask::Transporting;
                            agent.task_timer = 0.0;
                        }
                    } else {
                        agent.target_position = Some(target_pos);
                    }
                }
            },
            AgentTask::Transporting => {
                // 最寄りの建物に運搬
                if let Some((building_id, _)) = find_operational_building(buildings) {
                    let target_pos = buildings[building_id].position;
                    let distance = (agent.position - target_pos).length();
                    
                    if distance < 2.0 && agent.task_timer > 1.0 {
                        buildings[building_id].input_storage += 10.0;
                        agent.carried_resource = None;
                        agent.current_task = AgentTask::Idle;
                    } else {
                        agent.target_position = Some(target_pos);
                    }
                } else {
                    agent.current_task = AgentTask::Idle;
                }
            },
            AgentTask::Operating(building_id) => {
                if building_id < buildings.len() {
                    let target_pos = buildings[building_id].position + Vec3::new(0.0, 0.0, 1.5);
                    agent.target_position = Some(target_pos);
                    buildings[building_id].worker_assigned = Some(i);
                }
            },
            _ => {}
        }
    }
}

fn draw_resource(resource: &Resource, time: f32) {
    let bob = (time * 2.0).sin() * 0.1;
    let pos = resource.position + Vec3::new(0.0, bob, 0.0);
    
    let color = match resource.resource_type {
        ResourceType::RawOre => GOLD,
        ResourceType::Energy => SKYBLUE,
        ResourceType::Component => MAGENTA,
        ResourceType::ProcessedMetal => SILVER,
    };
    
    // 光るエフェクト（サイズ変化）
    let pulse = (time * 4.0).sin().abs() * 0.2 + 0.8;
    draw_cube(pos, Vec3::new(0.4 * pulse, 0.4 * pulse, 0.4 * pulse), None, color);
    
    // 光る外周
    draw_cube(pos, Vec3::new(0.6 * pulse, 0.6 * pulse, 0.6 * pulse), None, 
              Color::new(color.r, color.g, color.b, 0.3));
}

fn draw_building(building: &Building) {
    let pos = building.position;
    let progress = building.progress;
    
    // 建設進捗に応じた高さ
    let height = 1.0 + progress * 1.5;
    
    let (size, color) = match building.building_type {
        BuildingType::Mine => (Vec3::new(2.0, height, 2.0), BROWN),
        BuildingType::Factory => (Vec3::new(3.0, height, 2.5), DARKGRAY),
        BuildingType::Storage => (Vec3::new(2.5, height, 2.5), BLUE),
        BuildingType::Conveyor => (Vec3::new(1.0, 0.2, 3.0), YELLOW),
    };
    
    // メインの建物
    draw_cube(pos + Vec3::new(0.0, height / 2.0, 0.0), size, None, color);
    
    // 進捗インジケーター
    if progress < 1.0 {
        // 建設中の枠組み
        draw_cube(pos + Vec3::new(0.0, height / 2.0, 0.0), size + Vec3::new(0.1, 0.1, 0.1), None, 
                  Color::new(1.0, 1.0, 1.0, 0.3));
    } else {
        // 完成時のディテール
        match building.building_type {
            BuildingType::Mine => {
                // 採掘装置
                draw_cube(pos + Vec3::new(0.0, height + 0.3, 0.0), Vec3::new(0.5, 0.6, 0.5), None, ORANGE);
            },
            BuildingType::Factory => {
                // 煙突
                draw_cube(pos + Vec3::new(0.8, height + 0.5, 0.0), Vec3::new(0.3, 1.0, 0.3), None, DARKGRAY);
                // 煙エフェクト
                if building.is_operational {
                    draw_cube(pos + Vec3::new(0.8, height + 1.2, 0.0), Vec3::new(0.2, 0.3, 0.2), None, GRAY);
                }
            },
            BuildingType::Storage => {
                // ドア
                draw_cube(pos + Vec3::new(0.0, 0.5, 1.3), Vec3::new(0.8, 1.0, 0.1), None, BROWN);
            },
            _ => {}
        }
    }
    
    // ストレージ状況表示
    if building.is_operational {
        let input_bar = building.input_storage / 100.0;
        let output_bar = building.output_storage / 100.0;
        
        if input_bar > 0.0 {
            draw_cube(pos + Vec3::new(-1.0, height + 0.2, 0.0), 
                      Vec3::new(0.1, input_bar * 0.5, 0.1), None, GREEN);
        }
        if output_bar > 0.0 {
            draw_cube(pos + Vec3::new(1.0, height + 0.2, 0.0), 
                      Vec3::new(0.1, output_bar * 0.5, 0.1), None, BLUE);
        }
    }
}

fn draw_agent_task_indicator(agent: &AIAgent) {
    let indicator_pos = agent.position + Vec3::new(0.0, 2.5, 0.0);
    let indicator_color = match agent.current_task {
        AgentTask::Idle => GRAY,
        AgentTask::Exploring => GREEN,
        AgentTask::Building(_) => ORANGE,
        AgentTask::Collecting(_) => GOLD,
        AgentTask::Transporting => BLUE,
        AgentTask::Operating(_) => PURPLE,
    };
    
    draw_cube(indicator_pos, Vec3::new(0.2, 0.2, 0.2), None, indicator_color);
    
    // 運搬中リソースの表示
    if let Some(resource_type) = agent.carried_resource {
        let resource_color = match resource_type {
            ResourceType::RawOre => GOLD,
            ResourceType::Energy => SKYBLUE,
            ResourceType::Component => MAGENTA,
            ResourceType::ProcessedMetal => SILVER,
        };
        draw_cube(agent.position + Vec3::new(0.0, 1.8, 0.0), 
                  Vec3::new(0.3, 0.3, 0.3), None, resource_color);
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