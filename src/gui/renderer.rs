//! Web content renderer using wgpu
//! 
//! This module handles rendering web content to the screen using wgpu for GPU acceleration.
//! It integrates with the existing CSS/layout engine to provide visual rendering.

use wgpu::{
    Device, Queue, Surface, SurfaceConfiguration, TextureFormat, PresentMode,
    Instance, Adapter, RequestAdapterOptions, DeviceDescriptor, Features, Limits,
    CommandEncoderDescriptor, TextureUsages, RenderPassDescriptor, RenderPassColorAttachment,
    Operations, LoadOp, Color,
};
use winit::{dpi::PhysicalSize, window::Window};
use anyhow::{Result, Context};
use std::sync::Arc;

use super::{WindowManager, BrowserUI, TabManager};

/// Render state for web content rendering
pub struct RenderState {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
}

/// Web content renderer
pub struct WebRenderer {
    render_state: RenderState,
    egui_renderer: egui_wgpu::Renderer,
    egui_winit: egui_winit::State,
    egui_ctx: egui::Context,
    window: std::sync::Arc<Window>,
}

impl WebRenderer {
    /// Create a new web renderer
    pub async fn new(window: &std::sync::Arc<Window>) -> Result<Self> {
        tracing::info!("Initializing web renderer with wgpu");

        let size = window.inner_size();

        // Create wgpu instance
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // Create surface
        let surface = instance.create_surface(std::sync::Arc::clone(window))
            .context("Failed to create wgpu surface")?;

        // Request adapter
        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await
            .context("Failed to find suitable GPU adapter")?;

        tracing::info!("Using GPU adapter: {}", adapter.get_info().name);

        // Request device and queue
        let (device, queue) = adapter.request_device(
            &DeviceDescriptor {
                required_features: Features::empty(),
                required_limits: Limits::default(),
                label: Some("Thalora Browser Device"),
            },
            None,
        ).await
            .context("Failed to create GPU device")?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo, // VSync
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        let render_state = RenderState {
            device,
            queue,
            surface,
            surface_config,
            size,
        };

        // Initialize egui for UI rendering
        let egui_ctx = egui::Context::default();
        let egui_winit = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            window.as_ref(),
            Some(window.scale_factor() as f32),
            Some(2048), // max texture side
        );

        let egui_renderer = egui_wgpu::Renderer::new(
            &render_state.device,
            surface_format,
            None,
            1,
        );

        tracing::info!("Web renderer initialized successfully");

        Ok(Self {
            render_state,
            egui_renderer,
            egui_winit,
            egui_ctx,
            window: std::sync::Arc::clone(window),
        })
    }

    /// Handle window events (returns true if event was consumed)
    pub fn handle_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.egui_winit.on_window_event(&self.window, event).consumed
    }

    /// Resize the renderer
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            tracing::debug!("Resizing renderer to: {:?}", new_size);
            
            self.render_state.size = new_size;
            self.render_state.surface_config.width = new_size.width;
            self.render_state.surface_config.height = new_size.height;
            
            self.render_state.surface.configure(
                &self.render_state.device, 
                &self.render_state.surface_config
            );
        }
    }

    /// Render a single frame
    pub fn render(
        &mut self,
        browser_ui: &mut BrowserUI,
        tab_manager: &TabManager,
    ) -> Result<()> {
        // Get surface texture
        let output = self.render_state.surface.get_current_texture()
            .context("Failed to get surface texture")?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder = self.render_state.device.create_command_encoder(
            &CommandEncoderDescriptor {
                label: Some("Thalora Render Encoder"),
            }
        );

        // Begin render pass for web content
        {
            let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Web Content Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 1.0, // White background
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // TODO: Render actual web content here
            // For now, we just clear to white background
            // In the future, this will render the DOM tree with CSS styling
        }

        // Render egui UI on top
        let raw_input = self.egui_winit.take_egui_input(&self.window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            browser_ui.show(ctx, tab_manager);
        });

        self.egui_winit.handle_platform_output(
            &self.window,
            full_output.platform_output,
        );

        let clipped_primitives = self.egui_ctx.tessellate(
            full_output.shapes,
            full_output.pixels_per_point,
        );

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(
                &self.render_state.device,
                &self.render_state.queue,
                *id,
                image_delta,
            );
        }

        self.egui_renderer.update_buffers(
            &self.render_state.device,
            &self.render_state.queue,
            &mut encoder,
            &clipped_primitives,
            &egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.render_state.size.width, self.render_state.size.height],
                pixels_per_point: self.egui_ctx.pixels_per_point(),
            },
        );

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("egui Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.egui_renderer.render(&mut render_pass, &clipped_primitives, &egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.render_state.size.width, self.render_state.size.height],
                pixels_per_point: self.egui_ctx.pixels_per_point(),
            });
        }

        for x in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(x);
        }

        // Submit commands
        self.render_state.queue.submit([encoder.finish()]);
        output.present();

        Ok(())
    }
}