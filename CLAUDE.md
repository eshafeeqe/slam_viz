# SLAM Visualizer — Claude Context

## Build & Run

```bash
source ~/.cargo/env   # Rust is installed via rustup; cargo not on PATH by default
cargo build
cargo run
```

## Project Structure

```
slam_viz/
├── src/
│   ├── main.rs              # winit ApplicationHandler entry point
│   ├── app.rs               # Top-level App: event loop, render orchestration
│   ├── data/
│   │   ├── pose.rs          # CameraPose struct (timestamp, position, orientation)
│   │   └── loader.rs        # load_from_json / load_from_csv / load_poses
│   ├── renderer/
│   │   ├── context.rs       # wgpu Device, Queue, Surface init (GpuContext)
│   │   ├── scene_renderer.rs# Offscreen texture + render pass orchestration
│   │   ├── camera.rs        # OrbitCamera + CameraUniform
│   │   ├── frustum.rs       # Instanced camera frustum wireframes
│   │   ├── grid.rs          # XZ ground plane grid
│   │   └── axes.rs          # XYZ world-origin axes gizmo
│   ├── ui/
│   │   ├── layout.rs        # Top-level egui panel layout (calls show_ui)
│   │   ├── timeline.rs      # Timeline scrubber + play/pause/speed
│   │   └── viewport.rs      # egui panel that displays the wgpu offscreen texture
│   └── state/
│       └── playback.rs      # PlaybackState: frame advance, seek, speed
├── shaders/
│   ├── frustum.wgsl
│   ├── grid.wgsl
│   └── axes.wgsl
└── assets/
    └── sample_poses.json    # 200-pose circular helix trajectory (generated)
```

## Key Architectural Decisions

- **Offscreen texture pattern**: The 3D scene renders into a `wgpu::Texture`, registered with egui-wgpu as a `TextureId`, then displayed via `ui.image()` in the `CentralPanel`. egui owns window layout; wgpu owns 3D rendering.
- **Render order per frame**: (1) update camera uniform, (2) update frustum instance buffer, (3) offscreen scene pass, (4) egui pass, (5) present surface.
- **Instance buffer**: Pre-allocated for `MAX_POSES = 10_000` at startup. `queue.write_buffer` updates the active slice each frame — no per-frame reallocation.

## wgpu 22 API Gotchas

- `wgpu::Instance::new(descriptor)` — takes ownership, **no `&` borrow**
- `entry_point` on vertex/fragment state is `&str`, **not** `Option<&str>`
- egui-wgpu `Renderer::render` requires `RenderPass<'static>` — call `.forget_lifetime()` on the render pass before passing it

## GPU / Display (this machine)

- **Hybrid GPU**: NVIDIA GTX 1060 (discrete) + Intel Skylake (integrated, display-connected)
- Using `power_preference: None` in `request_adapter` so wgpu picks the Intel display adapter; `HighPerformance` selects NVIDIA which has no display access and immediately panics with `ERROR_INITIALIZATION_FAILED / device lost`
- Set `WGPU_BACKEND=gl` to force OpenGL if Vulkan causes issues

## egui + 3D Camera Input — Critical Pattern

**Do NOT** use `if resp.consumed { return; }` as a blanket early return for mouse events. egui sets `consumed = true` whenever the pointer is over any panel — including the 3D viewport `CentralPanel` — which kills all camera input.

**Correct pattern**:
- Feed all events to egui via `egui_state.on_window_event()` (always)
- Keyboard: respect `resp.consumed` (protects text field focus)
- Mouse button press: guard with `egui_ctx.is_using_pointer()` only
- Mouse button release: always clear drag state (no guard)
- Cursor move / scroll: guard with `is_using_pointer()` only

`is_using_pointer()` = egui is actively dragging a widget (slider, scrollbar).
`wants_pointer_input()` = pointer is over *any* egui area (too broad — blocks viewport).

## Camera Controls (Blender-style)

| Input | Action |
|-------|--------|
| MMB drag | Orbit |
| Shift + MMB drag | Pan |
| Ctrl + MMB drag | Zoom |
| Scroll wheel | Zoom |
| Left drag | Orbit (laptop fallback) |
| Numpad 7 / 1 / 3 | Top / Front / Right view |
| Numpad 5 | Reset view |

## Coordinate System

Right-handed, Y-up:
- X → right
- Y → up
- Z → toward viewer

Camera `position()` negates pitch so positive pitch = camera above target (intuitive).

## Data Format

```json
[
  { "timestamp": 0.0, "position": [x, y, z], "orientation": [qx, qy, qz, qw] }
]
```
CSV: `timestamp,px,py,pz,qx,qy,qz,qw` (header row optional, `#` comments skipped).
