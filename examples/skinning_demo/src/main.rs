//! Skeletal Animation Demo
//!
//! Phase 2 Task 5 (Phase F): Interactive demonstration of skeletal animation system
//! Controls: Space (play/pause), [/] (speed), R (reset), G (CPU/GPU toggle), ESC (exit)

use astraweave_render::animation::*;
use glam::{Mat4, Quat, Vec3};
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
enum SkinningMode {
    CPU,
    #[allow(dead_code)]
    GPU, // Available with --features skinning-gpu
}

/// Demo application state
struct DemoApp {
    window: Option<Window>,
    skeleton: Skeleton,
    clip: AnimationClip,
    current_time: f32,
    playback_speed: f32,
    is_playing: bool,
    last_frame: Instant,
    mode: SkinningMode,
    frame_times: Vec<f32>, // Rolling window for FPS calc
    frame_count: u32,
    hud_timer: Instant,
}

impl DemoApp {
    fn new() -> Self {
        Self {
            window: None,
            skeleton: create_demo_skeleton(),
            clip: create_demo_animation(),
            current_time: 0.0,
            playback_speed: 1.0,
            is_playing: true,
            last_frame: Instant::now(),
            mode: SkinningMode::CPU,
            frame_times: Vec::with_capacity(60),
            frame_count: 0,
            hud_timer: Instant::now(),
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;

        // Track frame times for FPS
        self.frame_times.push(dt);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }

        if self.is_playing {
            self.current_time += dt * self.playback_speed;
            if self.current_time > self.clip.duration {
                self.current_time -= self.clip.duration; // Wrap
            }
        }
    }

    fn render_text_hud(&self) {
        // Text HUD (print to console for now - would use egui in full version)
        let avg_frame_time = if !self.frame_times.is_empty() {
            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
        } else {
            0.0
        };
        let fps = if avg_frame_time > 0.0 {
            1.0 / avg_frame_time
        } else {
            0.0
        };

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  SKINNING DEMO - Phase 2 Task 5");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  Mode:       {:?}", self.mode);
        println!("  Joints:     {}", self.skeleton.joints.len());
        println!("  Clip:       {}", self.clip.name);
        println!(
            "  Time:       {:.2}s / {:.2}s",
            self.current_time, self.clip.duration
        );
        println!("  Speed:      {:.2}×", self.playback_speed);
        println!(
            "  Status:     {}",
            if self.is_playing { "Playing" } else { "Paused" }
        );
        println!(
            "  FPS:        {:.1} ({:.2}ms)",
            fps,
            avg_frame_time * 1000.0
        );
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  Controls:");
        println!("    Space     Play/Pause");
        println!("    [/]       Slow/Fast playback");
        println!("    R         Reset to t=0");
        println!("    G         Toggle CPU/GPU");
        println!("    ESC       Exit");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }

    fn handle_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Space => {
                self.is_playing = !self.is_playing;
                println!(
                    "▶️ Animation {}",
                    if self.is_playing { "Playing" } else { "Paused" }
                );
            }
            KeyCode::BracketLeft => {
                self.playback_speed *= 0.5;
                println!("🐌 Playback speed: {:.2}×", self.playback_speed);
            }
            KeyCode::BracketRight => {
                self.playback_speed *= 2.0;
                println!("🚀 Playback speed: {:.2}×", self.playback_speed);
            }
            KeyCode::KeyR => {
                self.current_time = 0.0;
                println!("🔄 Reset to t=0");
            }
            KeyCode::KeyG => {
                #[cfg(feature = "skinning-gpu")]
                {
                    self.mode = match self.mode {
                        SkinningMode::CPU => SkinningMode::GPU,
                        SkinningMode::GPU => SkinningMode::CPU,
                    };
                    println!("🔧 Switched to {:?} mode", self.mode);
                }
                #[cfg(not(feature = "skinning-gpu"))]
                {
                    println!("⚠️  GPU mode requires --features skinning-gpu");
                }
            }
            _ => {}
        }
    }
}

impl ApplicationHandler for DemoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("AstraWeave - Skeletal Animation Demo (Phase 2 Task 5)")
                .with_inner_size(winit::dpi::LogicalSize::new(800, 600));

            match event_loop.create_window(window_attributes) {
                Ok(window) => {
                    self.window = Some(window);
                    // Initial HUD
                    self.render_text_hud();
                }
                Err(e) => {
                    eprintln!("Failed to create window: {:?}", e);
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        if window_id != window.id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                println!("\n👋 Exiting demo...\n");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.handle_input(key);
            }
            WindowEvent::RedrawRequested => {
                // Update animation
                self.update();

                // Sample animation
                let _local_poses = self.clip.sample(self.current_time, &self.skeleton);
                let _joint_matrices = compute_joint_matrices(&self.skeleton, &_local_poses);

                // CPU skinning (GPU would happen here with feature flag)
                match self.mode {
                    SkinningMode::CPU => {
                        // CPU skinning is already done via compute_joint_matrices
                    }
                    #[cfg(feature = "skinning-gpu")]
                    SkinningMode::GPU => {
                        // GPU skinning would upload joint_matrices to buffer
                        // and dispatch compute shader
                    }
                    #[cfg(not(feature = "skinning-gpu"))]
                    _ => {}
                }

                self.frame_count += 1;

                // Update HUD every 0.5s
                if self.hud_timer.elapsed().as_secs_f32() > 0.5 {
                    print!("\x1B[2J\x1B[1;1H"); // Clear console
                    self.render_text_hud();
                    self.hud_timer = Instant::now();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

/// Create demo skeleton (3 joints: root, spine, shoulder)
fn create_demo_skeleton() -> Skeleton {
    Skeleton {
        root_indices: vec![0],
        joints: vec![
            Joint {
                name: "root".to_string(),
                parent_index: None,
                inverse_bind_matrix: Mat4::IDENTITY,
                local_transform: Transform::default(),
            },
            Joint {
                name: "spine".to_string(),
                parent_index: Some(0),
                inverse_bind_matrix: Mat4::from_translation(Vec3::new(0.0, -1.0, 0.0)),
                local_transform: Transform {
                    translation: Vec3::new(0.0, 1.0, 0.0),
                    ..Default::default()
                },
            },
            Joint {
                name: "shoulder".to_string(),
                parent_index: Some(1),
                inverse_bind_matrix: Mat4::from_translation(Vec3::new(0.0, -2.0, 0.0)),
                local_transform: Transform {
                    translation: Vec3::new(0.0, 1.0, 0.0),
                    ..Default::default()
                },
            },
        ],
    }
}

/// Create demo animation (rotate spine from 0° to 90° over 2 seconds)
fn create_demo_animation() -> AnimationClip {
    AnimationClip {
        name: "demo_rotation".to_string(),
        duration: 2.0,
        channels: vec![AnimationChannel {
            target_joint_index: 1, // Animate spine
            times: vec![0.0, 1.0, 2.0],
            interpolation: Interpolation::Linear,
            data: ChannelData::Rotation(vec![
                Quat::IDENTITY,
                Quat::from_rotation_z(std::f32::consts::FRAC_PI_4), // 45°
                Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), // 90°
            ]),
        }],
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎮 Starting Skeletal Animation Demo...\n");

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = DemoApp::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
