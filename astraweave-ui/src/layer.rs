use anyhow::Result;
use egui::{Context, FullOutput};
use egui-wgpu::renderer::{Renderer as EguiRenderer, ScreenDescriptor};
use egui_winit::State as EguiWinit;
use winit::event::{WindowEvent};
use winit::window::Window;

pub struct UiLayer {
    egui_ctx: Context,
    egui_winit: EguiWinit,
    egui_rend: EguiRenderer,
    pub scale_factor: f32,
}

impl UiLayer {
    pub fn new(window: &Window, device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let egui_ctx = Context::default();
        let egui_winit = egui_winit::State::new(window);
        let egui_rend = EguiRenderer::new(device, format, None, 1, false);
        let scale_factor = window.scale_factor() as f32;
        Self { egui_ctx, egui_winit, egui_rend, scale_factor }
    }

    pub fn on_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.egui_winit.on_window_event(&self.egui_ctx, event).consumed
    }

    /// Begin a new egui frame.
    pub fn begin(&mut self, window: &Window) {
        let raw = self.egui_winit.take_egui_input(window);
        self.egui_ctx.begin_frame(raw);
    }

    /// End the frame and paint to the provided frame view.
    pub fn end_and_paint(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: (u32,u32),
    ) {
        let FullOutput { platform_output, textures_delta, shapes, .. } = self.egui_ctx.end_frame();
        self.egui_winit.handle_platform_output(&self.egui_ctx, None, platform_output);

        let meshes = self.egui_ctx.tessellate(shapes, self.scale_factor);
        let screen = ScreenDescriptor {
            physical_width: size.0,
            physical_height: size.1,
            scale_factor: self.scale_factor,
        };

        for (id, delta) in &textures_delta.set {
            self.egui_rend.update_texture(device, queue, *id, delta);
        }
        self.egui_rend.update_buffers(device, queue, encoder, &meshes, &screen);
        self.egui_rend.render(encoder, view, &meshes, &screen);

        for id in &textures_delta.free {
            self.egui_rend.free_texture(id);
        }
    }

    pub fn ctx(&self) -> &egui::Context { &self.egui_ctx }
    pub fn ctx_mut(&mut self) -> &mut egui::Context { &mut self.egui_ctx.clone() }
}
