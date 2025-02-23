// This is a GUI application
#![windows_subsystem = "windows"]

slint::include_modules!();

use std::error::Error;
use std::process::ExitCode;

use log::debug;
use slint::Image;

use mcrl3_gui::console;

// Initialize a tokio runtime for async calls
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<ExitCode, Box<dyn Error>> {
    // Attach the standard output to the command line.
    let _console = console::init()?;

    // Parse the command line arguments and enable the logger.
    env_logger::init();

    // The instance is a handle to our GPU
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });

    // Simply take the first adapter and create a device for it with all features.
    let adapter = instance
        .enumerate_adapters(wgpu::Backends::all())
        .first()
        .cloned()
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
            },
            None,
        )
        .await
        .unwrap();

    // Load a model to render

    let render_texture = device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: 1024,
            height: 1024,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        label: None,
        view_formats: &[],
    });

    // creating a buffer with MAP_READ flag (not necessarily mapped right away)
    // doing the GPU work of filling the texture texels
    // executing copy_texture_to_buffer to copy texels from this texture into the buffer (it has to be non-mapped at this point)
    // requesting to map that buffer to CPU with map_read_async
    // callback is provided that gets you the slice of data
    let _ = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 0 as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let app = Application::new()?;
    {
        // app.on_update_canvas(move |width, height, _| {
        //     if render_texture.width() != width as u32 || render_texture.height() != height as u32 {
        //         // Request another redraw when the size has changed.
        //         debug!(
        //             "Canvas size changed from {}x{} to {width}x{height}",
        //             render_texture.width(),
        //             render_texture.height()
        //         );
        //     }

        //     debug!("Updating canvas");
        //     Image::from_rgba8_premultiplied(buffer)
        // })
    }

    app.run()?;

    Ok(ExitCode::SUCCESS)
}
