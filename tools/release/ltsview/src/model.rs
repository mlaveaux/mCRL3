use std::error::Error;

use tobj::Model;
use wgpu::Device;
use wgpu::Queue;

fn test(device: &Device, queue: &Queue) -> Result<(), Box<dyn Error>> {
    let (models, _) = tobj::load_obj("data/teapot.obj", &tobj::GPU_LOAD_OPTIONS)?;

    for model in &models {
        // Bring the models into the GPU
        device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (models.len() * std::mem::size_of::<Model>()) as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
    }

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 1024,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let offset = 0;
    let data = &[0u8; 1024];

    queue.write_buffer(&buffer, offset, data);

    Ok(())
}
