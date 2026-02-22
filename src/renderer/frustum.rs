use glam::{Mat4, Quat, Vec3};
use wgpu::util::DeviceExt;
use crate::data::CameraPose;

const MAX_POSES: usize = 10_000;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct FrustumVertex {
    position: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FrustumInstance {
    model_matrix: [[f32; 4]; 4],
    color: [f32; 4],
}

pub struct FrustumRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    instance_count: u32,
}

impl FrustumRenderer {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("frustum_shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../shaders/frustum.wgsl").into(),
            ),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("frustum_pipeline_layout"),
            bind_group_layouts: &[camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Vertex: location 0 = position (vec3)
        // Instance: locations 1-4 = model matrix columns, location 5 = color
        let vertex_attribs = wgpu::vertex_attr_array![0 => Float32x3];
        let instance_attribs = wgpu::vertex_attr_array![
            1 => Float32x4,
            2 => Float32x4,
            3 => Float32x4,
            4 => Float32x4,
            5 => Float32x4
        ];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("frustum_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<FrustumVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &vertex_attribs,
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<FrustumInstance>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &instance_attribs,
                    },
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Frustum in local camera space: apex at origin, 4 corners of near plane
        // Camera looks along -Z in local space
        let s = 0.08_f32; // half-width of near plane
        let d = 0.15_f32; // depth of frustum
        let vertices = [
            FrustumVertex { position: [0.0, 0.0, 0.0] }, // 0: apex
            FrustumVertex { position: [-s,  s, -d] },     // 1: TL
            FrustumVertex { position: [ s,  s, -d] },     // 2: TR
            FrustumVertex { position: [ s, -s, -d] },     // 3: BR
            FrustumVertex { position: [-s, -s, -d] },     // 4: BL
        ];

        // 8 edges as LineList: 4 from apex to corners, 4 around near plane
        let indices: [u16; 16] = [
            0, 1,  0, 2,  0, 3,  0, 4,  // apex to corners
            1, 2,  2, 3,  3, 4,  4, 1,  // near plane rectangle
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("frustum_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("frustum_index_buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("frustum_instance_buffer"),
            size: (MAX_POSES * std::mem::size_of::<FrustumInstance>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            instance_buffer,
            index_buffer,
            index_count: 16,
            instance_count: 0,
        }
    }

    pub fn update_instances(&mut self, queue: &wgpu::Queue, poses: &[CameraPose], current_frame: usize) {
        let visible = &poses[..=(current_frame.min(poses.len().saturating_sub(1)))];
        let total = visible.len();
        let mut instances = Vec::with_capacity(total.min(MAX_POSES));

        for (i, pose) in visible.iter().enumerate().take(MAX_POSES) {
            let t = if total > 1 { i as f32 / (total - 1) as f32 } else { 1.0 };
            let is_current = i == current_frame;

            let color = if is_current {
                [1.0, 1.0, 0.2, 1.0] // bright yellow for current
            } else {
                // gradient: blue (old) -> cyan (recent)
                [0.1, 0.3 + t * 0.5, 0.8 - t * 0.4, 0.7]
            };

            let pos = Vec3::from(pose.position);
            let rot = Quat::from_array(pose.orientation);
            let model = Mat4::from_rotation_translation(rot, pos);

            instances.push(FrustumInstance {
                model_matrix: model.to_cols_array_2d(),
                color,
            });
        }

        self.instance_count = instances.len() as u32;
        if !instances.is_empty() {
            queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
        }
    }

    pub fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, camera_bind_group: &'a wgpu::BindGroup) {
        if self.instance_count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, camera_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..self.index_count, 0, 0..self.instance_count);
    }
}
