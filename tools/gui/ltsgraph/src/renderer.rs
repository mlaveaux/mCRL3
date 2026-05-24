use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex;

use clap::ValueEnum;
use femtovg::Canvas;
use femtovg::TextContext;
use femtovg::renderer::WGPURenderer;
use slint::Rgba8Pixel;
use slint::SharedPixelBuffer;
use wgpu::TextureDescriptor;
use wgpu::TextureFormat;
use wgpu::TextureUsages;

use merc_lts::LabelledTransitionSystem;
use merc_ltsgraph_lib::FemtovgRenderer;
use merc_ltsgraph_lib::SkiaRenderer;
use merc_ltsgraph_lib::Viewer;
use merc_utilities::MercError;

/// Selects which backend the [`Renderer`] uses to draw the graph.
#[derive(Clone, Debug, ValueEnum, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViewerType {
    /// Uses `tiny-skia` to render the graph on the CPU.
    Cpu,
    /// Uses `femtovg` to render the graph on the GPU, with wgpu.
    Gpu,
}

/// Settings controlling how the graph is drawn for a single frame.
#[derive(Clone, Default)]
pub struct RenderSettings {
    pub width: u32,
    pub height: u32,
    pub state_radius: f32,
    pub label_text_size: f32,
    pub draw_action_labels: bool,
    pub zoom_level: f32,
    pub view_x: f32,
    pub view_y: f32,
}

/// Output target shared with the GUI thread. Cloned into the GPU map-async callback.
pub type CanvasOutput = Arc<Mutex<SharedPixelBuffer<Rgba8Pixel>>>;

/// Local information required for the femtovg GPU renderer.
struct FemtovgInfo {
    renderer: FemtovgRenderer,
    canvas: Canvas<WGPURenderer>,
    texture: wgpu::Texture,
    buffer: Arc<wgpu::Buffer>,
}

/// Encapsulates the CPU (tiny-skia) and GPU (femtovg via wgpu) rendering paths.
pub struct Renderer {
    viewer_type: ViewerType,
    wgpu: Option<(wgpu::Device, wgpu::Queue)>,
    skia_renderer: Option<SkiaRenderer>,
    femtovg_info: Option<FemtovgInfo>,
    pixel_buffer: SharedPixelBuffer<Rgba8Pixel>,
}

impl Renderer {
    pub fn new(
        viewer_type: ViewerType,
        wgpu: Option<(wgpu::Device, wgpu::Queue)>,
        width: u32,
        height: u32,
    ) -> Renderer {
        Renderer {
            viewer_type,
            wgpu,
            skia_renderer: None,
            femtovg_info: None,
            pixel_buffer: SharedPixelBuffer::<Rgba8Pixel>::new(width, height),
        }
    }

    /// Recreate the backend renderers for the given LTS.
    pub fn reload(
        &mut self,
        lts: Arc<LabelledTransitionSystem<String>>,
        settings: &RenderSettings,
    ) -> Result<(), MercError> {
        self.skia_renderer = Some(SkiaRenderer::new(lts.clone()));

        self.femtovg_info = if let Some((device, queue)) = &self.wgpu {
            // Ensure that we embed one font that can be used, since on Windows there are no default fonts.
            let text_context = TextContext::default();
            text_context.add_font_mem(include_bytes!("../data/NotoSans-Regular.ttf") as &[u8])?;

            let gpu_renderer = WGPURenderer::new(device.clone(), queue.clone());
            let canvas = Canvas::new_with_text_context(gpu_renderer, text_context)?;

            let texture = create_texture(device, settings.width, settings.height);
            let buffer = Arc::new(create_buffer(device, settings.width, settings.height));

            Some(FemtovgInfo {
                renderer: FemtovgRenderer::new(lts),
                canvas,
                texture,
                buffer,
            })
        } else {
            None
        };

        Ok(())
    }

    /// Render `viewer` using the configured backend and write the result into `output`.
    pub fn render(
        &mut self,
        viewer: &mut Viewer,
        settings: &RenderSettings,
        output: &CanvasOutput,
    ) -> Result<(), MercError> {
        match self.viewer_type {
            ViewerType::Cpu => self.render_cpu(viewer, settings, output),
            ViewerType::Gpu => self.render_gpu(viewer, settings, output),
        }
    }

    fn render_cpu(
        &mut self,
        viewer: &mut Viewer,
        settings: &RenderSettings,
        output: &CanvasOutput,
    ) -> Result<(), MercError> {
        if self.pixel_buffer.width() != settings.width || self.pixel_buffer.height() != settings.height {
            self.pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(settings.width, settings.height);
        }

        let mut image =
            tiny_skia::PixmapMut::from_bytes(self.pixel_buffer.make_mut_bytes(), settings.width, settings.height)
                .unwrap();

        if let Some(skia_renderer) = &mut self.skia_renderer {
            skia_renderer.render(
                &mut image,
                viewer,
                settings.draw_action_labels,
                settings.state_radius,
                settings.view_x,
                settings.view_y,
                settings.width,
                settings.height,
                settings.zoom_level,
                settings.label_text_size,
            );
        }

        *output.lock().unwrap() = self.pixel_buffer.clone();
        Ok(())
    }

    fn render_gpu(
        &mut self,
        viewer: &mut Viewer,
        settings: &RenderSettings,
        output: &CanvasOutput,
    ) -> Result<(), MercError> {
        let (device, queue) = self
            .wgpu
            .as_ref()
            .expect("GPU rendering requires wgpu to be initialized");

        let Some(femtovg_info) = &mut self.femtovg_info else {
            return Ok(());
        };

        if femtovg_info.texture.width() != settings.width || femtovg_info.texture.height() != settings.height {
            femtovg_info.texture = create_texture(device, settings.width, settings.height);
            femtovg_info.buffer = Arc::new(create_buffer(device, settings.width, settings.height));
        }

        femtovg_info.renderer.render(
            &mut femtovg_info.canvas,
            viewer,
            settings.draw_action_labels,
            settings.state_radius,
            settings.view_x,
            settings.view_y,
            settings.width,
            settings.height,
            settings.zoom_level,
            settings.label_text_size,
        )?;

        let render_buffer = femtovg_info
            .canvas
            .flush_to_output(&femtovg_info.texture)
            .ok_or("Failed to flush the output")?;

        // Copy the texture to a buffer such that it can be mapped on the CPU
        let copy = femtovg_info.texture.as_image_copy();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ltsgraph canvas encoder"),
        });

        encoder.copy_texture_to_buffer(
            copy,
            wgpu::TexelCopyBufferInfo {
                buffer: &femtovg_info.buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(align_up(settings.width, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * 4),
                    rows_per_image: Some(settings.height),
                },
            },
            wgpu::Extent3d {
                width: femtovg_info.texture.width(),
                height: femtovg_info.texture.height(),
                depth_or_array_layers: 1,
            },
        );

        queue.submit([render_buffer, encoder.finish()]);

        let buffer = femtovg_info.buffer.clone();
        let output = output.clone();
        let width = settings.width;
        let height = settings.height;
        femtovg_info
            .buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                if result.is_ok() {
                    let mut canvas = output.lock().unwrap();

                    *canvas =
                        SharedPixelBuffer::clone_from_slice(buffer.slice(..).get_mapped_range().deref(), width, height);

                    buffer.unmap();
                }
            });

        // Wait for the buffer to be mapped and the data to be copied.
        device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })?;

        Ok(())
    }
}

/// Aligns a number up to the next multiple of the given alignment.
const fn align_up(num: u32, align: u32) -> u32 {
    ((num) + ((align) - 1)) & !((align) - 1)
}

fn create_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("ltsgraph canvas texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn create_buffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ltsgraph canvas buffer"),
        size: (align_up(width, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * height * 4) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}
