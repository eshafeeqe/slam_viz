# SLAM Visualizer

A real-time 3D trajectory visualizer for SLAM (Simultaneous Localization and Mapping) camera pose data, built with Rust, wgpu, and egui.

![SLAM Visualizer](assets/sample_poses.json)

## Features

- **3D Viewport** — Renders camera frustums, a ground-plane grid, and XYZ axes gizmo using wgpu (WebGPU)
- **Blender-style camera controls** — Orbit, pan, zoom with MMB / Shift+MMB / Ctrl+MMB or scroll wheel
- **Timeline scrubber** — Play, pause, seek, and adjust playback speed (0.25×, 0.5×, 1×, 2×)
- **Pose inspector** — Live readout of position, Euler angles (deg), and quaternion for the active frame
- **File loading** — Open JSON or CSV pose files via menu or drag-and-drop
- **Modern dark UI** — VS Code / Blender-style theme with cyan accent color

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- A GPU with Vulkan, Metal, or OpenGL support

### Build & Run

```bash
git clone <repo-url>
cd slam_viz

# If Rust was installed via rustup and cargo isn't on PATH:
source ~/.cargo/env

cargo build
cargo run
```

The app loads `assets/sample_poses.json` (200-pose circular helix) on startup.

### Troubleshooting (Hybrid GPU / Linux)

If you get a `device lost` or `ERROR_INITIALIZATION_FAILED` panic, force OpenGL:

```bash
WGPU_BACKEND=gl cargo run
```

This is common on machines with both a discrete NVIDIA GPU and an integrated Intel GPU where the display is connected to the Intel adapter.

## Data Formats

### JSON

```json
[
  { "timestamp": 0.0, "position": [x, y, z], "orientation": [qx, qy, qz, qw] },
  { "timestamp": 0.033, "position": [x, y, z], "orientation": [qx, qy, qz, qw] }
]
```

### CSV

```
timestamp,px,py,pz,qx,qy,qz,qw
0.0,1.0,0.5,0.0,0.0,0.0,0.0,1.0
0.033,1.1,0.5,0.0,0.0,0.0,0.0,1.0
```

Header row is optional. Lines starting with `#` are skipped.

### Loading a File

- **Menu:** `File → Open…`
- **Drag & drop:** Drop a `.json` or `.csv` file onto the window

## Camera Controls

| Input | Action |
|---|---|
| MMB drag | Orbit |
| Shift + MMB drag | Pan |
| Ctrl + MMB drag | Zoom |
| Scroll wheel | Zoom |
| Left drag | Orbit (laptop / no MMB) |
| Numpad 7 | Top view |
| Numpad 1 | Front view |
| Numpad 3 | Right view |
| Numpad 5 | Reset view |

## Project Structure

```
slam_viz/
├── src/
│   ├── main.rs              # winit ApplicationHandler entry point
│   ├── app.rs               # Event loop, render orchestration, theme
│   ├── data/
│   │   ├── pose.rs          # CameraPose struct
│   │   └── loader.rs        # JSON + CSV loaders
│   ├── renderer/
│   │   ├── context.rs       # wgpu device/queue/surface (GpuContext)
│   │   ├── scene_renderer.rs# Offscreen texture + render passes
│   │   ├── camera.rs        # OrbitCamera + CameraUniform
│   │   ├── frustum.rs       # Instanced camera frustum wireframes
│   │   ├── grid.rs          # XZ ground-plane grid
│   │   └── axes.rs          # XYZ world-origin axes gizmo
│   ├── ui/
│   │   ├── layout.rs        # Panel layout, info panel, theme constants
│   │   ├── timeline.rs      # Timeline scrubber + playback controls
│   │   └── viewport.rs      # egui panel displaying the wgpu texture
│   └── state/
│       └── playback.rs      # PlaybackState: frame advance, seek, speed
├── shaders/
│   ├── frustum.wgsl
│   ├── grid.wgsl
│   └── axes.wgsl
└── assets/
    └── sample_poses.json    # 200-pose circular helix (bundled sample)
```

## Tech Stack

| Crate | Role |
|---|---|
| `wgpu 22` | GPU rendering (Vulkan / Metal / OpenGL / DX12) |
| `egui 0.29` | Immediate-mode UI |
| `egui-wgpu 0.29` | egui ↔ wgpu integration (offscreen texture pattern) |
| `winit 0.30` | Window + event loop |
| `glam 0.29` | Math (Vec3, Quat, Mat4) |
| `serde_json` | JSON pose file parsing |
| `rfd 0.15` | Native file-open dialog |

## Coordinate System

Right-handed, Y-up:
- **X** → right
- **Y** → up
- **Z** → toward viewer
