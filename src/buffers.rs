use crate::context::Context;
use bytemuck::Pod;
use wgpu::{Label, util::DeviceExt};

pub struct UniformBuffer<T> {
    pub data: T,
    pub buffer: wgpu::Buffer,
}

impl<T> UniformBuffer<T>
where
    T: Pod,
{
    pub fn new(data: T, context: &Context, label: Option<&str>) -> Self {
        let buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(&[data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        Self { data, buffer }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, data: T) {
        self.data = data;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
    }
}

pub struct Texture {
    pub extent: wgpu::Extent3d,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn new(
        width: u32,
        height: u32,
        context: &Context,
        usage: wgpu::TextureUsages,
        label: Option<&str>,
    ) -> Self {
        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: context.surface_format,
            usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            extent,
            texture,
            view,
            sampler,
        }
    }
}

pub struct StorageTexture {
    pub extent: wgpu::Extent3d,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl StorageTexture {
    pub fn new(
        width: u32,
        height: u32,
        context: &Context,
        usage: wgpu::TextureUsages,
        label: Option<&str>,
    ) -> Self {
        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: context.surface_format,
            usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            extent,
            texture,
            view,
        }
    }
}
