# SLAM Visualizer — Project Plan

## Project Overview

A purpose-built SLAM visualization tool written in Rust using `wgpu` for GPU rendering and `egui` for the UI framework. The initial milestone is a 3D viewer for camera pose trajectories with a scrubable timeline. The architecture is designed to grow into a full SLAM debugging and introspection platform — supporting keypoints, feature matches, map points, bundle adjustment parameters, and more.

---

## Goals

### Milestone 1 (This Plan)
- Load a sequence of camera poses (position + orientation over time)
- Render camera frustums in a 3D viewport
- Scrubable timeline — user slides to a time index and the viewer smoothly updates
- Smooth 3D camera orbit/pan/zoom navigation

### Future Milestones (Architectural Hooks to Keep in Mind)
- 2D image panel with keypoint overlays and match lines
- 3D map point cloud rendering
- Covisibility graph edges between keyframes
- Bundle adjustment residual visualization
- Per-frame reprojection error colormapping

---

## Technology Stack

| Layer | Crate | Purpose |
|---|---|---|
| GPU rendering | `wgpu` | Cross-platform GPU API (Vulkan, Metal, DX12, WebGPU) |
| UI framework | `egui` | Immediate-mode UI: panels, sliders, timeline |
| Windowing | `winit` | OS window + event loop |
| egui ↔ wgpu bridge | `egui-wgpu` + `egui-winit` | Connects egui to the wgpu render pass |
| Math | `glam` | Vec3, Quat, Mat4 — no-std friendly, fast |
| Serialization | `serde` + `serde_json` | Loading pose data from JSON files |
| Logging | `tracing` | Structured logging for debugging |

### Why no Bevy
The ECS abstraction adds friction for batch-updated algorithm data (e.g. BA updating thousands of poses at once). Direct `wgpu` + `egui` gives full control over the render pipeline and teaches transferable GPU knowledge. The 3D viewport is rendered to a `wgpu` texture and displayed as an `egui` image — the same pattern used by Rerun.

---

## Repository Structure

```
slam_viz/
├── Cargo.toml
├── src/
│   ├── main.rs                  # Entry point: winit event loop
│   ├── app.rs                   # Top-level App struct, owns all state
│   ├── data/
│   │   ├── mod.rs
│   │   ├── pose.rs              # CameraPose struct (position, orientation, timestamp)
│   │   └── loader.rs            # Load poses from JSON / CSV
│   ├── renderer/
│   │   ├── mod.rs
│   │   ├── context.rs           # wgpu Device, Queue, Surface setup
│   │   ├── scene_renderer.rs    # Owns the offscreen render texture + render pass
│   │   ├── frustum.rs           # Frustum mesh + instance buffer + shader
│   │   ├── grid.rs              # Ground plane grid lines
│   │   ├── axes.rs              # World origin XYZ axes gizmo
│   │   └── camera.rs            # Orbit camera: view/proj matrix, input handling
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── layout.rs            # Top-level egui panel layout
│   │   ├── timeline.rs          # Timeline scrubber widget
│   │   └── viewport.rs          # egui panel that displays the wgpu texture
│   └── state/
│       ├── mod.rs
│       └── playback.rs          # Playback state: current frame, play/pause, speed
├── assets/
│   └── sample_poses.json        # Sample pose sequence for testing
└── shaders/
    ├── frustum.wgsl             # Frustum vertex + fragment shader
    ├── grid.wgsl                # Grid shader
    └── axes.wgsl                # Axes gizmo shader
```

---

## Data Format

### Camera Pose (`CameraPose`)

```rust
pub struct CameraPose {
    pub timestamp: f64,       // seconds
    pub position: [f32; 3],   // world-space XYZ
    pub orientation: [f32; 4], // quaternion XYZW
}
```

### Sample JSON Input (`assets/sample_poses.json`)

```json
[
  { "timestamp": 0.0,  "position": [0.0, 0.0, 0.0],  "orientation": [0.0, 0.0, 0.0, 1.0] },
  { "timestamp": 0.033,"position": [0.05, 0.0, 0.1], "orientation": [0.0, 0.02, 0.0, 0.9998] },
  { "timestamp": 0.066,"position": [0.1, 0.0, 0.2],  "orientation": [0.0, 0.04, 0.0, 0.999] }
]
```

The loader should accept both JSON and a simple CSV format (`timestamp,px,py,pz,qx,qy,qz,qw`) for flexibility.

---

## Architecture: How the Pieces Connect

```
winit EventLoop
    │
    └─► App::update(event)
            │
            ├─► PlaybackState::tick()         // advance frame if playing
            │
            ├─► SceneRenderer::render()        // draw 3D scene to offscreen texture
            │       │
            │       ├─► OrbitCamera::view_proj_matrix()
            │       ├─► FrustumRenderer::draw(visible_poses)
            │       ├─► GridRenderer::draw()
            │       └─► AxesRenderer::draw()
            │
            └─► egui frame
                    │
                    ├─► ViewportPanel::show(scene_texture)   // display wgpu texture in egui
                    ├─► TimelinePanel::show(&mut playback)   // scrubber + play controls
                    └─► InfoPanel::show(current_pose)        // current pose data readout
```

The 3D scene is rendered to an offscreen `wgpu::Texture`. This texture is registered with egui as a `TextureId` and displayed via `ui.image()` in the viewport panel. This is the key architectural pattern — egui owns the window layout, wgpu owns the 3D rendering, and they communicate through a shared texture handle.

---

## Implementation Plan

### Phase 0 — Project Scaffolding
**Goal:** Compiling app with a blank egui window.

Tasks:
1. `cargo new slam_viz`
2. Add dependencies to `Cargo.toml`: `wgpu`, `egui`, `egui-wgpu`, `egui-winit`, `winit`, `glam`, `serde`, `serde_json`, `tracing`, `tracing-subscriber`
3. Implement `main.rs`: create `winit` event loop, create window
4. Implement `renderer/context.rs`: initialize `wgpu` Instance, Adapter, Device, Queue, Surface
5. Implement basic `egui-winit` + `egui-wgpu` integration in the event loop
6. Render a blank egui window with a placeholder panel

**Acceptance criteria:** Window opens, egui renders a "Hello World" label, no panics.

---

### Phase 1 — Data Loading
**Goal:** Load and store camera poses in memory.

Tasks:
1. Implement `data/pose.rs` — `CameraPose` struct with `serde` derives
2. Implement `data/loader.rs` — `load_from_json(path) -> Vec<CameraPose>` and `load_from_csv(path) -> Vec<CameraPose>`
3. Generate `assets/sample_poses.json` — a circular camera trajectory with ~200 frames (programmatically generated, not hand-written)
4. Load poses at startup, store in `App`
5. Log pose count and timestamp range on startup via `tracing`

**Acceptance criteria:** Poses load without error, count logged to console.

---

### Phase 2 — Offscreen Render Texture + Viewport Panel
**Goal:** A wgpu texture displayed inside an egui panel. Nothing drawn yet — just the plumbing.

Tasks:
1. Implement `renderer/scene_renderer.rs`:
   - Create offscreen `wgpu::Texture` at a fixed resolution (e.g. 1280×720)
   - Create `wgpu::TextureView` for rendering into it
   - Expose a `render()` method that begins a render pass, clears to dark background color, ends pass
2. Register the texture with egui-wgpu's `RenderState` to get a `TextureId`
3. Implement `ui/viewport.rs` — `egui::CentralPanel` that calls `ui.image(texture_id, size)`
4. Wire together: each frame, render the offscreen texture then display it in egui

**Acceptance criteria:** Egui window shows a dark colored rectangle in the center panel where the 3D scene will appear.

---

### Phase 3 — Orbit Camera
**Goal:** Interactive 3D camera that responds to mouse input.

Tasks:
1. Implement `renderer/camera.rs` — `OrbitCamera` struct:
   - State: `target: Vec3`, `distance: f32`, `yaw: f32`, `pitch: f32`
   - Methods: `view_matrix() -> Mat4`, `proj_matrix(aspect: f32) -> Mat4`
   - Input handling: left-drag orbits, right-drag pans, scroll zooms
2. Create a `CameraUniform` struct with `view_proj: [[f32; 4]; 4]`
3. Create a `wgpu::Buffer` (uniform buffer) for the camera, update it each frame
4. Create a `wgpu::BindGroupLayout` and `wgpu::BindGroup` for the camera uniform
5. Handle `winit` mouse events in `App::update()` and forward to `OrbitCamera`

**Note on egui input conflict:** Check `egui_ctx.wants_pointer_input()` before forwarding mouse events to the orbit camera. If egui wants the input (e.g. user is interacting with a panel), do not pass it to the camera.

**Acceptance criteria:** Camera uniform buffer updates each frame. No visible output yet but no panics.

---

### Phase 4 — Ground Grid
**Goal:** First visible 3D geometry — a reference grid on the XZ plane.

Tasks:
1. Implement `shaders/grid.wgsl`:
   - Vertex shader: takes position, multiplies by `view_proj`
   - Fragment shader: outputs a dim grey color
2. Implement `renderer/grid.rs`:
   - Generate grid line vertices on the CPU (e.g. 20×20 grid, 1m spacing)
   - Upload to a `wgpu::Buffer`
   - Create render pipeline with `PrimitiveTopology::LineList`
   - `draw()` method takes a render pass and draws the grid
3. Call `GridRenderer::draw()` inside `SceneRenderer::render()`

**Acceptance criteria:** Dark viewport now shows a grey grid. Camera orbit/pan/zoom works and moves relative to the grid.

---

### Phase 5 — Camera Frustum Rendering
**Goal:** Render a camera frustum for each pose in the sequence.

Tasks:
1. Design frustum geometry: 5 vertices (apex + 4 corners of near plane), 8 edges as `LineList`
2. Implement `shaders/frustum.wgsl`:
   - Per-instance model matrix (one per pose) passed via instance buffer
   - Color varies by time index (e.g. gradient from blue=old to yellow=recent)
3. Implement `renderer/frustum.rs`:
   - Base frustum vertices in local camera space (unit frustum)
   - Instance buffer: one `FrustumInstance { model_matrix: [[f32;4];4], color: [f32;4] }` per pose
   - Render pipeline with instance buffer layout
   - `update_instances(poses: &[CameraPose], current_frame: usize)` — rebuilds instance data
   - `draw()` method
4. In `SceneRenderer`, accept `&[CameraPose]` and current frame index, pass to frustum renderer

**Frustum instance data layout:**
```rust
#[repr(C)]
struct FrustumInstance {
    model_matrix: [[f32; 4]; 4],  // columns 0-3: locations 0-3 in shader
    color: [f32; 4],              // location 4
}
```

**Acceptance criteria:** All loaded camera poses appear as frustum wireframes in the 3D viewport. Orbiting the camera shows them from different angles.

---

### Phase 6 — Playback State + Timeline UI
**Goal:** Timeline scrubber that controls which frames are visible.

Tasks:
1. Implement `state/playback.rs` — `PlaybackState`:
   ```rust
   pub struct PlaybackState {
       pub current_frame: usize,
       pub total_frames: usize,
       pub is_playing: bool,
       pub playback_speed: f32,      // 1.0 = realtime
       pub accumulated_time: f32,    // for frame advance
   }
   ```
   - `tick(delta_seconds: f32, timestamps: &[f64])` — advance `current_frame` based on elapsed time and speed
   - `seek(frame: usize)` — jump to frame
2. Implement `ui/timeline.rs` — `TimelinePanel::show()`:
   - `egui::Slider` bound to `current_frame` (0..total_frames)
   - Play/Pause button
   - Speed selector (0.25×, 0.5×, 1×, 2×)
   - Current timestamp display (seconds)
   - Frame counter display (`123 / 456`)
3. In `FrustumRenderer::update_instances()`, only show poses up to `current_frame`:
   - Poses before current frame: rendered with trail color (dim)
   - Current frame pose: highlighted color (bright white/yellow)
   - Poses after current frame: not rendered
4. Track `last_frame_time` in the event loop using `std::time::Instant` for delta time

**Acceptance criteria:** Dragging the timeline slider shows/hides frustums smoothly. Play button animates through the sequence. Speed control works.

---

### Phase 7 — World Axes Gizmo + Info Panel
**Goal:** Polish pass — orientation reference and data readout.

Tasks:
1. Implement `renderer/axes.rs` — XYZ axes at world origin (red/green/blue lines)
2. Implement `ui/layout.rs` — side panel showing:
   - Current frame index and timestamp
   - Current camera position (x, y, z)
   - Current camera orientation as Euler angles (converted from quaternion for readability)
   - Total pose count
3. Add a "Reset Camera" button that returns orbit camera to default position
4. Add a "Fit to Scene" button that sets orbit camera distance to encompass all poses

**Acceptance criteria:** Full UI layout: 3D viewport center, timeline bottom, info panel right side. Axes visible in 3D view.

---

### Phase 8 — File Loading UI
**Goal:** Load pose files at runtime without recompiling.

Tasks:
1. Add `rfd` crate (native file dialog) for "Open File" functionality
2. Add "Open" button to the top menu bar (`egui::TopBottomPanel`)
3. On file open: reload `App.poses`, reset `PlaybackState`, re-upload instance buffer
4. Handle errors gracefully — show an egui modal with the error message if parsing fails
5. Support drag-and-drop: handle `winit::event::WindowEvent::DroppedFile`

**Acceptance criteria:** User can open a JSON or CSV file from disk and see the new trajectory without restarting the app.

---

## Key Implementation Notes for the AI Agent

### wgpu + egui Texture Sharing Pattern
```rust
// Register the offscreen texture with egui-wgpu
let texture_id = renderer.register_native_texture(
    &device,
    &scene_texture_view,
    wgpu::FilterMode::Linear,
);

// In the egui frame, display it
egui::CentralPanel::default().show(ctx, |ui| {
    ui.image(egui::load::SizedTexture::new(texture_id, [width as f32, height as f32]));
});
```

### Render Pass Order
Each frame must follow this order:
1. Update camera uniform buffer (`queue.write_buffer`)
2. Update frustum instance buffer (`queue.write_buffer`)
3. Begin offscreen render pass → draw grid → draw axes → draw frustums → end pass
4. Begin egui render pass → render egui → end pass
5. Present surface

### Instance Buffer Sizing
Pre-allocate the instance buffer for `MAX_POSES` (e.g. 10,000) at startup. Use `queue.write_buffer` to update only the active slice each frame. Avoid reallocating buffers per frame.

### Shader Vertex Buffer Layouts
```rust
// Camera uniform bind group layout (group 0, binding 0)
// Frustum base geometry (location 0: position vec3)
// Frustum instance data (locations 1-4: model matrix columns, location 5: color)
```

### Input Handling Priority
Always check `egui_ctx.wants_pointer_input()` and `egui_ctx.wants_keyboard_input()` before processing input events for the 3D camera. Egui panels (sliders, buttons) must take input priority.

### Coordinate System Convention
Use a **right-handed Y-up coordinate system** throughout:
- X: right
- Y: up  
- Z: toward viewer (out of screen)

This matches `glam`'s defaults and standard OpenGL convention. SLAM datasets often use Z-forward (camera convention) — document any coordinate transform applied at load time.

---

## `Cargo.toml` Dependencies

```toml
[package]
name = "slam_viz"
version = "0.1.0"
edition = "2021"

[dependencies]
wgpu = "22"
egui = "0.29"
egui-wgpu = "0.29"
egui-winit = "0.29"
winit = { version = "0.30", features = ["rwh_06"] }
glam = { version = "0.29", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
rfd = "0.15"          # native file dialog (Phase 8)
bytemuck = { version = "1", features = ["derive"] }  # safe cast for GPU buffers
pollster = "0.3"      # block on async wgpu init in main
```

---

## Testing Strategy

Each phase has a clear visual acceptance criterion. In addition:

- `data/loader.rs`: unit tests for JSON and CSV parsing, including malformed input
- `renderer/camera.rs`: unit tests for `view_matrix()` and `proj_matrix()` — check known camera positions produce expected matrices
- `state/playback.rs`: unit tests for `tick()` — verify frame advance at different speeds and that it clamps at end of sequence
- Generate a deterministic test pose sequence (circular helix) programmatically for visual regression testing

---

## Future Architecture Hooks

The following are **not** in Milestone 1 but the architecture should not close the door on them:

- **`renderer/pointcloud.rs`**: instanced sphere or point rendering for 3D map points
- **`renderer/match_lines.rs`**: line segments between 2D keypoints across image pairs
- **`ui/image_panel.rs`**: egui panel showing a raw image frame with 2D overlays drawn via `egui::Painter`
- **`data/graph.rs`**: covisibility graph as an adjacency structure, rendered as 3D line segments
- **`ui/ba_panel.rs`**: bundle adjustment parameter inspector with residual histogram

Keep the `SceneRenderer` modular — each renderer (`FrustumRenderer`, `GridRenderer`, etc.) is an independent struct with its own pipeline, buffers, and `draw()` method. Adding a new primitive means adding a new struct, not modifying existing ones.