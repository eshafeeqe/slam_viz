use wgpu::util::DeviceExt;
use crate::data::CameraPose;
use super::{
    camera::{OrbitCamera, CameraUniform},
    grid::GridRenderer,
    axes::AxesRenderer,
    frustum::FrustumRenderer,
};

pub const SCENE_WIDTH: u32 = 1280;
pub const SCENE_HEIGHT: u32 = 720;

pub struct SceneRenderer {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    grid: GridRenderer,
    axes: AxesRenderer,
    pub frustum: FrustumRenderer,
    pub format: wgpu::TextureFormat,
}

impl SceneRenderer {
    pub fn new(device: &wgpu::Device) -> Self {
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("scene_texture"),
            size: wgpu::Extent3d { width: SCENE_WIDTH, height: SCENE_HEIGHT, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("scene_depth"),
            size: wgpu::Extent3d { width: SCENE_WIDTH, height: SCENE_HEIGHT, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let camera_uniform = CameraUniform { view_proj: glam::Mat4::IDENTITY.to_cols_array_2d() };
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::bytes_of(&camera_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bg"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let grid = GridRenderer::new(device, format, &camera_bind_group_layout);
        let axes = AxesRenderer::new(device, format, &camera_bind_group_layout);
        let frustum = FrustumRenderer::new(device, format, &camera_bind_group_layout);

        Self {
            texture,
            texture_view,
            depth_texture,
            depth_view,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
            grid,
            axes,
            frustum,
            format,
        }
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, camera: &OrbitCamera) {
        let aspect = SCENE_WIDTH as f32 / SCENE_HEIGHT as f32;
        let uniform = CameraUniform::from_camera(camera, aspect);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &OrbitCamera,
        poses: &[CameraPose],
        current_frame: usize,
    ) {
        self.update_camera(queue, camera);
        self.frustum.update_instances(queue, poses, current_frame);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("scene_encoder"),
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.08, g: 0.08, b: 0.1, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.grid.draw(&mut pass, &self.camera_bind_group);
            self.axes.draw(&mut pass, &self.camera_bind_group);
            self.frustum.draw(&mut pass, &self.camera_bind_group);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
