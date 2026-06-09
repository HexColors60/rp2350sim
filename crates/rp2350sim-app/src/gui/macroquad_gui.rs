//! Macroquad + egui GUI backend.
//!
//! This is the default GUI backend using macroquad for windowing/input
//! and egui for the immediate mode UI.

use crate::app::App;
use crate::config::Config;
use crate::gui::{GuiBackend, GuiConfig};
use macroquad::prelude::*;

/// System fonts directory on Windows
#[cfg(target_os = "windows")]
const SYSTEM_FONTS_DIR: &str = r"C:\Windows\Fonts";

/// System fonts directories on Linux
#[cfg(target_os = "linux")]
const SYSTEM_FONTS_DIRS: &[&str] = &[
    "/usr/share/fonts",
    "/usr/local/share/fonts",
    "~/.local/share/fonts",
    "~/.fonts",
];

/// System fonts directory on macOS
#[cfg(target_os = "macos")]
const SYSTEM_FONTS_DIRS: &[&str] = &[
    "/System/Library/Fonts",
    "/Library/Fonts",
    "~/Library/Fonts",
];

/// Preferred monospace fonts (in order of preference)
const MONOSPACE_FONTS: &[&str] = &[
    "consola.ttf",      // Consolas
    "couri.ttf",        // Courier New
    "lucon.ttf",        // Lucida Console
    "sourcecodepro-regular.ttf",
    "jetbrainsmono-regular.ttf",
    "firamono-regular.ttf",
];

/// Preferred UI fonts (in order of preference)
const UI_FONTS: &[&str] = &[
    "segoeui.ttf",      // Segoe UI
    "arial.ttf",        // Arial
    "tahoma.ttf",       // Tahoma
    "calibri.ttf",      // Calibri
    "verdana.ttf",      // Verdana
];

/// Macroquad GUI backend.
pub struct MacroquadBackend {
    gui_config: GuiConfig,
}

impl GuiBackend for MacroquadBackend {
    fn init(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            gui_config: GuiConfig::from(config),
        })
    }

    fn run(&mut self, app: &mut App) -> anyhow::Result<()> {
        // Configure macroquad window
        let conf = macroquad::conf::Conf {
            miniquad_conf: miniquad::conf::Conf {
                window_title: self.gui_config.window_title.clone(),
                window_width: self.gui_config.window_width as i32,
                window_height: self.gui_config.window_height as i32,
                window_resizable: self.gui_config.resizable,
                high_dpi: self.gui_config.high_dpi,
                sample_count: 4, // MSAA
                ..Default::default()
            },
            ..Default::default()
        };

        // Start the macroquad event loop
        macroquad::Window::from_config(conf, run_gui_loop(app.clone_config()));

        Ok(())
    }

    fn name() -> &'static str {
        "macroquad"
    }
}

/// Run the GUI loop (async for macroquad).
async fn run_gui_loop(config: Config) {
    // Create application
    let mut app = App::new(config);

    // Mark as running for GUI mode
    app.start_gui();

    // Initialize egui
    let egui_ctx = egui::Context::default();

    // Load system fonts
    setup_fonts(&egui_ctx);

    let mut egui_input = egui::RawInput::default();

    // Track time
    let start_time = std::time::Instant::now();
    let mut last_frame = start_time;

    // Initialize UI state
    app.ui_state().console_output.push("RP2350 Simulator v0.1.0".to_string());
    app.ui_state().console_output.push("Type 'help' for available commands".to_string());
    app.ui_state().console_output.push("".to_string());

    // Main loop
    loop {
        // Calculate delta time
        let now = std::time::Instant::now();
        let delta = now - last_frame;
        last_frame = now;

        // Update egui input
        egui_input.time = Some(start_time.elapsed().as_secs_f64());
        egui_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(screen_width(), screen_height()),
        ));

        // Handle mouse input
        let mouse_pos = mouse_position();
        egui_input.events.push(egui::Event::PointerMoved(egui::Pos2::new(mouse_pos.0, mouse_pos.1)));

        if is_mouse_button_pressed(MouseButton::Left) {
            egui_input.events.push(egui::Event::PointerButton {
                pos: egui::Pos2::new(mouse_pos.0, mouse_pos.1),
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            });
        }
        if is_mouse_button_released(MouseButton::Left) {
            egui_input.events.push(egui::Event::PointerButton {
                pos: egui::Pos2::new(mouse_pos.0, mouse_pos.1),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            });
        }

        // Handle keyboard input
        let modifiers = egui::Modifiers {
            ctrl: is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl),
            shift: is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift),
            alt: is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt),
            ..Default::default()
        };
        
        let mut key_events = Vec::new();
        for key in get_keys_pressed() {
            if let Some(egui_key) = map_key(key) {
                key_events.push(egui::Event::Key {
                    key: egui_key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers,
                });
            }
        }
        
        // Track key releases
        for key in get_keys_released() {
            if let Some(egui_key) = map_key(key) {
                key_events.push(egui::Event::Key {
                    key: egui_key,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers,
                });
            }
        }
        
        egui_input.events.extend(key_events);

        // Handle text input
        for key in get_keys_pressed().into_iter() {
            let c = match key {
                KeyCode::Space => Some(' '),
                KeyCode::Enter => Some('\n'),
                KeyCode::Tab => Some('\t'),
                _ => None,
            };
            if let Some(c) = c {
                egui_input.events.push(egui::Event::Text(c.to_string()));
            }
        }

        // Handle scroll
        let scroll = mouse_wheel();
        if scroll.1 != 0.0 {
            egui_input.events.push(egui::Event::Scroll(egui::vec2(scroll.0, scroll.1)));
        }

        // Step simulation
        if let Err(e) = app.step_simulation() {
            app.ui_state().console_output.push(format!("Error: {}", e));
        }

        // Handle UI events
        app.handle_events();

        // Begin egui frame
        egui_ctx.begin_frame(egui_input.take());

        // Draw UI
        app.draw_ui(&egui_ctx);

        // End egui frame
        let egui_output = egui_ctx.end_frame();
        let clipped_primitives = egui_ctx.tessellate(egui_output.shapes, 1.0);

        // Clear background
        clear_background(Color::from_rgba(30, 30, 35, 255));

        // Draw egui
        draw_egui(&egui_ctx, &clipped_primitives, &egui_output.textures_delta);

        // Draw FPS
        draw_text(
            &format!("FPS: {:.0}", 1.0 / delta.as_secs_f32().max(0.001)),
            10.0,
            10.0,
            16.0,
            Color::from_rgba(150, 150, 150, 255),
        );

        // Check for exit
        if !app.is_running() {
            break;
        }

        // Wait for next frame
        next_frame().await;
    }
}

/// Draw egui primitives using macroquad.
fn draw_egui(
    _ctx: &egui::Context,
    clipped_primitives: &[egui::ClippedPrimitive],
    textures_delta: &egui::TexturesDelta,
) {
    // Handle texture updates
    for (id, image_delta) in &textures_delta.set {
        let _ = (id, image_delta);
    }

    // Draw primitives
    for egui::ClippedPrimitive { clip_rect, primitive } in clipped_primitives {
        match primitive {
            egui::epaint::Primitive::Mesh(mesh) => {
                draw_egui_mesh(mesh, *clip_rect);
            }
            egui::epaint::Primitive::Callback(callback) => {
                let _ = callback;
            }
        }
    }

    // Free textures
    for id in &textures_delta.free {
        let _ = id;
    }
}

/// Draw an egui mesh.
fn draw_egui_mesh(mesh: &egui::epaint::Mesh, _clip_rect: egui::Rect) {
    if mesh.vertices.is_empty() || mesh.indices.is_empty() {
        return;
    }

    for chunk in mesh.indices.chunks(3) {
        if chunk.len() == 3 {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            if i0 < mesh.vertices.len() && i1 < mesh.vertices.len() && i2 < mesh.vertices.len() {
                let v0 = &mesh.vertices[i0];
                let v1 = &mesh.vertices[i1];
                let v2 = &mesh.vertices[i2];

                let c0 = v0.color;
                let c1 = v1.color;
                let c2 = v2.color;

                let avg_color = Color::from_rgba(
                    ((c0.r() as u32 + c1.r() as u32 + c2.r() as u32) / 3) as u8,
                    ((c0.g() as u32 + c1.g() as u32 + c2.g() as u32) / 3) as u8,
                    ((c0.b() as u32 + c1.b() as u32 + c2.b() as u32) / 3) as u8,
                    ((c0.a() as u32 + c1.a() as u32 + c2.a() as u32) / 3) as u8,
                );

                draw_triangle(
                    Vec2::new(v0.pos.x, v0.pos.y),
                    Vec2::new(v1.pos.x, v1.pos.y),
                    Vec2::new(v2.pos.x, v2.pos.y),
                    avg_color,
                );
            }
        }
    }
}

/// Map macroquad key to egui key.
fn map_key(key: KeyCode) -> Option<egui::Key> {
    match key {
        KeyCode::A => Some(egui::Key::A),
        KeyCode::B => Some(egui::Key::B),
        KeyCode::C => Some(egui::Key::C),
        KeyCode::D => Some(egui::Key::D),
        KeyCode::E => Some(egui::Key::E),
        KeyCode::F => Some(egui::Key::F),
        KeyCode::G => Some(egui::Key::G),
        KeyCode::H => Some(egui::Key::H),
        KeyCode::I => Some(egui::Key::I),
        KeyCode::J => Some(egui::Key::J),
        KeyCode::K => Some(egui::Key::K),
        KeyCode::L => Some(egui::Key::L),
        KeyCode::M => Some(egui::Key::M),
        KeyCode::N => Some(egui::Key::N),
        KeyCode::O => Some(egui::Key::O),
        KeyCode::P => Some(egui::Key::P),
        KeyCode::Q => Some(egui::Key::Q),
        KeyCode::R => Some(egui::Key::R),
        KeyCode::S => Some(egui::Key::S),
        KeyCode::T => Some(egui::Key::T),
        KeyCode::U => Some(egui::Key::U),
        KeyCode::V => Some(egui::Key::V),
        KeyCode::W => Some(egui::Key::W),
        KeyCode::X => Some(egui::Key::X),
        KeyCode::Y => Some(egui::Key::Y),
        KeyCode::Z => Some(egui::Key::Z),
        KeyCode::Key0 => Some(egui::Key::Num0),
        KeyCode::Key1 => Some(egui::Key::Num1),
        KeyCode::Key2 => Some(egui::Key::Num2),
        KeyCode::Key3 => Some(egui::Key::Num3),
        KeyCode::Key4 => Some(egui::Key::Num4),
        KeyCode::Key5 => Some(egui::Key::Num5),
        KeyCode::Key6 => Some(egui::Key::Num6),
        KeyCode::Key7 => Some(egui::Key::Num7),
        KeyCode::Key8 => Some(egui::Key::Num8),
        KeyCode::Key9 => Some(egui::Key::Num9),
        KeyCode::Space => Some(egui::Key::Space),
        KeyCode::Enter => Some(egui::Key::Enter),
        KeyCode::Tab => Some(egui::Key::Tab),
        KeyCode::Escape => Some(egui::Key::Escape),
        KeyCode::Backspace => Some(egui::Key::Backspace),
        KeyCode::Delete => Some(egui::Key::Delete),
        KeyCode::Insert => Some(egui::Key::Insert),
        KeyCode::Home => Some(egui::Key::Home),
        KeyCode::End => Some(egui::Key::End),
        KeyCode::PageUp => Some(egui::Key::PageUp),
        KeyCode::PageDown => Some(egui::Key::PageDown),
        KeyCode::Up => Some(egui::Key::ArrowUp),
        KeyCode::Down => Some(egui::Key::ArrowDown),
        KeyCode::Left => Some(egui::Key::ArrowLeft),
        KeyCode::Right => Some(egui::Key::ArrowRight),
        KeyCode::Minus => Some(egui::Key::Minus),
        KeyCode::Equal => Some(egui::Key::Plus),
        KeyCode::LeftBracket => Some(egui::Key::OpenBracket),
        KeyCode::RightBracket => Some(egui::Key::CloseBracket),
        KeyCode::Backslash => Some(egui::Key::Backslash),
        KeyCode::Semicolon => Some(egui::Key::Semicolon),
        KeyCode::Comma => Some(egui::Key::Comma),
        KeyCode::Period => Some(egui::Key::Period),
        KeyCode::Slash => Some(egui::Key::Slash),
        KeyCode::F1 => Some(egui::Key::F1),
        KeyCode::F2 => Some(egui::Key::F2),
        KeyCode::F3 => Some(egui::Key::F3),
        KeyCode::F4 => Some(egui::Key::F4),
        KeyCode::F5 => Some(egui::Key::F5),
        KeyCode::F6 => Some(egui::Key::F6),
        KeyCode::F7 => Some(egui::Key::F7),
        KeyCode::F8 => Some(egui::Key::F8),
        KeyCode::F9 => Some(egui::Key::F9),
        KeyCode::F10 => Some(egui::Key::F10),
        KeyCode::F11 => Some(egui::Key::F11),
        KeyCode::F12 => Some(egui::Key::F12),
        _ => None,
    }
}

/// Load system fonts and configure egui context.
fn setup_fonts(egui_ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Try to load system fonts
    if let Some((mono_name, mono_data)) = find_and_load_font(MONOSPACE_FONTS) {
        tracing::info!("Loaded monospace font: {}", mono_name);
        fonts.font_data.insert(
            "SystemMonospace".to_owned(),
            egui::FontData::from_owned(mono_data),
        );
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "SystemMonospace".to_owned());
    }

    if let Some((ui_name, ui_data)) = find_and_load_font(UI_FONTS) {
        tracing::info!("Loaded UI font: {}", ui_name);
        fonts.font_data.insert(
            "SystemUI".to_owned(),
            egui::FontData::from_owned(ui_data),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "SystemUI".to_owned());
    }

    // Add fallback fonts for better Unicode coverage
    add_fallback_fonts(&mut fonts);

    egui_ctx.set_fonts(fonts);
}

/// Find and load the first available font from a list of candidates.
fn find_and_load_font(candidates: &[&str]) -> Option<(String, Vec<u8>)> {
    #[cfg(target_os = "windows")]
    {
        let fonts_dir = std::path::Path::new(SYSTEM_FONTS_DIR);
        
        for &font_name in candidates {
            let font_path = fonts_dir.join(font_name);
            if font_path.exists() {
                if let Ok(data) = std::fs::read(&font_path) {
                    return Some((font_name.to_string(), data));
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        for fonts_dir_str in SYSTEM_FONTS_DIRS {
            let fonts_dir = shellexpand::tilde(fonts_dir_str);
            let fonts_dir = std::path::Path::new(fonts_dir.as_ref());
            
            for &font_name in candidates {
                if let Some(data) = search_font_recursive(fonts_dir, font_name) {
                    return Some((font_name.to_string(), data));
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        for fonts_dir_str in SYSTEM_FONTS_DIRS {
            let fonts_dir = shellexpand::tilde(fonts_dir_str);
            let fonts_dir = std::path::Path::new(fonts_dir.as_ref());
            
            for &font_name in candidates {
                let font_path = fonts_dir.join(font_name);
                if font_path.exists() {
                    if let Ok(data) = std::fs::read(&font_path) {
                        return Some((font_name.to_string(), data));
                    }
                }
            }
        }
    }

    None
}

/// Search for a font file recursively in a directory.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn search_font_recursive(dir: &std::path::Path, font_name: &str) -> Option<Vec<u8>> {
    if !dir.exists() {
        return None;
    }

    let direct_path = dir.join(font_name);
    if direct_path.exists() {
        if let Ok(data) = std::fs::read(&direct_path) {
            return Some(data);
        }
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(data) = search_font_recursive(&path, font_name) {
                    return Some(data);
                }
            } else if path.file_name().map_or(false, |f| f == font_name) {
                if let Ok(data) = std::fs::read(&path) {
                    return Some(data);
                }
            }
        }
    }

    None
}

/// Add fallback fonts for Unicode coverage.
fn add_fallback_fonts(fonts: &mut egui::FontDefinitions) {
    #[cfg(target_os = "windows")]
    {
        let fonts_dir = std::path::Path::new(SYSTEM_FONTS_DIR);

        let fallback_fonts = [
            ("seguisym.ttf", "SegoeUISymbol"),
            ("seguiemj.ttf", "SegoeUIEmoji"),
            ("simsun.ttc", "SimSun"),
            ("msgothic.ttc", "MSGothic"),
            ("malgun.ttf", "MalgunGothic"),
            ("arialuni.ttf", "ArialUnicode"),
            ("cambria.ttc", "Cambria"),
        ];

        for (filename, font_key) in fallback_fonts {
            let font_path = fonts_dir.join(filename);
            if font_path.exists() {
                if let Ok(data) = std::fs::read(&font_path) {
                    fonts.font_data.insert(
                        font_key.to_owned(),
                        egui::FontData::from_owned(data),
                    );

                    fonts
                        .families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .push(font_key.to_owned());
                    fonts
                        .families
                        .entry(egui::FontFamily::Monospace)
                        .or_default()
                        .push(font_key.to_owned());
                }
            }
        }
    }
}