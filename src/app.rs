use std::sync::Arc;
use std::time::Instant;
use egui::TextureId;
use winit::event::{MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

use crate::data::{load_poses, CameraPose};
use crate::renderer::{GpuContext, OrbitCamera, SceneRenderer};
use crate::state::PlaybackState;
use crate::ui::{show_ui, pane_kind::PaneKind};

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
    tile_tree: egui_tiles::Tree<PaneKind>,
}

fn build_default_tree() -> egui_tiles::Tree<PaneKind> {
    let mut tiles = egui_tiles::Tiles::default();

    let view3d = tiles.insert_pane(PaneKind::View3D);
    let picker = tiles.insert_pane(PaneKind::PlotPicker);
    let info   = tiles.insert_pane(PaneKind::InfoPanel);

    let root = tiles.insert_horizontal_tile(vec![view3d, picker, info]);

    if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Linear(lin))) =
        tiles.get_mut(root)
    {
        lin.shares.set_share(view3d,  0.60);
        lin.shares.set_share(picker,  0.22);
        lin.shares.set_share(info,    0.18);
    }

    egui_tiles::Tree::new("slam_tiles", root, tiles)
}

impl App {
    pub async fn new(window: Arc<Window>) -> Self {
        let gpu = GpuContext::new(window.clone()).await;
        let scene = SceneRenderer::new(&gpu.device);

        let egui_ctx = egui::Context::default();

        // ── Modern dark theme ──────────────────────────────────────────────
        egui_ctx.set_visuals({
            let mut v = egui::Visuals::dark();
            v.panel_fill       = egui::Color32::from_rgb(24, 24, 32);
            v.window_fill      = egui::Color32::from_rgb(24, 24, 32);
            v.extreme_bg_color = egui::Color32::from_rgb(18, 18, 24);
            v.faint_bg_color   = egui::Color32::from_rgb(30, 30, 40);
            v.window_stroke    = egui::Stroke::new(1.0, egui::Color32::from_rgb(48, 48, 62));
            v.window_rounding  = egui::Rounding::same(6.0);
            v.slider_trailing_fill = true;
            v.selection = egui::style::Selection {
                bg_fill: egui::Color32::from_rgb(0, 100, 116),
                stroke:  egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 188, 212)),
            };
            v.widgets.noninteractive = egui::style::WidgetVisuals {
                weak_bg_fill: egui::Color32::from_rgb(24, 24, 32),
                bg_fill:      egui::Color32::from_rgb(24, 24, 32),
                bg_stroke:    egui::Stroke::new(1.0, egui::Color32::from_rgb(48, 48, 62)),
                fg_stroke:    egui::Stroke::new(1.0, egui::Color32::from_rgb(140, 140, 158)),
                rounding:     egui::Rounding::same(4.0),
                expansion:    0.0,
            };
            v.widgets.inactive = egui::style::WidgetVisuals {
                weak_bg_fill: egui::Color32::from_rgb(32, 32, 42),
                bg_fill:      egui::Color32::from_rgb(32, 32, 42),
                bg_stroke:    egui::Stroke::new(1.0, egui::Color32::from_rgb(48, 48, 62)),
                fg_stroke:    egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 215)),
                rounding:     egui::Rounding::same(4.0),
                expansion:    0.0,
            };
            v.widgets.hovered = egui::style::WidgetVisuals {
                weak_bg_fill: egui::Color32::from_rgb(42, 42, 56),
                bg_fill:      egui::Color32::from_rgb(42, 42, 56),
                bg_stroke:    egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 188, 212)),
                fg_stroke:    egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 188, 212)),
                rounding:     egui::Rounding::same(4.0),
                expansion:    1.0,
            };
            v.widgets.active = egui::style::WidgetVisuals {
                weak_bg_fill: egui::Color32::from_rgb(50, 50, 66),
                bg_fill:      egui::Color32::from_rgb(50, 50, 66),
                bg_stroke:    egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 188, 212)),
                fg_stroke:    egui::Stroke::new(2.0, egui::Color32::WHITE),
                rounding:     egui::Rounding::same(4.0),
                expansion:    1.0,
            };
            v.widgets.open = egui::style::WidgetVisuals {
                weak_bg_fill: egui::Color32::from_rgb(32, 32, 42),
                bg_fill:      egui::Color32::from_rgb(24, 24, 32),
                bg_stroke:    egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 188, 212)),
                fg_stroke:    egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 215)),
                rounding:     egui::Rounding::same(4.0),
                expansion:    0.0,
            };
            v
        });

        egui_ctx.style_mut(|style| {
            use egui::{FontFamily::Proportional, FontId, TextStyle::*};
            style.text_styles = std::collections::BTreeMap::from([
                (Small,     FontId::new(11.0, Proportional)),
                (Body,      FontId::new(13.0, Proportional)),
                (Monospace, FontId::new(13.0, egui::FontFamily::Monospace)),
                (Button,    FontId::new(13.0, Proportional)),
                (Heading,   FontId::new(16.0, Proportional)),
            ]);
            style.spacing.item_spacing   = egui::vec2(6.0, 4.0);
            style.spacing.interact_size  = egui::vec2(40.0, 24.0);
            style.spacing.button_padding = egui::vec2(8.0, 4.0);
        });
        // ──────────────────────────────────────────────────────────────────

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
            tile_tree: build_default_tree(),
        }
    }

    pub fn window_event(
        &mut self,
        event: &WindowEvent,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        // Always feed the event to egui so UI stays responsive.
        // IMPORTANT: do NOT early-return on resp.consumed for mouse events —
        // egui marks everything consumed when the pointer is over a panel (including
        // the 3D viewport CentralPanel), which would kill all camera input.
        // We use is_using_pointer() per-event to guard camera logic instead.
        let resp = self.egui_state.on_window_event(&self.window, event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                self.gpu.resize(size.width, size.height);
            }

            WindowEvent::DroppedFile(path) => {
                self.load_file(path.clone());
            }

            // Track modifier keys (always — no conflict with egui)
            WindowEvent::ModifiersChanged(mods) => {
                self.shift_held = mods.state().shift_key();
                self.ctrl_held = mods.state().control_key();
                self.update_drag_mode();
            }

            // Numpad preset views — only when egui doesn't want keyboard
            WindowEvent::KeyboardInput { event, .. } => {
                if resp.consumed {
                    return; // egui is handling keyboard (e.g. text field focused)
                }
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

            // Mouse button — always process for camera; is_using_pointer() guards
            // against clicks that started on a real UI widget (slider, button, etc.)
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = *state == winit::event::ElementState::Pressed;

                // Release always clears drag state regardless of egui
                if !pressed {
                    match button {
                        MouseButton::Middle => self.mouse_state = MouseState::None,
                        MouseButton::Left => {
                            if matches!(self.mouse_state, MouseState::Orbit { .. }) {
                                self.mouse_state = MouseState::None;
                            }
                        }
                        _ => {}
                    }
                    return;
                }

                // For press: only start camera drag when egui is NOT actively using pointer
                if self.egui_ctx.is_using_pointer() {
                    return;
                }

                match button {
                    MouseButton::Middle => self.start_drag(),
                    // Left button: fallback orbit (laptop / no MMB)
                    MouseButton::Left => {
                        self.mouse_state = MouseState::Orbit { last_x: 0.0, last_y: 0.0 };
                    }
                    _ => {}
                }
            }

            // Cursor move — drive active drag; never blocked by resp.consumed
            WindowEvent::CursorMoved { position, .. } => {
                let x = position.x as f32;
                let y = position.y as f32;

                // If egui grabbed the pointer mid-drag, cancel the drag
                if self.egui_ctx.is_using_pointer() {
                    self.mouse_state = MouseState::None;
                    return;
                }

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
                            self.camera.zoom(-dy * 0.02);
                        }
                        *last_y = y;
                    }
                    MouseState::None => {}
                }
            }

            // Scroll — zoom; skip only when egui is actively using it (e.g. scrollable panel)
            // Scroll-wheel zoom is handled inside the View3D pane (tile_behavior.rs)
            // so that it only fires when the pointer is actually over the viewport.

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
                &mut self.tile_tree,
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
