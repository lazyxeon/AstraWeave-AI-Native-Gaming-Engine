use egui::{Context, FullOutput};
use egui_wgpu::{Renderer as EguiRenderer, ScreenDescriptor};
use egui_winit::State as EguiWinit;
use winit::event::WindowEvent;
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
        let egui_winit = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            Some(winit::window::Theme::Dark), // theme parameter (winit theme)
            Some(device.limits().max_texture_dimension_2d as usize), // max_texture_side
        );
        let egui_rend = EguiRenderer::new(device, format, None, 1, false); // false for srgb_support
        let scale_factor = window.scale_factor() as f32;
        Self {
            egui_ctx,
            egui_winit,
            egui_rend,
            scale_factor,
        }
    }

    pub fn on_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let response = self.egui_winit.on_window_event(window, event);
        response.consumed
    }

    /// Begin a new egui frame.
    pub fn begin(&mut self, window: &Window) {
        let raw = self.egui_winit.take_egui_input(window);
        self.egui_ctx.begin_pass(raw);
    }

    /// End the frame and return the primitives for rendering.
    /// The caller must create a render pass and call `paint_primitives`.
    pub fn end_frame(
        &mut self,
        window: &Window,
    ) -> (
        Vec<egui::ClippedPrimitive>,
        egui::TexturesDelta,
        ScreenDescriptor,
    ) {
        let FullOutput {
            platform_output,
            textures_delta,
            shapes,
            ..
        } = self.egui_ctx.end_pass();
        self.egui_winit
            .handle_platform_output(window, platform_output);

        let meshes = self.egui_ctx.tessellate(shapes, self.scale_factor);
        let screen = ScreenDescriptor {
            size_in_pixels: [0, 0], // Will be set by caller
            pixels_per_point: self.scale_factor,
        };

        (meshes, textures_delta, screen)
    }

    /// Paint the egui primitives. Must be called after end_frame.
    pub fn paint(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        meshes: &[egui::ClippedPrimitive],
        textures_delta: &egui::TexturesDelta,
        mut screen: ScreenDescriptor,
        size: (u32, u32),
    ) {
        screen.size_in_pixels = [size.0, size.1];

        // Update textures
        for (id, delta) in &textures_delta.set {
            self.egui_rend.update_texture(device, queue, *id, delta);
        }

        // Update buffers
        self.egui_rend
            .update_buffers(device, queue, encoder, meshes, &screen);

        // Render
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // SAFETY: The render pass doesn't outlive the encoder scope,
            // so extending the lifetime to 'static is safe here.
            // This is required due to egui-wgpu's API in version 0.32.
            let render_pass_static: &mut wgpu::RenderPass<'static> =
                unsafe { std::mem::transmute(&mut render_pass) };
            self.egui_rend.render(render_pass_static, meshes, &screen);
        }

        // Free textures
        for id in &textures_delta.free {
            self.egui_rend.free_texture(id);
        }
    }

    /// End the frame and paint to the provided frame view (legacy method).
    pub fn end_and_paint(
        &mut self,
        window: &Window,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: (u32, u32),
    ) {
        let (meshes, textures_delta, screen) = self.end_frame(window);
        self.paint(
            device,
            queue,
            encoder,
            view,
            &meshes,
            &textures_delta,
            screen,
            size,
        );
    }

    pub fn ctx(&self) -> &egui::Context {
        &self.egui_ctx
    }
    pub fn ctx_mut(&mut self) -> &mut egui::Context {
        &mut self.egui_ctx
    }
}
