use std::sync::Arc;
use std::time::Instant;
use egui::TextureId;
use winit::event::{MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

use crate::data::{load_poses, CameraPose};
use crate::renderer::{GpuContext, OrbitCamera, SceneRenderer};
use crate::state::PlaybackState;
use crate::ui::show_ui;

/// Blender-style mouse state
enum MouseState {
    None,
    /// MMB drag (no modifiers) → orbit
    Orbit { last_x: f32, last_y: f32 },
    /// Shift+MMB drag → pan
    Pan { last_x: f32, last_y: f32 },
    /// Ctrl+MMB drag → zoom (move along view axis)
    Zoom { last_y: f32 },
}

pub struct App {
    window: Arc<Window>,
    gpu: GpuContext,
    scene: SceneRenderer,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    scene_texture_id: TextureId,
    poses: Vec<CameraPose>,
    playback: PlaybackState,
    camera: OrbitCamera,
    last_frame: Instant,
    mouse_state: MouseState,
    shift_held: bool,
    ctrl_held: bool,
    open_file_requested: bool,
    error_msg: Option<String>,
}

impl App {
    pub async fn new(window: Arc<Window>) -> Self {
        let gpu = GpuContext::new(window.clone()).await;
        let scene = SceneRenderer::new(&gpu.device);

        let egui_ctx = egui::Context::default();
        let viewport_id = egui::ViewportId::ROOT;
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            viewport_id,
            &window,
            None,
            None,
            None,
        );

        let mut egui_renderer = egui_wgpu::Renderer::new(
            &gpu.device,
            gpu.surface_config.format,
            None,
            1,
            false,
        );

        let scene_texture_id = egui_renderer.register_native_texture(
            &gpu.device,
            &scene.texture_view,
            wgpu::FilterMode::Linear,
        );

        // Load sample poses
        let poses_path = std::path::Path::new("assets/sample_poses.json");
        let poses = if poses_path.exists() {
            match load_poses(poses_path) {
                Ok(p) => {
                    tracing::info!(
                        "Loaded {} poses, ts range: {:.3}s – {:.3}s",
                        p.len(),
                        p.first().map(|x| x.timestamp).unwrap_or(0.0),
                        p.last().map(|x| x.timestamp).unwrap_or(0.0),
                    );
                    p
                }
                Err(e) => {
                    tracing::error!("Failed to load poses: {e}");
                    vec![]
                }
            }
        } else {
            tracing::warn!("assets/sample_poses.json not found");
            vec![]
        };

        let total = poses.len();
        let playback = PlaybackState::new(total);
        let mut camera = OrbitCamera::new();
        camera.fit_to_scene(&poses);

        Self {
            window,
            gpu,
            scene,
            egui_ctx,
            egui_state,
            egui_renderer,
            scene_texture_id,
            poses,
            playback,
            camera,
            last_frame: Instant::now(),
            mouse_state: MouseState::None,
            shift_held: false,
            ctrl_held: false,
            open_file_requested: false,
            error_msg: None,
        }
    }

    pub fn window_event(
        &mut self,
        event: &WindowEvent,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        // Feed event to egui first
        let resp = self.egui_state.on_window_event(&self.window, event);
        if resp.consumed {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                self.gpu.resize(size.width, size.height);
            }

            WindowEvent::DroppedFile(path) => {
                self.load_file(path.clone());
            }

            // Track modifier keys
            WindowEvent::ModifiersChanged(mods) => {
                self.shift_held = mods.state().shift_key();
                self.ctrl_held = mods.state().control_key();
                // If modifier changed mid-drag, update the drag mode
                self.update_drag_mode();
            }

            // Numpad preset views (Blender-style)
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == winit::event::ElementState::Pressed {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        match key {
                            KeyCode::Numpad1 => self.camera.set_front_view(),
                            KeyCode::Numpad3 => self.camera.set_right_view(),
                            KeyCode::Numpad7 => self.camera.set_top_view(),
                            KeyCode::Numpad5 => self.camera.reset(),
                            _ => {}
                        }
                    }
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if self.egui_ctx.is_using_pointer() {
                    return;
                }
                let pressed = *state == winit::event::ElementState::Pressed;
                match button {
                    // Middle mouse button — Blender's primary 3D navigation button
                    MouseButton::Middle => {
                        if pressed {
                            self.start_drag();
                        } else {
                            self.mouse_state = MouseState::None;
                        }
                    }
                    // Keep left button as fallback orbit (for users without MMB / on laptops)
                    MouseButton::Left => {
                        if pressed {
                            self.mouse_state =
                                MouseState::Orbit { last_x: 0.0, last_y: 0.0 };
                        } else if matches!(self.mouse_state, MouseState::Orbit { .. }) {
                            self.mouse_state = MouseState::None;
                        }
                    }
                    _ => {}
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                if self.egui_ctx.is_using_pointer() {
                    self.mouse_state = MouseState::None;
                    return;
                }
                let x = position.x as f32;
                let y = position.y as f32;

                match &mut self.mouse_state {
                    MouseState::Orbit { last_x, last_y } => {
                        let dx = x - *last_x;
                        let dy = y - *last_y;
                        if *last_x != 0.0 || *last_y != 0.0 {
                            self.camera.orbit(dx, dy);
                        }
                        *last_x = x;
                        *last_y = y;
                    }
                    MouseState::Pan { last_x, last_y } => {
                        let dx = x - *last_x;
                        let dy = y - *last_y;
                        if *last_x != 0.0 || *last_y != 0.0 {
                            self.camera.pan(dx, dy);
                        }
                        *last_x = x;
                        *last_y = y;
                    }
                    MouseState::Zoom { last_y } => {
                        let dy = y - *last_y;
                        if *last_y != 0.0 {
                            // Drag up = zoom in (negative dy), drag down = zoom out
                            self.camera.zoom(-dy * 0.02);
                        }
                        *last_y = y;
                    }
                    MouseState::None => {}
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                if self.egui_ctx.is_using_pointer() {
                    return;
                }
                // Blender: scroll wheel zooms, one notch = ~10% distance change
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 * 0.05,
                };
                self.camera.zoom(scroll);
            }

            WindowEvent::RedrawRequested => {
                self.render();
            }

            _ => {}
        }
    }

    /// Start a drag with the correct mode based on current modifiers (Blender convention)
    fn start_drag(&mut self) {
        self.mouse_state = if self.shift_held {
            MouseState::Pan { last_x: 0.0, last_y: 0.0 }
        } else if self.ctrl_held {
            MouseState::Zoom { last_y: 0.0 }
        } else {
            MouseState::Orbit { last_x: 0.0, last_y: 0.0 }
        };
    }

    /// If a modifier key changes mid-drag, re-evaluate the drag mode
    fn update_drag_mode(&mut self) {
        if matches!(self.mouse_state, MouseState::Orbit { .. } | MouseState::Pan { .. } | MouseState::Zoom { .. }) {
            self.start_drag();
        }
    }

    pub fn about_to_wait(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;

        let timestamps: Vec<f64> = self.poses.iter().map(|p| p.timestamp).collect();
        self.playback.tick(dt, &timestamps);

        if self.open_file_requested {
            self.open_file_requested = false;
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Pose files", &["json", "csv"])
                .pick_file()
            {
                self.load_file(path);
            }
        }

        self.window.request_redraw();
    }

    fn load_file(&mut self, path: std::path::PathBuf) {
        match load_poses(&path) {
            Ok(poses) => {
                tracing::info!("Loaded {} poses from {}", poses.len(), path.display());
                self.poses = poses;
                self.playback = PlaybackState::new(self.poses.len());
                self.camera.fit_to_scene(&self.poses);
                self.error_msg = None;
            }
            Err(e) => {
                let msg = format!("{e}");
                tracing::error!("{msg}");
                self.error_msg = Some(msg);
            }
        }
    }

    fn render(&mut self) {
        self.scene.render(
            &self.gpu.device,
            &self.gpu.queue,
            &self.camera,
            &self.poses,
            self.playback.current_frame,
        );

        let output = match self.gpu.surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost) => {
                let w = self.gpu.surface_config.width;
                let h = self.gpu.surface_config.height;
                self.gpu.resize(w, h);
                return;
            }
            Err(e) => {
                tracing::error!("Surface error: {e}");
                return;
            }
        };

        let surface_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let raw_input = self.egui_state.take_egui_input(&self.window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            show_ui(
                ctx,
                self.scene_texture_id,
                &self.poses,
                &mut self.playback,
                &mut self.camera,
                &mut self.open_file_requested,
                &mut self.error_msg,
            );
        });

        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output.clone());

        let screen_desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.gpu.surface_config.width, self.gpu.surface_config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        let mut encoder =
            self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("egui_encoder"),
            });

        let clipped_primitives =
            self.egui_ctx.tessellate(full_output.shapes, screen_desc.pixels_per_point);

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(
                &self.gpu.device,
                &self.gpu.queue,
                *id,
                image_delta,
            );
        }

        self.egui_renderer.update_buffers(
            &self.gpu.device,
            &self.gpu.queue,
            &mut encoder,
            &clipped_primitives,
            &screen_desc,
        );

        {
            let mut egui_pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("egui_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &surface_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                })
                .forget_lifetime();

            self.egui_renderer.render(&mut egui_pass, &clipped_primitives, &screen_desc);
        }

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
