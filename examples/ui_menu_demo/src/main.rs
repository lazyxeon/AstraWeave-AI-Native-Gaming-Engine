/// # UI Menu Demo
///
/// Demonstrates the AstraWeave in-game UI system with main menu, pause menu, HUD, quest tracker, and minimap.
///
/// ## Controls:
/// - Click buttons with mouse to navigate menus
/// - Press ESC to toggle pause menu (when in-game)
/// - Press TAB to cycle through buttons (keyboard navigation)
/// - Press ENTER to activate focused button
/// - Press F3 to toggle HUD debug mode
/// - Press 1/2/3 to spawn damage numbers (when in-game)
/// - Press Q to toggle quest tracker visibility
/// - Press M to toggle minimap visibility
/// - Press C to collapse/expand quest tracker
/// - Press R to toggle minimap rotation (north-up vs player-relative)
/// - Press T to toggle dialogue demo
/// - Press H to heal player (health animation demo)
/// - Press D to damage player (health animation demo)
/// - Press N to trigger "New Quest!" notification (Week 4 Day 3)
/// - Press O to trigger "Objective Complete!" notification (Week 4 Day 3)
/// - Press P to trigger "Quest Complete!" notification (Week 4 Day 3)
/// - Press +/= to zoom in on minimap (Week 4 Day 4)
/// - Press -/_ to zoom out on minimap (Week 4 Day 4)
/// - Click minimap to spawn ping marker (Week 5 Day 1)
/// - "New Game" button starts the game (shows pause capability)
/// - "Quit" button exits the application
///
/// ## Architecture:
/// This example integrates:
/// - winit 0.30 for window management and event handling
/// - wgpu 25 for rendering backend
/// - astraweave-ui for menu system (UiLayer + MenuManager + HudManager)
///
/// The rendering flow:
/// 1. Clear background with solid color (simulated 3D scene)
/// 2. Render UI overlay using egui-wgpu (LoadOp::Load)
/// 3. Present to screen
///
/// ## Week 3 Progress:
/// - Day 1: HUD framework with F3 debug toggle
/// - Day 2: Health bars, resource meters, damage numbers (animated floating text)
/// - Day 3: Quest tracker (collapsible panel) and minimap (circular, with POI markers)
/// - Added keyboard navigation (TAB cycling)
/// - Added visual focus indicators
/// - Improved button styling and animations
use anyhow::Result;
use astraweave_ui::{
    DamageType, EnemyData, EnemyFaction, HudManager, MenuAction, MenuManager, UiLayer,
    // Week 3 Day 3: Quest tracker & minimap
    Quest, Objective, PoiMarker, PoiType,
};
use log::{error, info, warn};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

// Week 5 Day 2: Optional audio support (demonstration only)
#[cfg(feature = "audio")]
use astraweave_audio::AudioEngine;

/// Application state
struct App {
    /// UI rendering layer (egui + egui-wgpu)
    ui_layer: Option<UiLayer>,
    /// Menu state machine
    menu_manager: MenuManager,
    /// HUD manager (Week 3: in-game overlay)
    hud_manager: HudManager,
    /// WGPU device (GPU handle)
    device: Option<Arc<wgpu::Device>>,
    /// WGPU queue (command submission)
    queue: Option<Arc<wgpu::Queue>>,
    /// WGPU surface (window target)
    surface: Option<wgpu::Surface<'static>>,
    /// Surface configuration
    config: Option<wgpu::SurfaceConfiguration>,
    /// Window handle
    window: Option<Arc<Window>>,
    /// Game state flag (true = in-game, false = main menu)
    in_game: bool,
    /// Exit flag
    should_exit: bool,
    /// Frame timing (for FPS display)
    last_frame_time: std::time::Instant,
    frame_count: u32,
    fps: f32,
    /// Demo data: Mock enemies for health bar visualization (Week 3 Day 2)
    demo_enemies: Vec<EnemyData>,
    /// Demo data: Simulation time for animations
    demo_time: f32,
    /// Mouse position (for tooltips - Week 3 Day 5)
    mouse_position: (f32, f32),
    /// Week 5 Day 2: Optional audio engine for minimap click/ping sounds
    #[cfg(feature = "audio")]
    audio_engine: Option<AudioEngine>,
}

impl Default for App {
    fn default() -> Self {
        // Initialize HUD manager with demo quest and POI markers
        let mut hud_manager = HudManager::new();
        
        // Week 3 Day 3: Demo quest
        hud_manager.active_quest = Some(Quest {
            id: 1,
            title: "Gather the Ancient Shards".to_string(),
            description: "Collect the scattered Crystal Shards from the ruins".to_string(),
            objectives: vec![
                Objective {
                    id: 1,
                    description: "Collect Crystal Shards".to_string(),
                    completed: false,
                    progress: Some((3, 5)),  // 3 out of 5 collected
                },
                Objective {
                    id: 2,
                    description: "Defeat the Guardian".to_string(),
                    completed: false,
                    progress: None,
                },
                Objective {
                    id: 3,
                    description: "Return to the Temple".to_string(),
                    completed: false,
                    progress: None,
                },
            ],
        });
        
        // Week 3 Day 3: POI markers for minimap
        hud_manager.poi_markers = vec![
            PoiMarker {
                id: 1,
                world_pos: (10.0, 5.0),  // 2D (X, Z)
                poi_type: PoiType::Objective,
                label: Some("Shard Location".to_string()),
            },
            PoiMarker {
                id: 2,
                world_pos: (-8.0, 12.0),
                poi_type: PoiType::Waypoint,
                label: Some("Checkpoint".to_string()),
            },
            PoiMarker {
                id: 3,
                world_pos: (15.0, -7.0),
                poi_type: PoiType::Vendor,
                label: Some("Shop".to_string()),
            },
            PoiMarker {
                id: 4,
                world_pos: (-12.0, -5.0),
                poi_type: PoiType::Danger,
                label: Some("Guardian Lair".to_string()),
            },
        ];
        
        // Week 3 Day 3: Player position and rotation for minimap
        hud_manager.player_position = (0.0, 0.0);  // Center of map
        hud_manager.player_rotation = 0.0;  // Facing north
        
        Self {
            ui_layer: None,
            menu_manager: MenuManager::new(),
            hud_manager,
            device: None,
            queue: None,
            surface: None,
            config: None,
            window: None,
            in_game: false,
            should_exit: false,
            last_frame_time: std::time::Instant::now(),
            frame_count: 0,
            fps: 0.0,
            demo_enemies: vec![
                // Enemy 1: Hostile with 75% health
                {
                    let mut enemy = EnemyData::new(1, (5.0, 2.0, 0.0), 100.0, EnemyFaction::Hostile);
                    enemy.health = 75.0;  // Start damaged
                    enemy
                },
                // Enemy 2: Neutral with 50% health
                {
                    let mut enemy = EnemyData::new(2, (-5.0, 1.5, 0.0), 100.0, EnemyFaction::Neutral);
                    enemy.health = 50.0;  // Start damaged
                    enemy
                },
                // Enemy 3: Friendly with 90% health
                {
                    let mut enemy = EnemyData::new(3, (0.0, 3.0, 0.0), 100.0, EnemyFaction::Friendly);
                    enemy.health = 90.0;  // Start damaged
                    enemy
                },
            ],
            demo_time: 0.0,
            mouse_position: (0.0, 0.0),
            #[cfg(feature = "audio")]
            audio_engine: {
                // Week 5 Day 2: Initialize audio engine for minimap click/ping sounds
                match AudioEngine::new() {
                    Ok(engine) => {
                        info!("Audio engine initialized for minimap sounds");
                        Some(engine)
                    }
                    Err(e) => {
                        warn!("Failed to initialize audio engine: {}", e);
                        None
                    }
                }
            },
        }
    }
}

impl App {
    /// Start a demo dialogue with branching choices
    fn start_demo_dialogue(&mut self) {
        use astraweave_ui::{DialogueNode, DialogueChoice};
        
        let first_node = DialogueNode {
            id: 1,
            speaker_name: "Mysterious Stranger".to_string(),
            text: "Greetings, traveler. I sense great power within you. The ancient ruins to the north hold secrets that could unlock your true potential... but they are guarded by powerful creatures.".to_string(),
            choices: vec![
                DialogueChoice { id: 0, text: "Tell me more about these ruins".to_string(), next_node: Some(2) },
                DialogueChoice { id: 1, text: "What's in it for me?".to_string(), next_node: Some(3) },
                DialogueChoice { id: 2, text: "I'm not interested".to_string(), next_node: None },
            ],
            portrait_id: None,
        };
        self.hud_manager.start_dialogue(first_node);
        info!("Started demo dialogue (Mysterious Stranger)");
    }

    /// Load a dialogue node by ID (creates branching conversation tree)
    fn load_dialogue_node(&mut self, node_id: u32) {
        use astraweave_ui::{DialogueNode, DialogueChoice};
        
        let node = match node_id {
            2 => DialogueNode {
                id: 2,
                speaker_name: "Mysterious Stranger".to_string(),
                text: "The ruins were built by the Ancients, masters of both magic and technology. Legend says they left behind artifacts of immense power, sealed within crypts guarded by mechanical sentinels.".to_string(),
                choices: vec![
                    DialogueChoice { id: 0, text: "I accept your quest".to_string(), next_node: Some(4) },
                    DialogueChoice { id: 1, text: "Sounds dangerous. What else can you tell me?".to_string(), next_node: Some(3) },
                    DialogueChoice { id: 2, text: "Maybe another time".to_string(), next_node: None },
                ],
                portrait_id: None,
            },
            3 => DialogueNode {
                id: 3,
                speaker_name: "Mysterious Stranger".to_string(),
                text: "Gold, rare gems, enchanted weapons... The Ancients were wealthy beyond measure. But more importantly, you'll gain knowledge lost to time. Knowledge that could change the fate of this world.".to_string(),
                choices: vec![
                    DialogueChoice { id: 0, text: "I'm in. Where do I start?".to_string(), next_node: Some(4) },
                    DialogueChoice { id: 1, text: "Tell me about the ruins again".to_string(), next_node: Some(2) },
                    DialogueChoice { id: 2, text: "This sounds too risky".to_string(), next_node: None },
                ],
                portrait_id: None,
            },
            4 => DialogueNode {
                id: 4,
                speaker_name: "Mysterious Stranger".to_string(),
                text: "Excellent! Head north past the old watchtower. You'll find the entrance hidden behind a waterfall. Take this map - it will guide you through the first chamber. Good luck, traveler. We'll meet again.".to_string(),
                choices: vec![
                    DialogueChoice { id: 0, text: "[Accept the map and depart]".to_string(), next_node: None },
                ],
                portrait_id: None,
            },
            _ => {
                warn!("Unknown dialogue node ID: {}", node_id);
                return;
            }
        };
        
        self.hud_manager.start_dialogue(node);
        info!("Loaded dialogue node {}", node_id);
    }

    /// Initialize WGPU (async setup)
    async fn initialize_wgpu(&mut self, window: Arc<Window>) -> Result<()> {
        let size = window.inner_size();

        // Create WGPU instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // Create surface for the window
        let surface = instance.create_surface(window.clone())?;

        // Request adapter (GPU)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        info!("Using GPU: {}", adapter.get_info().name);

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("UI Demo Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                    trace: Default::default(),
                },
            )
            .await?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo, // VSync
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // Create UI layer
        let ui_layer = UiLayer::new(&window, &device, surface_format);

        info!("UI Menu Demo initialized successfully");

        self.ui_layer = Some(ui_layer);
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.config = Some(config);
        self.window = Some(window);

        // Week 5 Day 2: Setup audio callbacks for minimap interactions
        // Note: Audio callbacks are demonstrated but commented out due to lifetime complexity
        // In production code, use Arc<Mutex<AudioEngine>> or a message-passing approach
        #[cfg(feature = "audio")]
        {
            if self.audio_engine.is_some() {
                info!("Audio engine initialized (callbacks available)");
                
                // Example: Minimap click sound callback
                // self.hud_manager.set_minimap_click_callback(|dist| {
                //     // Pitch varies: 800Hz at center → 1200Hz at edge
                //     let pitch_hz = 800.0 + (dist * 400.0);
                //     // Play beep: audio.play_sfx_beep(pitch_hz, 0.05, 0.3);
                // });
                
                // Example: Ping spawn sound callback (3D spatial audio)
                // self.hud_manager.set_ping_spawn_callback(|world_pos| {
                //     // Play 3D beep at world position
                //     // let pos_3d = vec3(world_pos.0, 0.0, world_pos.1);
                //     // audio.play_sfx_3d_beep(0, pos_3d, 1200.0, 0.1, 0.6);
                // });
            }
        }

        Ok(())
    }

    /// Handle window resize
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if let (Some(config), Some(surface), Some(device)) =
            (&mut self.config, &self.surface, &self.device)
        {
            if new_size.width > 0 && new_size.height > 0 {
                config.width = new_size.width;
                config.height = new_size.height;
                surface.configure(device, config);
                info!("Window resized to {}x{}", new_size.width, new_size.height);
            }
        }
    }

    /// Handle keyboard input
    fn handle_key(&mut self, key: &Key, pressed: bool) {
        if !pressed {
            return;
        }

        // Check if we're rebinding a key
        if let Some(rebinding_key_id) = self.menu_manager.rebinding_key.clone() {
            // Capture the key for rebinding
            let key_name = match key {
                Key::Named(NamedKey::Space) => "Space".to_string(),
                Key::Named(NamedKey::Enter) => "Enter".to_string(),
                Key::Named(NamedKey::Tab) => "Tab".to_string(),
                Key::Named(NamedKey::Shift) => "LShift".to_string(),
                Key::Named(NamedKey::Control) => "LControl".to_string(),
                Key::Named(NamedKey::Alt) => "LAlt".to_string(),
                Key::Named(NamedKey::Escape) => {
                    // ESC cancels rebinding
                    self.menu_manager.rebinding_key = None;
                    info!("Cancelled key rebinding");
                    return;
                }
                Key::Character(c) => c.to_uppercase().to_string(),
                _ => {
                    // Unknown key, cancel rebinding
                    self.menu_manager.rebinding_key = None;
                    return;
                }
            };

            // Update the appropriate key binding
            match rebinding_key_id.as_str() {
                "move_forward" => self.menu_manager.settings.controls.move_forward = key_name.clone(),
                "move_backward" => self.menu_manager.settings.controls.move_backward = key_name.clone(),
                "move_left" => self.menu_manager.settings.controls.move_left = key_name.clone(),
                "move_right" => self.menu_manager.settings.controls.move_right = key_name.clone(),
                "jump" => self.menu_manager.settings.controls.jump = key_name.clone(),
                "crouch" => self.menu_manager.settings.controls.crouch = key_name.clone(),
                "sprint" => self.menu_manager.settings.controls.sprint = key_name.clone(),
                "attack" => self.menu_manager.settings.controls.attack = key_name.clone(),
                "interact" => self.menu_manager.settings.controls.interact = key_name.clone(),
                "inventory" => self.menu_manager.settings.controls.inventory = key_name.clone(),
                _ => {}
            }

            info!("Rebound {} to {}", rebinding_key_id, key_name);
            self.menu_manager.rebinding_key = None;
            return;
        }

        match key {
            Key::Named(NamedKey::Escape) => {
                if self.in_game {
                    // ESC toggles pause menu when in-game
                    // (HUD visibility managed separately - stays visible even when paused)
                    self.menu_manager.toggle_pause();
                    info!("Toggled pause menu");
                }
            }
            Key::Named(NamedKey::F3) => {
                // F3 toggles HUD debug mode (shows element bounds and stats)
                self.hud_manager.toggle_debug();
            }
            Key::Named(NamedKey::Enter) => {
                // ENTER can activate focused button (egui handles this internally)
                info!("Enter key pressed");
            }
            Key::Character(c) => {
                // Week 3 Day 2: Damage number demo (keys 1/2/3)
                // Week 3 Day 3: Quest tracker/minimap toggle (keys Q/M/C/R)
                // Week 3 Day 4: Dialogue system (key T for trigger, 1-4 for choices when in dialogue)
                if self.in_game && !self.menu_manager.is_menu_visible() {
                    // Handle dialogue choices first (when dialogue is active)
                    if self.hud_manager.active_dialogue.is_some() {
                        match c.as_str() {
                            "1" | "2" | "3" | "4" => {
                                let choice_id = c.parse::<u32>().unwrap_or(0);
                                if let Some(next_node_id) = self.hud_manager.select_dialogue_choice(choice_id - 1) {
                                    // Load next dialogue node
                                    self.load_dialogue_node(next_node_id);
                                } else {
                                    // End dialogue
                                    self.hud_manager.end_dialogue();
                                }
                                return;  // Don't process other key handlers
                            }
                            _ => {}
                        }
                    }
                    
                    // Normal game controls
                    match c.as_str() {
                        "1" if self.hud_manager.active_dialogue.is_none() => {
                            // Normal damage on enemy 1 (hostile) - only if not in dialogue
                            if !self.demo_enemies.is_empty() {
                                let pos = self.demo_enemies[0].world_pos;
                                self.hud_manager.spawn_damage(25, pos, DamageType::Normal);
                                info!("Spawned normal damage (25) on enemy 1");
                            }
                        }
                        "2" if self.hud_manager.active_dialogue.is_none() => {
                            // Critical damage on enemy 2 (neutral) - only if not in dialogue
                            if self.demo_enemies.len() > 1 {
                                let pos = self.demo_enemies[1].world_pos;
                                self.hud_manager.spawn_damage(50, pos, DamageType::Critical);
                                info!("Spawned critical damage (50) on enemy 2");
                            }
                        }
                        "3" if self.hud_manager.active_dialogue.is_none() => {
                            // Self-damage at player position (center) - only if not in dialogue
                            self.hud_manager.spawn_damage(10, (0.0, 0.5, 0.0), DamageType::SelfDamage);
                            info!("Spawned self-damage (10)");
                        }
                        "q" | "Q" => {
                            // Toggle quest tracker visibility
                            self.hud_manager.toggle_quest_tracker();
                        }
                        "m" | "M" => {
                            // Toggle minimap visibility
                            self.hud_manager.toggle_minimap();
                        }
                        "c" | "C" => {
                            // Toggle quest tracker collapse/expand
                            self.hud_manager.toggle_quest_collapse();
                        }
                        "r" | "R" => {
                            // Toggle minimap rotation mode (north-up vs player-relative)
                            self.hud_manager.toggle_minimap_rotation();
                        }
                        "t" | "T" => {
                            // Toggle dialogue demo (Week 3 Day 4)
                            if self.hud_manager.active_dialogue.is_some() {
                                self.hud_manager.end_dialogue();
                            } else {
                                self.start_demo_dialogue();
                            }
                        }
                        "h" | "H" => {
                            // Week 4 Day 1: Heal player (trigger health increase animation)
                            self.hud_manager.player_stats.health = 
                                (self.hud_manager.player_stats.health + 20.0).min(self.hud_manager.player_stats.max_health);
                            info!("Player healed +20 HP (current: {:.0}/{:.0})", 
                                self.hud_manager.player_stats.health,
                                self.hud_manager.player_stats.max_health);
                        }
                        "d" | "D" => {
                            // Week 4 Day 1: Damage player (trigger health decrease animation + flash)
                            self.hud_manager.player_stats.health = 
                                (self.hud_manager.player_stats.health - 15.0).max(0.0);
                            info!("Player took 15 damage (current: {:.0}/{:.0})", 
                                self.hud_manager.player_stats.health,
                                self.hud_manager.player_stats.max_health);
                        }
                        "n" | "N" => {
                            // Week 4 Day 3: Trigger "New Quest!" notification
                            use astraweave_ui::hud::QuestNotification;
                            let notification = QuestNotification::new_quest(
                                "The Lost Artifact".to_string(),
                                "Find the ancient relic in the ruins".to_string(),
                            );
                            self.hud_manager.notification_queue.push(notification);
                            info!("Triggered 'New Quest!' notification");
                        }
                        "o" | "O" => {
                            // Week 4 Day 3: Trigger "Objective Complete!" notification
                            use astraweave_ui::hud::QuestNotification;
                            let notification = QuestNotification::objective_complete(
                                "Defeat 10 enemies".to_string(),
                            );
                            self.hud_manager.notification_queue.push(notification);
                            info!("Triggered 'Objective Complete!' notification");
                        }
                        "p" | "P" => {
                            // Week 4 Day 3: Trigger "Quest Complete!" notification with rewards
                            use astraweave_ui::hud::QuestNotification;
                            let rewards = vec![
                                "500 Gold".to_string(),
                                "Legendary Sword".to_string(),
                                "Achievement: Hero".to_string(),
                            ];
                            let notification = QuestNotification::quest_complete(
                                "The Lost Artifact".to_string(),
                                rewards,
                            );
                            self.hud_manager.notification_queue.push(notification);
                            info!("Triggered 'Quest Complete!' notification");
                        }
                        "+" | "=" => {
                            // Week 4 Day 4: Zoom in on minimap
                            let new_zoom = (self.hud_manager.minimap_zoom() + 0.25).min(3.0);
                            self.hud_manager.set_minimap_zoom(new_zoom);
                        }
                        "-" | "_" => {
                            // Week 4 Day 4: Zoom out on minimap
                            let new_zoom = (self.hud_manager.minimap_zoom() - 0.25).max(0.5);
                            self.hud_manager.set_minimap_zoom(new_zoom);
                        }
                        // Week 5 Day 1: Removed G key ping spawn - now click minimap directly
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    /// Update and render a frame
    fn render(&mut self) -> Result<()> {
        // Update FPS counter and demo time
        let now = std::time::Instant::now();
        let delta = now.duration_since(self.last_frame_time).as_secs_f32();
        self.frame_count += 1;
        
        // Update demo time for animations
        self.demo_time += delta;
        
        // Update FPS every 30 frames
        if self.frame_count >= 30 {
            self.fps = self.frame_count as f32 / delta;
            self.last_frame_time = now;
            self.frame_count = 0;
        }
        
        // Update HUD (damage number animations, etc.)
        self.hud_manager.update(delta);
        
        // Sync demo enemies to HUD
        self.hud_manager.enemies = self.demo_enemies.clone();

        let window = self.window.as_ref().ok_or_else(|| anyhow::anyhow!("No window"))?;
        let surface = self.surface.as_ref().ok_or_else(|| anyhow::anyhow!("No surface"))?;
        let device = self.device.as_ref().ok_or_else(|| anyhow::anyhow!("No device"))?;
        let queue = self.queue.as_ref().ok_or_else(|| anyhow::anyhow!("No queue"))?;
        let config = self.config.as_ref().ok_or_else(|| anyhow::anyhow!("No config"))?;
        let ui_layer = self.ui_layer.as_mut().ok_or_else(|| anyhow::anyhow!("No UI layer"))?;

        // Get next framebuffer
        let output = match surface.get_current_texture() {
            Ok(output) => output,
            Err(wgpu::SurfaceError::Lost) => {
                warn!("Surface lost, reconfiguring...");
                surface.configure(device, config);
                return Ok(());
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                error!("Out of memory!");
                self.should_exit = true;
                return Err(anyhow::anyhow!("Out of memory"));
            }
            Err(e) => {
                warn!("Surface error: {:?}", e);
                return Ok(());
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Render "3D scene" background (solid color for this demo)
        {
            let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Background Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(if self.in_game {
                            // Dark blue (simulated game scene)
                            wgpu::Color {
                                r: 0.02,
                                g: 0.02,
                                b: 0.1,
                                a: 1.0,
                            }
                        } else {
                            // Black (main menu)
                            wgpu::Color::BLACK
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // Render UI overlay
        ui_layer.begin(window);

        // Display FPS counter in corner (always visible)
        let ctx = ui_layer.ctx();
        {
            use astraweave_ui::egui;
            egui::Area::new(egui::Id::new("fps_counter"))
                .fixed_pos(egui::pos2(10.0, 10.0))
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new(format!("FPS: {:.1}", self.fps))
                            .size(16.0)
                            .color(egui::Color32::from_rgb(200, 200, 200)),
                    );
                });
        }
        
        // Render HUD overlay (Week 3: in-game UI elements)
        // Only render when in-game (not in menus)
        if self.in_game && !self.menu_manager.is_menu_visible() {
            self.hud_manager.render(ctx);
            
            // Week 3 Day 5: Demo tooltips for UI elements
            // Show tooltip when hovering over top-right corner (minimap region approximation)
            if self.hud_manager.state().show_minimap {
                let screen_width = config.width as f32;
                let minimap_x = screen_width - 220.0;  // Minimap is 200px + 20px margin
                let minimap_y = 20.0;
                let minimap_size = 200.0;
                
                // Check if mouse is in minimap region
                if self.mouse_position.0 >= minimap_x 
                    && self.mouse_position.0 <= minimap_x + minimap_size
                    && self.mouse_position.1 >= minimap_y
                    && self.mouse_position.1 <= minimap_y + minimap_size 
                {
                    use astraweave_ui::TooltipData;
                    
                    let rotation_mode = if self.hud_manager.state().minimap_rotation { 
                        "Player-Relative" 
                    } else { 
                        "North-Up" 
                    };
                    
                    let tooltip = TooltipData {
                        title: "Minimap".to_string(),
                        description: "Shows nearby area and points of interest. Press M to toggle visibility, R to change rotation mode.".to_string(),
                        stats: vec![
                            ("Rotation".to_string(), rotation_mode.to_string()),
                            ("POI Markers".to_string(), "3 visible".to_string()),
                        ],
                        flavor_text: Some("The ancient cartographers would be jealous.".to_string()),
                    };
                    
                    self.hud_manager.show_tooltip(tooltip, self.mouse_position);
                } else {
                    // Hide tooltip when not hovering
                    self.hud_manager.hide_tooltip();
                }
            }
            
            // Show tooltip when hovering over quest tracker region (left side)
            if self.hud_manager.state().show_objectives && self.hud_manager.hovered_tooltip.is_none() {
                let quest_x = 20.0;
                let quest_y = 60.0;
                let quest_width = 350.0;
                let quest_height = 200.0;  // Approximate
                
                if self.mouse_position.0 >= quest_x
                    && self.mouse_position.0 <= quest_x + quest_width
                    && self.mouse_position.1 >= quest_y
                    && self.mouse_position.1 <= quest_y + quest_height
                {
                    use astraweave_ui::TooltipData;
                    
                    let tooltip = TooltipData {
                        title: "Active Quest".to_string(),
                        description: "Track your current quest objectives. Press Q to toggle, C to collapse/expand.".to_string(),
                        stats: vec![
                            ("Progress".to_string(), "1/2 objectives".to_string()),
                            ("Reward".to_string(), "500 XP, Ancient Amulet".to_string()),
                        ],
                        flavor_text: None,
                    };
                    
                    self.hud_manager.show_tooltip(tooltip, self.mouse_position);
                }
            }
        }

        let menu_action = self.menu_manager.show(ctx);

        let size = (config.width, config.height);
        ui_layer.end_and_paint(window, &view, &mut encoder, device, queue, size);

        // Submit commands
        queue.submit(Some(encoder.finish()));
        output.present();

        // Handle menu action
        if menu_action != MenuAction::None {
            info!("Menu action: {:?}", menu_action);
            match menu_action {
                MenuAction::NewGame => {
                    info!("Starting new game...");
                    self.in_game = true;
                    self.menu_manager.handle_action(MenuAction::NewGame);
                }
                MenuAction::LoadGame => {
                    info!("Loading game... (not implemented in demo)");
                }
                MenuAction::SaveGame => {
                    info!("Saving game... (not implemented in demo)");
                }
                MenuAction::Resume => {
                    info!("Resuming game...");
                    self.menu_manager.handle_action(MenuAction::Resume);
                }
                MenuAction::Settings => {
                    info!("Opening settings... (not implemented yet - Week 2)");
                    self.menu_manager.handle_action(MenuAction::Settings);
                }
                MenuAction::ApplySettings => {
                    info!("Applying settings (saving to disk)...");
                    self.menu_manager.handle_action(MenuAction::ApplySettings);
                }
                MenuAction::CancelSettings => {
                    info!("Cancelling settings (reverting changes)...");
                    self.menu_manager.handle_action(MenuAction::CancelSettings);
                }
                MenuAction::Quit => {
                    // Let MenuManager handle quit (context-sensitive: settings->back, pause->main, main->exit)
                    let was_main_menu = self.menu_manager.is_main_menu();
                    self.menu_manager.handle_action(MenuAction::Quit);
                    
                    // Only exit if we were on main menu
                    if was_main_menu {
                        info!("Quitting application...");
                        self.should_exit = true;
                    } else {
                        info!("Returning to previous menu...");
                    }
                }
                MenuAction::None => {}
            }
        }

        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return; // Already initialized
        }

        // Create window
        let window_attributes = Window::default_attributes()
            .with_title("AstraWeave UI Menu Demo")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                error!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        // Initialize WGPU (async)
        if let Err(e) = pollster::block_on(self.initialize_wgpu(window)) {
            error!("Failed to initialize WGPU: {}", e);
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Let UI layer handle events first (consumes mouse/keyboard for UI)
        if let Some(ui_layer) = &mut self.ui_layer {
            if let Some(window) = &self.window {
                if ui_layer.on_event(window, &event) {
                    return; // Event consumed by UI
                }
            }
        }

        // Handle application events
        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                self.resize(physical_size);
            }
            WindowEvent::CursorMoved { position, .. } => {
                // Track mouse position for tooltips (Week 3 Day 5)
                self.mouse_position = (position.x as f32, position.y as f32);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_key(&event.logical_key, event.state.is_pressed());
            }
            WindowEvent::RedrawRequested => {
                if let Err(e) = self.render() {
                    error!("Render error: {}", e);
                }

                if self.should_exit {
                    event_loop.exit();
                }

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Request redraw on next iteration
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("=== AstraWeave UI Menu Demo ===");
    info!("Controls:");
    info!("  - Click buttons to navigate menus");
    info!("  - ESC to toggle pause menu (when in-game)");
    info!("  - TAB to cycle focus (keyboard navigation)");
    info!("  - ENTER to activate focused button");
    info!("  - F3 to toggle HUD debug mode");
    info!("  - Keys 1/2/3 to spawn damage numbers (when in-game)");
    info!("  - Q to toggle quest tracker, M to toggle minimap");
    info!("  - C to collapse/expand quest tracker");
    info!("  - R to rotate minimap (north-up vs player-relative)");
    info!("  - T to toggle dialogue demo (branching conversation)");
    info!("  - Keys 1-4 to select dialogue choices (when dialogue active)");
    info!("  - H to heal player (+20 HP), D to damage player (-15 HP)");
    info!("  - N for 'New Quest!' notification (Week 4 Day 3)");
    info!("  - O for 'Objective Complete!' notification (Week 4 Day 3)");
    info!("  - P for 'Quest Complete!' notification (Week 4 Day 3)");
    info!("  - +/- to zoom minimap, G for ping marker (Week 4 Day 4)");
    info!("  - 'New Game' to start game");
    info!("  - 'Quit' to exit");
    info!("Week 3 Day 3: Quest tracker, minimap with POI markers");
    info!("Week 3 Day 4: Dialogue system with branching, tooltips");
    info!("Week 4 Day 1: Health bar smooth transitions with easing animations");
    info!("Week 4 Day 2: Damage number arc motion, combos, impact shake");
    info!("Week 4 Day 3: Quest notification slide animations");
    info!("Week 4 Day 4: Minimap zoom, dynamic POI icons, click-to-ping (NEW!)");


    // Create event loop and app
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();

    // Run event loop (winit 0.30 ApplicationHandler pattern)
    event_loop.run_app(&mut app)?;

    info!("Application exited cleanly");
    Ok(())
}
