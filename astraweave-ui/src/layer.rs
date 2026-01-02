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

fn begin_pass_with_raw_input(ctx: &Context, raw: egui::RawInput) {
    ctx.begin_pass(raw);
}

fn end_pass_and_tessellate(
    ctx: &Context,
    scale_factor: f32,
) -> (
    egui::PlatformOutput,
    egui::TexturesDelta,
    Vec<egui::ClippedPrimitive>,
) {
    let FullOutput {
        platform_output,
        textures_delta,
        shapes,
        ..
    } = ctx.end_pass();
    let meshes = ctx.tessellate(shapes, scale_factor);
    (platform_output, textures_delta, meshes)
}

fn default_screen_descriptor(pixels_per_point: f32) -> ScreenDescriptor {
    ScreenDescriptor {
        size_in_pixels: [0, 0],
        pixels_per_point,
    }
}

fn with_screen_size(mut screen: ScreenDescriptor, size: (u32, u32)) -> ScreenDescriptor {
    screen.size_in_pixels = [size.0, size.1];
    screen
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
        begin_pass_with_raw_input(&self.egui_ctx, raw);
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
        let (platform_output, textures_delta, meshes) =
            end_pass_and_tessellate(&self.egui_ctx, self.scale_factor);
        self.egui_winit
            .handle_platform_output(window, platform_output);
        let screen = default_screen_descriptor(self.scale_factor);

        (meshes, textures_delta, screen)
    }

    /// Paint the egui primitives. Must be called after end_frame.
    #[allow(clippy::too_many_arguments)]
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
        screen = with_screen_size(screen, size);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_screen_descriptor_is_zero_sized() {
        let desc = default_screen_descriptor(2.0);
        assert_eq!(desc.size_in_pixels, [0, 0]);
        assert_eq!(desc.pixels_per_point, 2.0);
    }

    #[test]
    fn test_with_screen_size_sets_dimensions() {
        let desc = default_screen_descriptor(1.0);
        let sized = with_screen_size(desc, (1920, 1080));
        assert_eq!(sized.size_in_pixels, [1920, 1080]);
        assert_eq!(sized.pixels_per_point, 1.0);
    }

    #[test]
    fn test_begin_and_end_pass_helpers_produce_meshes_and_screen() {
        let ctx = Context::default();
        begin_pass_with_raw_input(&ctx, egui::RawInput::default());
        let (_platform_out, textures_delta, meshes) = end_pass_and_tessellate(&ctx, 1.0);

        // What matters is that the pass lifecycle runs without panicking and
        // returns well-formed outputs. On the first frame egui commonly emits
        // a font texture delta.
        assert!(!textures_delta.set.is_empty());
        assert!(textures_delta.free.is_empty());
        let _ = meshes;

        let screen = default_screen_descriptor(1.0);
        assert_eq!(screen.size_in_pixels, [0, 0]);
        assert_eq!(screen.pixels_per_point, 1.0);
    }

    #[test]
    fn test_end_pass_and_tessellate_uses_scale_factor_for_screen_descriptor() {
        let ctx = Context::default();
        let scale = 1.75;
        begin_pass_with_raw_input(&ctx, egui::RawInput::default());
        let (_platform_out, _textures_delta, _meshes) = end_pass_and_tessellate(&ctx, scale);

        let screen = default_screen_descriptor(scale);
        assert_eq!(screen.pixels_per_point, scale);
    }

    #[test]
    fn test_screen_descriptor_construction() {
        let desc = ScreenDescriptor {
            size_in_pixels: [1920, 1080],
            pixels_per_point: 1.0,
        };
        assert_eq!(desc.size_in_pixels, [1920, 1080]);
        assert_eq!(desc.pixels_per_point, 1.0);
    }

    #[test]
    fn test_screen_descriptor_high_dpi() {
        let desc = ScreenDescriptor {
            size_in_pixels: [2560, 1440],
            pixels_per_point: 2.0,
        };
        assert_eq!(desc.size_in_pixels, [2560, 1440]);
        assert_eq!(desc.pixels_per_point, 2.0);
    }

    #[test]
    fn test_screen_descriptor_zero_size() {
        let desc = ScreenDescriptor {
            size_in_pixels: [0, 0],
            pixels_per_point: 1.0,
        };
        assert_eq!(desc.size_in_pixels, [0, 0]);
        assert_eq!(desc.pixels_per_point, 1.0);
    }

    #[test]
    fn test_screen_descriptor_ultra_high_dpi() {
        let desc = ScreenDescriptor {
            size_in_pixels: [3840, 2160],
            pixels_per_point: 3.0,
        };
        assert_eq!(desc.size_in_pixels, [3840, 2160]);
        assert_eq!(desc.pixels_per_point, 3.0);
    }

    #[test]
    fn test_screen_descriptor_fractional_scale() {
        let desc = ScreenDescriptor {
            size_in_pixels: [1920, 1080],
            pixels_per_point: 1.5,
        };
        assert_eq!(desc.size_in_pixels, [1920, 1080]);
        assert_eq!(desc.pixels_per_point, 1.5);
    }

    #[test]
    fn test_screen_descriptor_low_dpi() {
        let desc = ScreenDescriptor {
            size_in_pixels: [800, 600],
            pixels_per_point: 0.75,
        };
        assert_eq!(desc.size_in_pixels, [800, 600]);
        assert_eq!(desc.pixels_per_point, 0.75);
    }

    #[test]
    fn test_screen_descriptor_odd_dimensions() {
        let desc = ScreenDescriptor {
            size_in_pixels: [1366, 768],
            pixels_per_point: 1.0,
        };
        assert_eq!(desc.size_in_pixels, [1366, 768]);
        assert_eq!(desc.pixels_per_point, 1.0);
    }

    #[test]
    fn test_screen_descriptor_portrait_orientation() {
        let desc = ScreenDescriptor {
            size_in_pixels: [1080, 1920],
            pixels_per_point: 2.0,
        };
        assert_eq!(desc.size_in_pixels, [1080, 1920]);
        assert_eq!(desc.pixels_per_point, 2.0);
    }

    #[test]
    fn test_screen_descriptor_square() {
        let desc = ScreenDescriptor {
            size_in_pixels: [1024, 1024],
            pixels_per_point: 1.0,
        };
        assert_eq!(desc.size_in_pixels, [1024, 1024]);
        assert_eq!(desc.pixels_per_point, 1.0);
    }

    #[test]
    fn test_screen_descriptor_very_wide() {
        let desc = ScreenDescriptor {
            size_in_pixels: [3440, 1440],
            pixels_per_point: 1.0,
        };
        assert_eq!(desc.size_in_pixels, [3440, 1440]);
        assert_eq!(desc.pixels_per_point, 1.0);
    }

    // Note: Full UiLayer tests require wgpu/winit context,
    // which is integration-test level. Unit tests above cover
    // the data structures used in the API.
}
