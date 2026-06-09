//! Native Windows API GUI backend.
//!
//! This backend uses winit for windowing, wgpu for rendering,
//! and egui for the UI. It provides native Windows integration.

use crate::app::App;
use crate::config::Config;
use crate::gui::{GuiBackend, GuiConfig};
use egui::emath::vec2;
use egui::epaint::Color32;
use std::sync::Arc;

/// WinAPI/wgpu GUI backend.
pub struct WinapiBackend {
    gui_config: GuiConfig,
}

impl GuiBackend for WinapiBackend {
    fn init(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            gui_config: GuiConfig::from(config),
        })
    }

    fn run(&mut self, app: &mut App) -> anyhow::Result<()> {
        // Run with pollster (async runtime for wgpu)
        pollster::block_on(run_wgpu_gui(self.gui_config.clone(), app))?;

        Ok(())
    }

    fn name() -> &'static str {
        "winapi"
    }
}

/// Run the wgpu-based GUI.
async fn run_wgpu_gui(gui_config: GuiConfig, _app: &mut App) -> anyhow::Result<()> {
    // Create window with winit
    let event_loop = winit::event_loop::EventLoopBuilder::<()>::new().build()?;

    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .with_title(&gui_config.window_title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                gui_config.window_width as f64,
                gui_config.window_height as f64,
            ))
            .with_resizable(gui_config.resizable)
            .build(&event_loop)?
    );

    // Create wgpu instance
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    // Create surface
    let surface = instance.create_surface(window.clone())?;
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .ok_or_else(|| anyhow::anyhow!("Failed to find suitable GPU adapter"))?;

    // Create device and queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("RP2350 Simulator Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await?;

    // Configure surface
    let size = window.inner_size();
    let mut surface_config = surface
        .get_default_config(&adapter, size.width, size.height)
        .ok_or_else(|| anyhow::anyhow!("Failed to get surface config"))?;

    if gui_config.vsync {
        surface_config.present_mode = wgpu::PresentMode::AutoVsync;
    }

    surface.configure(&device, &surface_config);

    // Create egui context
    let egui_ctx = egui::Context::default();
    let viewport_id = egui::ViewportId::ROOT;
    let mut egui_winit = egui_winit::State::new(
        egui_ctx.clone(),
        viewport_id,
        &*window,
        None,
        None,
    );

    // Load fonts
    setup_fonts(&egui_ctx);

    // Create egui renderer
    let mut egui_renderer = egui_wgpu::Renderer::new(&device, surface_config.format, None, 1);

    // App state
    let mut sim_running = false;
    let mut pc: u32 = 0;
    let mut cycles: u64 = 0;

    tracing::info!("WinAPI/wgpu GUI initialized");

    // Run event loop
    event_loop.run(move |event, window_target| {
        match event {
            winit::event::Event::WindowEvent { ref event, window_id } if window_id == window.id() => {
                // Handle egui input
                let response = egui_winit.on_window_event(&window, event);

                if response.consumed {
                    return;
                }

                match event {
                    winit::event::WindowEvent::CloseRequested => {
                        window_target.exit();
                    }
                    winit::event::WindowEvent::Resized(new_size) => {
                        surface_config.width = new_size.width.max(1);
                        surface_config.height = new_size.height.max(1);
                        surface.configure(&device, &surface_config);
                    }
                    winit::event::WindowEvent::KeyboardInput { event, .. } => {
                        if event.state == winit::event::ElementState::Pressed {
                            if let winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) = event.physical_key {
                                window_target.exit();
                            }
                        }
                    }
                    _ => {}
                }
            }
            winit::event::Event::AboutToWait => {
                // Request redraw
                window.request_redraw();
            }
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::RedrawRequested,
                window_id,
            } if window_id == window.id() => {
                // Begin frame
                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [surface_config.width, surface_config.height],
                    pixels_per_point: window.scale_factor() as f32,
                };

                let raw_input = egui_winit.take_egui_input(&window);
                let output = egui_ctx.run(raw_input, |ctx| {
                    draw_ui(ctx, &mut sim_running, &mut pc, &mut cycles);
                });

                egui_winit.handle_platform_output(&window, output.platform_output);

                // Render
                if let Ok(frame) = surface.get_current_texture() {
                    let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

                    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });

                    // Clear background
                    {
                        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Clear Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.118,
                                        g: 0.118,
                                        b: 0.141,
                                        a: 1.0,
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });
                    }

                    // Draw egui
                    let triangles = egui_ctx.tessellate(output.shapes, 1.0);
                    for (id, image_delta) in &output.textures_delta.set {
                        egui_renderer.update_texture(&device, &queue, *id, image_delta);
                    }
                    egui_renderer.update_buffers(&device, &queue, &mut encoder, &triangles, &screen_descriptor);
                    
                    // Create render pass for egui
                    {
                        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Egui Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });
                        egui_renderer.render(&mut render_pass, &triangles, &screen_descriptor);
                    }

                    queue.submit(std::iter::once(encoder.finish()));
                    frame.present();

                    // Free textures
                    for id in &output.textures_delta.free {
                        egui_renderer.free_texture(id);
                    }
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}

/// Draw the UI.
fn draw_ui(ctx: &egui::Context, sim_running: &mut bool, pc: &mut u32, cycles: &mut u64) {
    // Menu bar
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Load Binary...").clicked() {
                    ui.close_menu();
                }
                if ui.button("Exit").clicked() {
                    ui.close_menu();
                }
            });
            ui.menu_button("Run", |ui| {
                if ui.button("Start").clicked() {
                    *sim_running = true;
                    ui.close_menu();
                }
                if ui.button("Pause").clicked() {
                    *sim_running = false;
                    ui.close_menu();
                }
                if ui.button("Reset").clicked() {
                    *sim_running = false;
                    *pc = 0;
                    *cycles = 0;
                    ui.close_menu();
                }
            });
            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    ui.close_menu();
                }
            });
        });
    });

    // Toolbar
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("▶ Start").clicked() {
                *sim_running = true;
            }
            if ui.button("⏸ Pause").clicked() {
                *sim_running = false;
            }
            if ui.button("⏹ Reset").clicked() {
                *sim_running = false;
                *pc = 0;
                *cycles = 0;
            }
            ui.separator();
            ui.label(format!("PC: 0x{:08X}", pc));
            ui.separator();
            ui.label(format!("Cycles: {}", cycles));
        });
    });

    // Main panel
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Left panel
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.heading("Virtual Board");
                    ui.separator();

                    // LED
                    ui.horizontal(|ui| {
                        ui.label("LED (GPIO25):");
                        let led_color = Color32::from_rgb(50, 50, 50);
                        let (rect, _) = ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::hover());
                        ui.painter().circle_filled(rect.center(), 8.0, led_color);
                    });

                    ui.add_space(5.0);

                    // Buttons
                    ui.label("Virtual Buttons:");
                    ui.horizontal(|ui| {
                        for i in 0..4 {
                            if ui.small_button(format!("BTN{}", i)).clicked() {}
                        }
                    });

                    ui.add_space(5.0);

                    // GPIO indicators
                    ui.label("GPIO Status:");
                    ui.horizontal(|ui| {
                        for _pin in 0..16 {
                            let color = Color32::from_rgb(40, 40, 50);
                            let (rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 2.0, color);
                        }
                    });
                });

                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.heading("UART Terminal");
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .show(ui, |ui| {
                            ui.monospace("RP2350 Simulator v0.1.0\n> _");
                        });
                });
            });

            ui.separator();

            // Right panel
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.heading("CPU Registers");
                    ui.separator();

                    egui::Grid::new("registers").show(ui, |ui| {
                        for i in 0..8 {
                            ui.label(format!("R{}", i));
                            ui.monospace("0x00000000");
                            ui.label(format!("R{}", i + 8));
                            ui.monospace("0x00000000");
                            ui.end_row();
                        }
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Flags:");
                        ui.label(egui::RichText::new("N").color(Color32::DARK_GRAY));
                        ui.label(egui::RichText::new("Z").color(Color32::DARK_GRAY));
                        ui.label(egui::RichText::new("C").color(Color32::DARK_GRAY));
                        ui.label(egui::RichText::new("V").color(Color32::DARK_GRAY));
                    });
                });

                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.heading("GPIO Pins");
                    ui.separator();

                    egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                        egui::Grid::new("gpio_pins").show(ui, |ui| {
                            for pin in 0..15 {
                                ui.label(format!("GPIO{}", pin));
                                ui.label(egui::RichText::new("0").color(Color32::DARK_GRAY));
                                if (pin + 1) % 3 == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                    });
                });
            });
        });
    });

    // Status bar
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Core: ARM Cortex-M33");
            ui.separator();
            let status = if *sim_running {
                egui::RichText::new("Running").color(Color32::GREEN)
            } else {
                egui::RichText::new("Stopped").color(Color32::RED)
            };
            ui.label(status);
            ui.separator();
            ui.label("Backend: WinAPI/wgpu");
        });
    });
}

/// Setup fonts.
fn setup_fonts(egui_ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Try to load system fonts on Windows
    #[cfg(target_os = "windows")]
    {
        let fonts_dir = std::path::Path::new(r"C:\Windows\Fonts");

        // Load Consolas for monospace
        let consolas = fonts_dir.join("consola.ttf");
        if consolas.exists() {
            if let Ok(data) = std::fs::read(&consolas) {
                fonts.font_data.insert(
                    "Consolas".to_owned(),
                    egui::FontData::from_owned(data),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, "Consolas".to_owned());
                tracing::info!("Loaded font: consola.ttf");
            }
        }

        // Load Segoe UI for proportional
        let segoe = fonts_dir.join("segoeui.ttf");
        if segoe.exists() {
            if let Ok(data) = std::fs::read(&segoe) {
                fonts.font_data.insert(
                    "SegoeUI".to_owned(),
                    egui::FontData::from_owned(data),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "SegoeUI".to_owned());
                tracing::info!("Loaded font: segoeui.ttf");
            }
        }
    }

    egui_ctx.set_fonts(fonts);
}