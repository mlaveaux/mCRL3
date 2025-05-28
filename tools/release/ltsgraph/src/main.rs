// This is a GUI application
#![windows_subsystem = "windows"]

slint::include_modules!();

use std::fs::File;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use clap::Parser;

use clap::ValueEnum;
use femtovg::renderer::WGPURenderer;
use femtovg::Canvas;
use log::debug;
use log::info;
use ltsgraph::init_wgpu;
use slint::Image;
use slint::Rgba8Pixel;
use slint::SharedPixelBuffer;
use slint::invoke_from_event_loop;
use slint::quit_event_loop;

use ltsgraph::PauseableThread;
use ltsgraph::show_error_dialog;
use mcrl3_gui::console;
use mcrl3_lts::LabelledTransitionSystem;
use mcrl3_lts::read_aut;
use mcrl3_ltsgraph_lib::FemtovgRenderer;
use mcrl3_ltsgraph_lib::GraphLayout;
use mcrl3_ltsgraph_lib::SkiaRenderer;
use mcrl3_ltsgraph_lib::Viewer;
use mcrl3_utilities::MCRL3Error;

#[derive(Clone, Debug, ValueEnum, PartialEq, Eq, PartialOrd, Ord)]
enum ViewerType {
    /// Uses tiny-skia to render the graph on the CPU
    CPU,
    /// Uses femtovg to render the graph on the GPU, with wgpu
    GPU,
}

#[derive(Parser, Debug)]
#[command(name = "Maurice Laveaux", about = "A lts viewing tool")]
pub struct Cli {
    #[arg(value_name = "FILE")]
    labelled_transition_system: Option<String>,

    #[arg(default_value_t = ViewerType::CPU, value_enum)]
    viewer: ViewerType,
}

/// Represents the viewer state including the viewer, pixel buffer, and renderer.
/// Encapsulates all rendering-related components.
struct ViewerState {
    viewer: Viewer,
    pixel_buffer: SharedPixelBuffer<Rgba8Pixel>,

    skia_renderer: SkiaRenderer,
    femtovg_renderer: Option<(FemtovgRenderer, Canvas<WGPURenderer>)>,
}

impl ViewerState {
    pub fn new(lts: Arc<LabelledTransitionSystem>, wgpu: &Option<(wgpu::Device, wgpu::Queue)>) -> Result<Self, MCRL3Error> {
        let viewer = Viewer::new(lts.clone());
        let pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(1, 1);
        let skia_renderer = SkiaRenderer::new(lts.clone());

        let femtovg_renderer = if let Some((device, queue)) = wgpu {
            let gpu_renderer = WGPURenderer::new(device.clone(), queue.clone());
            let canvas = Canvas::new(gpu_renderer)?;

            Some((FemtovgRenderer::new(lts), canvas))
        } else {
            None
        };


        Ok(Self {
            viewer,
            pixel_buffer,
            skia_renderer,
            femtovg_renderer
        })
    }
}

/// Contains all the GUI related state information.
struct GuiState {
    graph_layout: Mutex<GraphLayout>,
    viewer: Mutex<ViewerState>,
}

#[derive(Clone, Default)]
pub struct GuiSettings {
    // Layout related settings
    pub handle_length: f32,
    pub repulsion_strength: f32,
    pub delta: f32,

    // View related settings
    pub width: u32,
    pub height: u32,
    pub state_radius: f32,
    pub label_text_size: f32,
    pub draw_action_labels: bool,

    pub zoom_level: f32,
    pub view_x: f32,
    pub view_y: f32,
}

impl GuiSettings {
    pub fn new() -> GuiSettings {
        GuiSettings {
            width: 1,
            height: 1,
            zoom_level: 1.0,
            view_x: 500.0,
            view_y: 500.0,
            ..Default::default()
        }
    }
}

// Initialize a tokio runtime for async calls
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<ExitCode, MCRL3Error> {
    // Attach the standard output to the command line.
    let _console = console::init()?;

    // Parse the command line arguments and enable the logger.
    env_logger::init();

    let cli = Cli::parse();

    let wgpu = if cli.viewer == ViewerType::GPU {
        // Initialize wgpu for GPU rendering
        Some(init_wgpu().await?)
    } else {
        None
    };

    // Stores the shared state of the GUI components.
    let state = Arc::new(RwLock::new(None::<GuiState>));
    let settings = Arc::new(Mutex::new(GuiSettings::new()));
    let canvas = Arc::new(Mutex::new(SharedPixelBuffer::new(1, 1)));

    // Initialize the GUI, but show it later.
    let app = Application::new()?;

    {
        let app_weak = app.as_weak();
        let settings = settings.clone();

        app.on_settings_changed(move || {
            // Request the settings for the next simulation tick.
            if let Some(app) = app_weak.upgrade() {
                let mut settings = settings.lock().unwrap();
                settings.handle_length = app.global::<Settings>().get_handle_length();
                settings.repulsion_strength = app.global::<Settings>().get_repulsion_strength();
                settings.delta = app.global::<Settings>().get_timestep();
                settings.state_radius = app.global::<Settings>().get_state_radius();

                settings.draw_action_labels = app.global::<Settings>().get_draw_action_labels();
                settings.zoom_level = app.global::<Settings>().get_zoom_level();
                settings.view_x = app.global::<Settings>().get_view_x();
                settings.view_y = app.global::<Settings>().get_view_y();
                settings.label_text_size = app.global::<Settings>().get_label_text_height();
            }
        });
    };

    // Trigger it once to set the default values.
    app.invoke_settings_changed();

    // Render the view continuously, but only update the canvas when necessary
    let render_handle = {
        let state = state.clone();
        let app_weak: slint::Weak<Application> = app.as_weak();
        let settings = settings.clone();
        let canvas = canvas.clone();

        Arc::new(PauseableThread::new("ltsgraph canvas worker", move || {
            if let Some(state) = state.read().unwrap().deref() {
                // Render a new frame...
                let start = Instant::now();
                let mut viewer_state = state.viewer.lock().unwrap();

                // Resize the canvas when necessary
                let settings_clone = settings.lock().unwrap().clone();
                if viewer_state.pixel_buffer.width() != settings_clone.width
                    || viewer_state.pixel_buffer.height() != settings_clone.height
                {
                    viewer_state.pixel_buffer =
                        SharedPixelBuffer::<Rgba8Pixel>::new(settings_clone.width, settings_clone.height);
                }

                let guard = viewer_state.deref_mut();

                match cli.viewer {
                    ViewerType::CPU => {
                        let mut image = tiny_skia::PixmapMut::from_bytes(
                            guard.pixel_buffer.make_mut_bytes(),
                            settings_clone.width,
                            settings_clone.height,
                        )
                        .unwrap();

                        guard.skia_renderer.render(
                            &mut image,
                            &guard.viewer,
                            settings_clone.draw_action_labels,
                            settings_clone.state_radius,
                            settings_clone.view_x,
                            settings_clone.view_y,
                            settings_clone.width,
                            settings_clone.height,
                            settings_clone.zoom_level,
                            settings_clone.label_text_size,
                        );                        
                    }
                    // ViewerType::GPU => {
                    //     // Render the graph using femtovg on the GPU
                    //     if let Some((femtovg_renderer, canvas)) = &mut guard.femtovg_renderer {
                    //         femtovg_renderer.render(
                    //             canvas,
                    //             &guard.viewer,
                    //             settings_clone.draw_action_labels,
                    //             settings_clone.state_radius,
                    //             settings_clone.view_x,
                    //             settings_clone.view_y,
                    //             settings_clone.width,
                    //             settings_clone.height,
                    //             settings_clone.zoom_level,
                    //             settings_clone.label_text_size,
                    //         );
                    //     } 
                    // }
                }

                debug!(
                    "Rendering step ({} by {}) took {} ms",
                    settings_clone.width,
                    settings_clone.height,
                    (Instant::now() - start).as_millis()
                );

                *canvas.lock().unwrap() = viewer_state.pixel_buffer.clone();
            } else {
                // Resize the canvas when necessary
                let settings_clone = settings.lock().unwrap().clone();
                let mut canvas = canvas.lock().unwrap();
                if canvas.width() != settings_clone.width || canvas.height() != settings_clone.height {
                    *canvas = SharedPixelBuffer::<Rgba8Pixel>::new(settings_clone.width, settings_clone.height);
                }
            }

            // Request the to be updated.
            let app_weak = app_weak.clone();
            invoke_from_event_loop(move || {
                if let Some(app) = app_weak.upgrade() {
                    // Update the canvas
                    app.global::<Settings>()
                        .set_refresh(!app.global::<Settings>().get_refresh());
                };
            })
            .unwrap();

            false
        })?)
    };

    // Run the graph layout algorithm in a separate thread to avoid blocking the UI.
    let layout_handle = {
        let state = state.clone();
        let settings = settings.clone();
        let render_handle = render_handle.clone();

        Arc::new(PauseableThread::new("ltsgraph layout worker", move || {
            let start = Instant::now();
            let mut is_stable = true;

            if let Some(state) = state.read().unwrap().deref() {
                // Read the settings and free the lock since otherwise the callback above blocks.
                let settings = settings.lock().unwrap().clone();
                let mut layout = state.graph_layout.lock().unwrap();

                is_stable = layout.update(settings.handle_length, settings.repulsion_strength, settings.delta);
                if is_stable {
                    info!("Layout is stable!");
                }

                // Copy layout into the view.
                let mut viewer_state = state.viewer.lock().unwrap();
                viewer_state.viewer.update(&layout);

                // Request a redraw (if not already in progress).
                render_handle.resume();
            }

            // Keep at least 16 milliseconds between two layout runs.
            let duration = Instant::now() - start;
            debug!("Layout step took {} ms", duration.as_millis());
            thread::sleep(Duration::from_millis(16).saturating_sub(duration));

            // If stable pause the thread.
            !is_stable
        })?)
    };

    // Load an LTS from the given path and updates the state.
    let load_lts = {
        let state = state.clone();
        let layout_handle = layout_handle.clone();
        let render_handle = render_handle.clone();

        move |path: &Path| {
            debug!("Loading LTS {} ...", path.to_string_lossy());

            match File::open(path) {
                Ok(file) => {
                    match read_aut(file, vec![]) {
                        Ok(lts) => {
                            let lts = Arc::new(lts);
                            info!("Loaded lts {}", lts);

                            // Create the layout and viewer separately to make the initial state sensible.
                            let layout = GraphLayout::new(lts.clone());
                            let mut viewer = ViewerState::new(lts, &wgpu).unwrap();

                            viewer.viewer.update(&layout);

                            *state.write().unwrap() = Some(GuiState {
                                graph_layout: Mutex::new(layout),
                                viewer: Mutex::new(viewer),
                            });

                            // Enable the layout and rendering threads.
                            layout_handle.resume();
                            render_handle.resume();
                        }
                        Err(x) => {
                            show_error_dialog("Failed to load LTS!", &format!("{}", x));
                        }
                    }
                }
                Err(x) => {
                    show_error_dialog("Failed to load LTS!", &format!("{}", x));
                }
            }
        }
    };

    // When the simulation is toggled enable the layout thread.
    {
        let layout_handle = layout_handle.clone();
        app.on_run_simulation(move |enabled| {
            if enabled {
                layout_handle.resume();
            } else {
                layout_handle.pause();
            }
        })
    }

    // Simply return the current canvas, can be updated in the meantime.
    {
        let canvas = canvas.clone();
        let settings = settings.clone();
        let render_handle = render_handle.clone();

        app.on_update_canvas(move |width, height, _| {
            let mut settings = settings.lock().unwrap();
            settings.width = width as u32;
            settings.height = height as u32;

            let buffer = canvas.lock().unwrap().clone();
            if buffer.width() != settings.width || buffer.height() != settings.height {
                // Request another redraw when the size has changed.
                debug!(
                    "Canvas size changed from {}x{} to {width}x{height}",
                    buffer.width(),
                    buffer.height()
                );
                render_handle.resume();
            }

            debug!("Updating canvas");
            Image::from_rgba8_premultiplied(buffer)
        });
    }

    // If a redraw was requested resume the render thread.
    {
        let render_handle = render_handle.clone();
        app.on_request_redraw(move || {
            render_handle.resume();
        })
    }

    // Open the file dialog and load another LTS if necessary.
    {
        let load_lts = load_lts.clone();
        app.on_open_filedialog(move || {
            let load_lts = load_lts.clone();

            invoke_from_event_loop(move || {
                slint::spawn_local(async move {
                    if let Some(handle) = rfd::AsyncFileDialog::new().add_filter("", &["aut"]).pick_file().await {
                        load_lts(handle.path());
                    }
                })
                .unwrap();
            })
            .unwrap();
        });
    }

    // Focus on the graph
    {
        let settings = settings.clone();
        let state = state.clone();
        let render_handle = render_handle.clone();
        let app_weak = app.as_weak();
        let settings = settings.clone();

        app.on_focus_view(move || {
            if let Some(app) = app_weak.upgrade() {
                if let Some(state) = state.read().unwrap().deref() {
                    debug!("Centering view on graph.");

                    let state_view = state.viewer.lock().unwrap();

                    let center = state_view.viewer.center();

                    // Change the view to show the LTS in full.
                    app.global::<Settings>().set_view_x(center.x);
                    app.global::<Settings>().set_view_y(center.y);

                    let mut settings = settings.lock().unwrap();
                    settings.view_x = center.x;
                    settings.view_y = center.y;

                    render_handle.resume();
                }
            }
        });
    }

    app.on_quit(move || {
        // Stop the layout and quit.
        let _ = quit_event_loop();
    });

    // Loads the LTS given on the command line.
    if let Some(path) = &cli.labelled_transition_system {
        load_lts(Path::new(path));
    }

    app.run()?;

    // Stop the layout and quit.
    layout_handle.stop();
    render_handle.stop();

    Ok(ExitCode::SUCCESS)
}
