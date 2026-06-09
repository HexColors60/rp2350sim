//! Bevy game engine GUI backend.
//!
//! This backend uses Bevy for rendering and bevy_egui for the UI.

use crate::app::App;
use crate::config::Config;
use crate::gui::{GuiBackend, GuiConfig};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

/// Bevy GUI backend.
pub struct BevyBackend {
    gui_config: GuiConfig,
}

impl GuiBackend for BevyBackend {
    fn init(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            gui_config: GuiConfig::from(config),
        })
    }

    fn run(&mut self, app: &mut App) -> anyhow::Result<()> {
        // Create Bevy app
        let mut bevy_app = bevy::app::App::new();

        // Configure window
        bevy_app.insert_resource(ClearColor(Color::rgb(0.118, 0.118, 0.141)));

        // Add default plugins with window config
        bevy_app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: self.gui_config.window_title.clone(),
                resolution: WindowResolution::new(
                    self.gui_config.window_width as f32,
                    self.gui_config.window_height as f32,
                ),
                resizable: self.gui_config.resizable,
                ..default()
            }),
            ..default()
        }));

        // Add Egui plugin
        bevy_app.add_plugins(EguiPlugin);

        // Add our systems
        bevy_app.insert_resource(SimulatorState {
            running: true,
            config: app.get_config().clone(),
        });

        bevy_app.add_systems(Startup, setup_system);
        bevy_app.add_systems(Update, ui_system);

        // Run Bevy app
        bevy_app.run();

        Ok(())
    }

    fn name() -> &'static str {
        "bevy"
    }
}

/// Simulator state resource.
#[derive(Resource)]
struct SimulatorState {
    running: bool,
    config: Config,
}

/// Setup system.
fn setup_system(mut commands: Commands) {
    // Setup camera
    commands.spawn(Camera2dBundle::default());

    tracing::info!("Bevy GUI initialized");
}

/// UI system - draws the egui interface.
fn ui_system(
    mut contexts: EguiContexts,
    mut state: ResMut<SimulatorState>,
    mut exit: EventWriter<AppExit>,
) {
    let ctx = contexts.ctx_mut();

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Exit").clicked() {
                    exit.send(AppExit);
                }
            });
            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    ui.close_menu();
                }
            });
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("RP2350 Simulator - Bevy Backend");
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("▶ Start").clicked() {
                state.running = true;
            }
            if ui.button("⏸ Pause").clicked() {
                state.running = false;
            }
            if ui.button("⏹ Reset").clicked() {
                state.running = false;
            }
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.group(|ui| {
                ui.label(egui::RichText::new("CPU Registers").strong());
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
            });

            ui.add_space(10.0);

            ui.group(|ui| {
                ui.label(egui::RichText::new("GPIO Status").strong());
                ui.separator();

                ui.horizontal(|ui| {
                    for pin in 0..16 {
                        let color = if pin % 2 == 0 {
                            egui::Color32::from_rgb(0, 150, 0)
                        } else {
                            egui::Color32::from_rgb(50, 50, 50)
                        };
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, color);
                    }
                });
            });

            ui.add_space(10.0);

            ui.group(|ui| {
                ui.label(egui::RichText::new("UART Terminal").strong());
                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        ui.monospace("RP2350 Simulator v0.1.0\n> _");
                    });
            });
        });
    });

    // Status bar
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Core: ARM Cortex-M33");
            ui.separator();
            ui.label("PC: 0x00000000");
            ui.separator();
            let status = if state.running {
                egui::RichText::new("Running").color(egui::Color32::GREEN)
            } else {
                egui::RichText::new("Stopped").color(egui::Color32::RED)
            };
            ui.label(status);
        });
    });
}