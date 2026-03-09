#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod visualizer;

use color_thief::{ColorFormat, get_palette};
use eframe::egui;
use egui::epaint::{Mesh, Vertex};
use egui::{
    Color32, ColorImage, Context, Pos2, Rect, RichText, Shape, TextureHandle, scroll_area,
    style::HandleShape,
};
use player_core::{
    Player, PlayerCommand, Track,
    metadata::{CoverArt, read_metadata},
    player::Options,
    viz::waveform::waveform,
};
use visualizer::{SpectrumVisualizer, draw_waveform_raw};

use player_core::config::{AppConfig, load_config, save_config};
use std::sync::{Arc, Mutex};
use std::{path::PathBuf, thread::sleep, time::Duration};
use walkdir::WalkDir;

const AUDIO_EXTS: &[&str] = &["mp3", "wav", "flac", "ogg", "opus", "m4a", "aac"];
#[derive(Clone)]
struct PlayerApp {
    player: Player,
    volume: f32,
    visualizer: SpectrumVisualizer,
    cover_texture: Option<egui::TextureHandle>,
    current_track: Option<Track>,
    position: f32,
    palette: Vec<[u8; 3]>,
    palette_sorted: Vec<[u8; 3]>,
    state: String,
    text_color: Color32,
    fullscreen: bool,
    show_settings: bool,
    just_executed: bool,
    config: Arc<Mutex<AppConfig>>,
    rgb1: [f32; 3],
    rgb2: [f32; 3],
    rgb3: [f32; 3],
    show_picker1: bool,
    show_picker2: bool,
    show_picker3: bool,
    search_str: String,
    sort_option: Options,
    _playlist: Option<Vec<Track>>,
}

impl Default for PlayerApp {
    fn default() -> Self {
        let config = Arc::new(Mutex::new(load_config()));
        Self {
            player: Player::new(load_config().volume),
            volume: load_config().volume,
            visualizer: SpectrumVisualizer::new(config.clone()),
            cover_texture: None,
            current_track: None,
            position: 0.0,
            palette: vec![[0, 0, 0], [0, 0, 0], [0, 0, 0]],
            palette_sorted: vec![[0, 0, 0], [0, 0, 0], [0, 0, 0]],
            state: String::from("status: Welcome"),
            text_color: Color32::WHITE,
            fullscreen: false,
            show_settings: false,
            just_executed: true,
            config: config,
            rgb1: [
                load_config().theme.pallete_custom[0][0] as f32 / 255.0,
                load_config().theme.pallete_custom[0][1] as f32 / 255.0,
                load_config().theme.pallete_custom[0][2] as f32 / 255.0,
            ],
            rgb2: [
                load_config().theme.pallete_custom[1][0] as f32 / 255.0,
                load_config().theme.pallete_custom[1][1] as f32 / 255.0,
                load_config().theme.pallete_custom[1][2] as f32 / 255.0,
            ],
            rgb3: [
                load_config().theme.pallete_custom[2][0] as f32 / 255.0,
                load_config().theme.pallete_custom[2][1] as f32 / 255.0,
                load_config().theme.pallete_custom[2][2] as f32 / 255.0,
            ],
            show_picker1: false,
            show_picker2: false,
            show_picker3: false,
            search_str: String::from(""),
            sort_option: Options::Normal,
            _playlist: None,
        }
    }
}

impl PlayerApp {
    fn ensure_cover_loaded(&mut self, ctx: &egui::Context, ovride: bool) {
        let cfg = self.config.lock().unwrap();
        let current_track = {
            let state = self.player.state.lock().unwrap();
            let playing = state.playing;
            drop(state);

            let pl = self.player.playlist();
            let idx = self.player.playlist_idx();

            if playing && !pl.is_empty() {
                Some(pl[idx].clone())
            } else {
                None
            }
        };
        let should_reload = ovride
            || self.current_track.as_ref().map(|t| &t.path)
                != current_track.as_ref().map(|t| &t.path)
            || self.current_track.is_none();

        if should_reload {
            let cover = self.player.cover();
            self.cover_texture = Some(load_cover_texture(ctx, &cover).unwrap());
            self.current_track = current_track;
            if cfg.theme.follow_cover {
                self.palette = extract_palette(cover);
                self.palette_sorted = self.palette.clone();
                self.palette_sorted
                    .sort_by(|a, b| luminance(*a).partial_cmp(&luminance(*b)).unwrap());
            } else {
                self.palette = cfg.theme.pallete_custom.clone();
                self.palette_sorted = cfg.theme.pallete_custom.clone();
                self.palette_sorted
                    .sort_by(|a, b| luminance(*a).partial_cmp(&luminance(*b)).unwrap());
            }
            let palette = self.palette_sorted.clone();
            let panel = Color32::from_rgba_unmultiplied_const(
                palette[2][0],
                palette[2][1],
                palette[2][2],
                100,
            );
            let accent = Color32::from_rgba_unmultiplied_const(
                palette[1][0],
                palette[1][1],
                palette[1][2],
                100,
            );
            let text = Color32::from_rgb(palette[0][0], palette[0][1], palette[0][2]);
            self.text_color = text;

            let mut visuals = egui::Visuals::dark();

            visuals.window_fill = Color32::TRANSPARENT;
            visuals.panel_fill = Color32::TRANSPARENT;
            visuals.extreme_bg_color = Color32::TRANSPARENT;

            visuals.button_frame = true;

            visuals.widgets.noninteractive.bg_fill = panel.linear_multiply(1.05);
            visuals.widgets.noninteractive.fg_stroke.color = text;
            visuals.widgets.noninteractive.weak_bg_fill = panel.linear_multiply(1.05);

            visuals.widgets.inactive.bg_fill = panel.linear_multiply(1.05);
            visuals.widgets.inactive.fg_stroke.color = text;
            visuals.widgets.inactive.weak_bg_fill = panel.linear_multiply(1.05);

            visuals.widgets.hovered.bg_fill = accent.linear_multiply(0.65);
            visuals.widgets.hovered.fg_stroke.color = Color32::WHITE;
            visuals.widgets.hovered.weak_bg_fill = accent.linear_multiply(0.65);

            visuals.widgets.active.bg_fill = accent;
            visuals.override_text_color = Some(text.linear_multiply(1.2));

            visuals.widgets.inactive.bg_stroke.color = accent.linear_multiply(0.8);
            visuals.widgets.active.fg_stroke.color = accent;
            visuals.widgets.hovered.fg_stroke.color = accent;

            visuals.widgets.active.weak_bg_fill = accent;
            visuals.widgets.hovered.weak_bg_fill = accent;
            visuals.selection.bg_fill = text.gamma_multiply(0.9);

            ctx.set_visuals(visuals);
        }
    }
    fn load_library_async(&self) {
        let cfg = self.config.lock().unwrap();
        let sender = self.player.clone();
        let dirs = cfg.music_dirs.clone();
        let sort_option = self.sort_option.clone();

        std::thread::spawn(move || {
            let tracks = scan_music_dirs(&dirs);

            if !tracks.is_empty() {
                sender.send(PlayerCommand::SetPlaylist(tracks));
                sender.send(PlayerCommand::SortBy(sort_option));
            }
        });
    }
    pub fn draw_slanted_vertical_gradient(
        painter: &egui::Painter,
        rect: Rect,
        top_color: Color32,
        bottom_color: Color32,
        angle_deg: f32,
    ) {
        let angle_rad = angle_deg.to_radians();
        let height = rect.height();
        let x_offset = angle_rad.tan() * height;

        let bleed = x_offset.abs();

        let draw_rect = Rect::from_min_max(
            Pos2::new(rect.left() - bleed, rect.top() - bleed),
            Pos2::new(rect.right() + (bleed * 2.0), rect.bottom() + bleed),
        );

        let painter = painter.with_clip_rect(draw_rect);

        let mut mesh = Mesh::default();

        let mut push = |pos: Pos2, color: Color32| -> u32 {
            let idx = mesh.vertices.len() as u32;
            mesh.vertices.push(Vertex {
                pos,
                uv: egui::pos2(0.0, 0.0),
                color,
            });
            idx
        };

        let tl = push(
            Pos2::new(draw_rect.left() + x_offset, draw_rect.top()),
            top_color,
        );
        let tr = push(
            Pos2::new(draw_rect.right() + x_offset, draw_rect.top()),
            top_color,
        );
        let bl = push(
            Pos2::new(draw_rect.left(), draw_rect.bottom()),
            bottom_color,
        );
        let br = push(
            Pos2::new(draw_rect.right(), draw_rect.bottom()),
            bottom_color,
        );

        mesh.indices.extend_from_slice(&[tl, tr, br, tl, br, bl]);

        painter.add(Shape::mesh(mesh));
    }
}

impl eframe::App for PlayerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let palette = self.palette_sorted.clone();

        let panel =
            Color32::from_rgba_unmultiplied_const(palette[2][0], palette[2][1], palette[2][2], 120);
        let accent = panel.clone();
        let accent = accent.gamma_multiply(1.2);
        let text = Color32::from_rgb(palette[0][0], palette[0][1], palette[0][2]);

        let base_width: f32 = 532.0;
        let current_width = ctx.input(|i| i.viewport_rect().width());
        let scale = (current_width / base_width).clamp(0.1, 2.0);

        ctx.set_pixels_per_point(scale * ctx.pixels_per_point());
        // ctx.set_debug_on_hover(true);

        self.ensure_cover_loaded(&ctx, false);
        if self.show_settings {
            let mut show_settings = self.show_settings;

            egui::Window::new("Configuración")
                .movable(true)
                .collapsible(false)
                .resizable(false)
                .open(&mut show_settings)
                .frame({
                    let mut frame = egui::Frame::window(&ctx.style());

                    frame.fill = accent.linear_multiply(1.2);
                    frame.fill = egui::Color32::from_rgba_unmultiplied(
                        frame.fill.r(),
                        frame.fill.g(),
                        frame.fill.b(),
                        210,
                    );

                    frame
                })
                .show(ctx, |ui| {
                    egui::scroll_area::ScrollArea::vertical()
                        .max_height(ui.available_height() - 20.0)
                        .show(ui, |ui| {
                            let style = ui.style_mut();
                            style.animation_time = 2.0;

                            let mut reload_cover = false;
                            let mut reload_library = false;

                            {
                                let mut cfg = self.config.lock().unwrap();

                                ui.heading("General");
                                ui.separator();

                                if ui
                                    .checkbox(
                                        &mut cfg.fullscreen,
                                        "Abrir en pantalla completa por default",
                                    )
                                    .changed()
                                {
                                    save_config(&cfg);
                                }

                                ui.add_space(10.0);
                                ui.heading("Configuración del tema");
                                ui.separator();

                                if ui
                                    .checkbox(
                                        &mut cfg.theme.follow_cover,
                                        "cambiar colores por cover",
                                    )
                                    .changed()
                                {
                                    save_config(&cfg);
                                    reload_cover = true;
                                }

                                if !cfg.theme.follow_cover {
                                    ui.label("primer color de la paleta");

                                    let color = egui::Color32::from_rgb(
                                        (self.rgb1[0] * 255.0) as u8,
                                        (self.rgb1[1] * 255.0) as u8,
                                        (self.rgb1[2] * 255.0) as u8,
                                    );

                                    if ui.colored_label(color, "██████").clicked() {
                                        self.show_picker1 = true;
                                    }

                                    ui.label("segundo color de la paleta");

                                    let color = egui::Color32::from_rgb(
                                        (self.rgb2[0] * 255.0) as u8,
                                        (self.rgb2[1] * 255.0) as u8,
                                        (self.rgb2[2] * 255.0) as u8,
                                    );

                                    if ui.colored_label(color, "██████").clicked() {
                                        self.show_picker2 = true;
                                    }

                                    ui.label("tercer color de la paleta");

                                    let color = egui::Color32::from_rgb(
                                        (self.rgb3[0] * 255.0) as u8,
                                        (self.rgb3[1] * 255.0) as u8,
                                        (self.rgb3[2] * 255.0) as u8,
                                    );

                                    if ui.colored_label(color, "██████").clicked() {
                                        self.show_picker3 = true;
                                    }

                                    let mut open_picker1 = self.show_picker1;
                                    if open_picker1 {
                                        egui::Window::new("")
                                            .title_bar(false)
                                            .resizable(false)
                                            .collapsible(false)
                                            .fixed_size(egui::vec2(200.0, 160.0))
                                            .movable(true)
                                            .frame({
                                                let mut frame = egui::Frame::window(&ctx.style());

                                                frame.fill = accent.linear_multiply(1.2);
                                                frame.fill = egui::Color32::from_rgba_unmultiplied(
                                                    frame.fill.r(),
                                                    frame.fill.g(),
                                                    frame.fill.b(),
                                                    210,
                                                );

                                                frame
                                            })
                                            .show(ctx, |ui| {
                                                ui.vertical_centered(|ui| {
                                                    ui.heading("Color 1");

                                                    ui.add(
                                                        egui::Slider::new(
                                                            &mut self.rgb1[0],
                                                            0.0..=1.0,
                                                        )
                                                        .text("R"),
                                                    );
                                                    ui.add(
                                                        egui::Slider::new(
                                                            &mut self.rgb1[1],
                                                            0.0..=1.0,
                                                        )
                                                        .text("G"),
                                                    );
                                                    ui.add(
                                                        egui::Slider::new(
                                                            &mut self.rgb1[2],
                                                            0.0..=1.0,
                                                        )
                                                        .text("B"),
                                                    );

                                                    ui.add_space(6.0);

                                                    let color = egui::Color32::from_rgb(
                                                        (self.rgb1[0] * 255.0) as u8,
                                                        (self.rgb1[1] * 255.0) as u8,
                                                        (self.rgb1[2] * 255.0) as u8,
                                                    );

                                                    ui.horizontal(|ui| {
                                                        ui.label("Preview");
                                                        ui.colored_label(color, "██████");
                                                    });

                                                    ui.add_space(8.0);

                                                    if ui.button("OK").clicked() {
                                                        cfg.theme.pallete_custom[0] = [
                                                            (self.rgb1[0] * 255.0) as u8,
                                                            (self.rgb1[1] * 255.0) as u8,
                                                            (self.rgb1[2] * 255.0) as u8,
                                                        ];
                                                        save_config(&cfg);
                                                        open_picker1 = false;
                                                    }
                                                });
                                            });
                                    }
                                    self.show_picker1 = open_picker1;

                                    let mut open_picker2 = self.show_picker2;
                                    if open_picker2 {
                                        egui::Window::new("")
                                            .title_bar(false)
                                            .resizable(false)
                                            .collapsible(false)
                                            .movable(true)
                                            .fixed_size(egui::vec2(200.0, 160.0))
                                            .frame({
                                                let mut frame = egui::Frame::window(&ctx.style());

                                                frame.fill = accent.linear_multiply(1.2);
                                                frame.fill = egui::Color32::from_rgba_unmultiplied(
                                                    frame.fill.r(),
                                                    frame.fill.g(),
                                                    frame.fill.b(),
                                                    210,
                                                );

                                                frame
                                            })
                                            .show(ctx, |ui| {
                                                ui.vertical_centered(|ui| {
                                                    ui.heading("Color 2");

                                                    ui.add(
                                                        egui::Slider::new(
                                                            &mut self.rgb2[0],
                                                            0.0..=1.0,
                                                        )
                                                        .text("R"),
                                                    );
                                                    ui.add(
                                                        egui::Slider::new(
                                                            &mut self.rgb2[1],
                                                            0.0..=1.0,
                                                        )
                                                        .text("G"),
                                                    );
                                                    ui.add(
                                                        egui::Slider::new(
                                                            &mut self.rgb2[2],
                                                            0.0..=1.0,
                                                        )
                                                        .text("B"),
                                                    );

                                                    ui.add_space(6.0);

                                                    let color = egui::Color32::from_rgb(
                                                        (self.rgb2[0] * 255.0) as u8,
                                                        (self.rgb2[1] * 255.0) as u8,
                                                        (self.rgb2[2] * 255.0) as u8,
                                                    );

                                                    ui.horizontal(|ui| {
                                                        ui.label("Preview");
                                                        ui.colored_label(color, "██████");
                                                    });

                                                    ui.add_space(8.0);

                                                    if ui.button("OK").clicked() {
                                                        cfg.theme.pallete_custom[1] = [
                                                            (self.rgb2[0] * 255.0) as u8,
                                                            (self.rgb2[1] * 255.0) as u8,
                                                            (self.rgb2[2] * 255.0) as u8,
                                                        ];
                                                        save_config(&cfg);
                                                        open_picker2 = false;
                                                    }
                                                });
                                            });
                                    }
                                    self.show_picker2 = open_picker2;

                                    let mut open_picker3 = self.show_picker3;
                                    if open_picker3 {
                                        egui::Window::new("")
                                            .title_bar(false)
                                            .resizable(false)
                                            .collapsible(false)
                                            .fixed_size(egui::vec2(200.0, 160.0))
                                            .movable(true)
                                            .frame({
                                                let mut frame = egui::Frame::window(&ctx.style());

                                                frame.fill = accent.linear_multiply(1.2);
                                                frame.fill = egui::Color32::from_rgba_unmultiplied(
                                                    frame.fill.r(),
                                                    frame.fill.g(),
                                                    frame.fill.b(),
                                                    210,
                                                );

                                                frame
                                            })
                                            .show(ctx, |ui| {
                                                ui.vertical_centered(|ui| {
                                                    ui.heading("Color 3");

                                                    ui.add(
                                                        egui::Slider::new(
                                                            &mut self.rgb3[0],
                                                            0.0..=1.0,
                                                        )
                                                        .text("R"),
                                                    );
                                                    ui.add(
                                                        egui::Slider::new(
                                                            &mut self.rgb3[1],
                                                            0.0..=1.0,
                                                        )
                                                        .text("G"),
                                                    );
                                                    ui.add(
                                                        egui::Slider::new(
                                                            &mut self.rgb3[2],
                                                            0.0..=1.0,
                                                        )
                                                        .text("B"),
                                                    );

                                                    ui.add_space(6.0);

                                                    let color = egui::Color32::from_rgb(
                                                        (self.rgb3[0] * 255.0) as u8,
                                                        (self.rgb3[1] * 255.0) as u8,
                                                        (self.rgb3[2] * 255.0) as u8,
                                                    );

                                                    ui.horizontal(|ui| {
                                                        ui.label("Preview");
                                                        ui.colored_label(color, "██████");
                                                    });

                                                    ui.add_space(8.0);

                                                    if ui.button("OK").clicked() {
                                                        cfg.theme.pallete_custom[2] = [
                                                            (self.rgb3[0] * 255.0) as u8,
                                                            (self.rgb3[1] * 255.0) as u8,
                                                            (self.rgb3[2] * 255.0) as u8,
                                                        ];
                                                        save_config(&cfg);
                                                        open_picker3 = false;
                                                    }
                                                });
                                            });
                                    }
                                    self.show_picker3 = open_picker3;
                                }

                                ui.add_space(10.0);
                                ui.heading("FFT config");
                                ui.separator();

                                if ui
                                    .add(
                                        egui::Slider::new(&mut cfg.fft_size, 500..=24576)
                                            .text("fft size"),
                                    )
                                    .drag_stopped()
                                {
                                    save_config(&cfg);
                                }
                                let max_bars = if cfg.old_style {128} else {512};
                                if ui
                                    .add(
                                        egui::Slider::new(
                                            &mut cfg.spectrum_bars_quantity,
                                            40..=max_bars,
                                        )
                                        .step_by(1.0)
                                        .text("cantidad de barras del visualizador de espectro"),
                                    )
                                    .drag_stopped()
                                {
                                    save_config(&cfg);
                                }
                                if ui.checkbox(&mut cfg.spectrum_smooth, "Suavizado").changed() {
                                    save_config(&cfg);
                                }
                                if !cfg.old_style {
                                    if ui.checkbox(&mut cfg.line_mode, "Line mode").changed() {
                                        save_config(&cfg);
                                    }
                                }
                                if ui.checkbox(&mut cfg.old_style, "Old style").changed() {
                                    if cfg.old_style {
                                        cfg.line_mode = false;
                                    }
                                    save_config(&cfg);
                                }

                                ui.add_space(10.0);
                                ui.heading("Música local");
                                ui.separator();

                                let changed = draw_music_dirs(ui, &mut cfg);

                                if changed {
                                    save_config(&cfg);
                                    reload_library = true;
                                }
                            }

                            if reload_cover {
                                self.ensure_cover_loaded(&ctx, true);
                            }

                            if reload_library {
                                self.load_library_async();
                            }
                        });
                });

            self.show_settings = show_settings;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter();
            PlayerApp::draw_slanted_vertical_gradient(
                painter,
                rect,
                Color32::from_rgb(palette[2][0], palette[2][1], palette[2][2]),
                Color32::from_rgb(palette[1][0], palette[1][1], palette[1][2]),
                -6.0,
            );
            ui.horizontal(|ui| {
                if let Some(texture) = &self.cover_texture {
                    ui.add(
                        egui::Image::new(texture)
                            .fit_to_exact_size(egui::vec2(150.0, 150.0))
                            .corner_radius(6.0),
                    );
                }
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            let metadata = self.player.state.lock().unwrap().metadata.clone();
                            if let Some(metadata) = metadata {
                                let text =
                                    format!("\"{}\" By: {}", metadata.title, metadata.artist);
                                marquee_text(ui, &text, 40.0, self.text_color.clone());
                            } else {
                                marquee_text(
                                    ui,
                                    "\"ReAmped\" — XxAlexplosivoxX",
                                    40.0,
                                    self.text_color.clone(),
                                );
                            }
                            ui.horizontal(|ui| {
                                let state = self.player.state.lock().unwrap();

                                let shuffle_on = state.shuffle;
                                let repeat_on = state.repeat;
                                let repeat_one_on = state.repeat_one;
                                let play_on = state.playing;

                                drop(state);

                                if ui.add(egui::Button::new("⏮")).clicked() {
                                    self.player.send(PlayerCommand::Prev);
                                    self.ensure_cover_loaded(&ctx, true);
                                }

                                if ui.add(egui::Button::new("⏹")).clicked() {
                                    self.player.send(PlayerCommand::Stop);
                                }

                                if ui
                                    .add(egui::Button::selectable(
                                        play_on,
                                        egui::RichText::new("▶").color(if play_on {
                                            accent.linear_multiply(2.4)
                                        } else {
                                            ui.visuals().text_color()
                                        }),
                                    ))
                                    .clicked()
                                {
                                    self.player.send(PlayerCommand::Play);
                                }

                                if ui
                                    .add(egui::Button::selectable(
                                        !play_on,
                                        egui::RichText::new("⏸").color(if !play_on {
                                            accent.linear_multiply(2.4)
                                        } else {
                                            ui.visuals().text_color()
                                        }),
                                    ))
                                    .clicked()
                                {
                                    self.player.send(PlayerCommand::Pause);
                                }

                                if ui.add(egui::Button::new("⏭")).clicked() {
                                    self.player.send(PlayerCommand::Next);
                                    self.ensure_cover_loaded(&ctx, true);
                                }
                                ui.style_mut().visuals.widgets.noninteractive.bg_stroke =
                                    egui::Stroke::new(1.0, text);
                                ui.separator();

                                if ui
                                    .add(egui::Button::selectable(
                                        shuffle_on,
                                        egui::RichText::new("🔀").color(if shuffle_on {
                                            accent.linear_multiply(2.4)
                                        } else {
                                            ui.visuals().text_color()
                                        }),
                                    ))
                                    .clicked()
                                {
                                    self.player.send(PlayerCommand::ToggleShuffle);
                                }

                                if ui
                                    .add(egui::Button::selectable(
                                        repeat_on,
                                        egui::RichText::new("🔁").color(if repeat_on {
                                            accent.linear_multiply(2.4)
                                        } else {
                                            ui.visuals().text_color()
                                        }),
                                    ))
                                    .clicked()
                                {
                                    self.player.send(PlayerCommand::ToggleRepeat);
                                }

                                if ui
                                    .add(egui::Button::selectable(
                                        repeat_one_on,
                                        egui::RichText::new("🔂").color(if repeat_one_on {
                                            accent.linear_multiply(2.4)
                                        } else {
                                            ui.visuals().text_color()
                                        }),
                                    ))
                                    .clicked()
                                {
                                    self.player.send(PlayerCommand::ToggleRepeatOne);
                                }
                            });
                        });
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                if ui.button("🔄 rescan").clicked() {
                                    self.load_library_async();
                                }
                                if ui.button(if self.fullscreen { "🗖" } else { "🗗" }).clicked()
                                {
                                    self.fullscreen = !self.fullscreen;

                                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                                        self.fullscreen,
                                    ));
                                }
                                if ui.button("⚙").clicked() {
                                    self.show_settings = true;
                                }
                            });
                        });
                    });
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [39.8, 20.5],
                                egui::Label::new(
                                    "🔊 ".to_owned()
                                        + format!("{:.0}%", self.volume * 100.0).as_str(),
                                ),
                            );
                            let resp = ui.add(
                                egui::Slider::new(&mut self.volume, 0.0..=1.0)
                                    .show_value(false)
                                    .step_by(0.01)
                                    .handle_shape(HandleShape::Rect {
                                        aspect_ratio: (1.0),
                                    })
                                    .trailing_fill(true),
                            );
                            {
                                let mut cfg = self.config.lock().unwrap();
                                if resp.changed() {
                                    self.player.send(PlayerCommand::SetVolume(self.volume));
                                    cfg.volume = self.volume;
                                } else if resp.drag_stopped() {
                                    save_config(&cfg);
                                }
                            }
                            if ui
                                .button(
                                    "≡ order: ".to_owned() + self.sort_option.to_string().as_str(),
                                )
                                .clicked()
                            {
                                let sort_option = self.sort_option.clone();
                                match sort_option {
                                    Options::Normal => {
                                        self.sort_option = Options::Alphabetical;
                                    }
                                    Options::Alphabetical => {
                                        self.sort_option = Options::Normal;
                                    }
                                }
                                self.load_library_async();
                            }
                            if ui.button("🔀 Aleatorize").clicked() {
                                self.player.send(PlayerCommand::AleatoryFullRandom);
                            }
                        });
                        if ui
                            .horizontal(|ui| {
                                if !ui
                                    .add_sized(
                                        [ui.available_width() / 3.0, ui.available_height()],
                                        egui::TextEdit::singleline(&mut self.search_str)
                                            .hint_text(
                                                RichText::new("type here to search...")
                                                    .color(
                                                        egui::Color32::from_rgb(
                                                            self.palette_sorted[0][0],
                                                            self.palette_sorted[0][1],
                                                            self.palette_sorted[0][2],
                                                        )
                                                        .linear_multiply(0.5),
                                                    )
                                                    .italics(),
                                            )
                                            .background_color(
                                                egui::Color32::from_rgba_premultiplied(
                                                    self.palette_sorted[1][0],
                                                    self.palette_sorted[1][1],
                                                    self.palette_sorted[1][2],
                                                    100,
                                                )
                                                .linear_multiply(0.5),
                                            ),
                                    )
                                    .contains_pointer()
                                {
                                    // self.search_str = String::from("")
                                }
                                let playlist = self.player.playlist();
                                mini_playlist(
                                    ui,
                                    &playlist,
                                    self.current_track.clone(),
                                    self.player.is_playing(),
                                    accent,
                                    |i| self.player.send(PlayerCommand::JumpTo(i)),
                                    self.position,
                                    self.just_executed,
                                    self.search_str.clone(),
                                );
                            })
                            .response
                            .changed()
                        {
                            sleep(Duration::from_secs(2));
                            self.search_str = String::from("");
                        };
                        let samples = &self.player.samples;
                        let palette = &self.palette_sorted;
                        self.visualizer.draw_spectrum(
                            ui,
                            &samples,
                            palette[0][0],
                            palette[0][1],
                            palette[0][2],
                        );
                    });
                    // ui.add_space(8.0);
                });
            });
            ui.horizontal(|ui| {
                let mut duration = self.player.state.lock().unwrap().duration;

                let available = ui.available_width() - 76.0 - 16.0;

                let has_track = duration > 0.0;
                let mut pos = self.position;
                if !has_track {
                    pos = 0.0;
                    duration = 0.01;
                }
                ui.add_sized(
                    [38.0, 20.5],
                    egui::Label::new(format!(
                        "{:02}:{:02}",
                        pos.clone() as u32 / 60,
                        pos.clone() as u32 % 60
                    )),
                );
                ui.style_mut().spacing.slider_width = available;
                let response = ui.add_enabled(
                    has_track,
                    egui::Slider::new(&mut pos, 0.0..=duration)
                        .show_value(false)
                        .step_by(0.1)
                        .handle_shape(HandleShape::Rect {
                            aspect_ratio: (1.0),
                        })
                        .trailing_fill(true),
                );
                ui.add_sized(
                    [38.0, 20.5],
                    egui::Label::new(format!(
                        "{:02}:{:02}",
                        duration.clone() as u32 / 60,
                        duration.clone() as u32 % 60
                    )),
                );

                if has_track && !response.dragged() {
                    self.position = self.player.position();
                }
                if response.dragged() {
                    self.state = String::from("status: Seeking")
                }
                if response.drag_stopped() {
                    self.player.send(PlayerCommand::Seek(pos));
                    self.state = String::from("status: Playing");
                }
            });

            let height = ui.available_height() - 10.0;

            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), height),
                egui::Sense::hover(),
            );

            let painter = ui.painter_at(rect).with_clip_rect(rect);

            let wave = waveform(self.player.samples.clone(), 4108);
            let palette = &self.palette_sorted;

            draw_waveform_raw(
                &painter,
                rect,
                &wave,
                Color32::from_rgb(palette[0][0], palette[0][1], palette[0][2]).gamma_multiply(0.6),
                Color32::TRANSPARENT,
            );
            self.visualizer.draw_beat_stripes(ui, accent, text);
            if self.player.is_playing() {
                ctx.request_repaint();
                self.state = String::from("status: Playing");
                self.just_executed = false;
            } else {
                ctx.request_repaint();
                self.state = String::from("status: Paused")
            }

            if self.player.playlist().is_empty() {
                self.load_library_async();
            }
        });
    }
}

fn scan_music_dirs(dirs: &[PathBuf]) -> Vec<Track> {
    let mut tracks = Vec::new();

    for dir in dirs {
        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();

            if !AUDIO_EXTS.contains(&ext.as_str()) {
                continue;
            }

            let metadata = read_metadata(path);

            tracks.push(Track {
                path: path.to_path_buf(),
                title: metadata.title,
                artist: metadata.artist,
                duration: metadata.duration,
            });
        }
    }

    tracks
}

fn draw_music_dirs(ui: &mut egui::Ui, config: &mut AppConfig) -> bool {
    let mut changed = false;
    let mut to_remove = None;

    for (i, dir) in config.music_dirs.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(dir.display().to_string());

            if ui.small_button("❌").clicked() {
                to_remove = Some(i);
            }
        });
    }

    if let Some(i) = to_remove {
        config.music_dirs.remove(i);
        save_config(config);
        changed = true;
    }

    if ui.button("➕ Add folder").clicked() {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            config.music_dirs.push(path);
            save_config(config);
            changed = true;
        }
    }

    changed
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "NotoSans".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/NotoSans-VariableFont_wdth,wght.ttf"
        ))
        .into(),
    );

    fonts.font_data.insert(
        "Saira".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/Saira_Condensed-Thin.ttf"
        ))
        .into(),
    );

    fonts.font_data.insert(
        "NotoSans-JP".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/NotoSansJP-VariableFont_wght.ttf"
        ))
        .into(),
    );

    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "Saira".to_owned());

    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(0, "Saira".to_owned());

    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(1, "NotoSans-JP".to_owned());

    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(1, "NotoSans-JP".to_owned());

    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(2, "NotoSans".to_owned());

    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(2, "NotoSans".to_owned());

    ctx.set_fonts(fonts);
}

fn marquee_text(ui: &mut egui::Ui, text: &str, speed: f32, color: Color32) {
    let available_width = ui.available_width() - 122.0;
    let font_id = egui::TextStyle::Body.resolve(ui.style());
    let galley = ui.fonts_mut(|f| f.layout_no_wrap(text.to_owned(), font_id.clone(), color));

    let text_width = galley.size().x;
    let height = galley.size().y;

    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(available_width, height), egui::Sense::hover());

    let painter = ui.painter();

    if text_width <= available_width {
        painter.galley(rect.min, galley, color);
    } else {
        let time = ui.input(|i| i.time) as f32;
        let spacing = 40.0;
        let offset = (time * speed) % (text_width + spacing);

        let clip_painter = painter.with_clip_rect(rect);

        let x = rect.min.x - offset;

        clip_painter.galley(egui::pos2(x, rect.min.y), galley.clone(), color);

        clip_painter.galley(
            egui::pos2(x + text_width + spacing, rect.min.y),
            galley,
            color,
        );
    }
}

pub fn mini_playlist<F>(
    ui: &mut egui::Ui,
    playlist: &[Track],
    current: Option<Track>,
    playing: bool,
    accent: Color32,
    mut on_select: F,
    pos: f32,
    just_executed: bool,
    search_str: String,
) where
    F: FnMut(usize),
{
    let row_height = 18.0;
    let max_rows = 8;
    let height = row_height * max_rows as f32 + 6.0;

    let search = search_str.to_ascii_lowercase();

    egui::Frame::new()
        .fill(Color32::from_black_alpha(25))
        .corner_radius(2.0)
        .show(ui, |ui| {
            egui::ScrollArea::horizontal()
                .max_width(ui.available_width())
                .max_height(height)
                .scroll_bar_visibility(scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    for (i, track) in playlist.iter().enumerate() {
                        if !search.is_empty() && search.len() >= 3 {
                            if !track.title.to_ascii_lowercase().contains(&search) {
                                continue;
                            }
                        }

                        let is_current = current.as_ref().map_or(false, |c| c.path == track.path);

                        let icon = if is_current {
                            if playing { "▶" } else { "⏸" }
                        } else {
                            ""
                        };

                        let label = format!("{} {}", icon, truncate(&track.title, 20));

                        let resp = ui.add(egui::Button::selectable(is_current, label).fill(accent));

                        if is_current && pos < 0.1 && !just_executed {
                            resp.scroll_to_me(Some(egui::Align::Center));
                        }

                        if resp.clicked() {
                            on_select(i);
                        }
                    }
                });
        });
}

pub fn cover_to_color_image(cover: &CoverArt) -> Option<ColorImage> {
    let img = image::load_from_memory(&cover.data).ok()?;

    // Aseguramos formato RGBA8
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    Some(ColorImage::from_rgba_unmultiplied(
        [w as usize, h as usize],
        &rgba,
    ))
}

pub fn load_cover_texture(ctx: &Context, cover: &CoverArt) -> Option<TextureHandle> {
    let image = cover_to_color_image(cover)?;

    Some(ctx.load_texture("cover_art", image, Default::default()))
}

fn extract_palette(cover: CoverArt) -> Vec<[u8; 3]> {
    let img = image::load_from_memory(&cover.data).ok().unwrap();

    // let img = img.resize(128, 128, image::imageops::FilterType::Lanczos3);
    let rgb = img.to_rgb8();
    let pixels = rgb.as_raw();

    let palette = get_palette(pixels, ColorFormat::Rgb, 2, 3).unwrap_or_default();

    palette.into_iter().map(|c| [c.r, c.g, c.b]).collect()
}

fn luminance(c: [u8; 3]) -> f32 {
    0.2126 * c[0] as f32 + 0.7152 * c[1] as f32 + 0.0722 * c[2] as f32
}

fn truncate(text: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (i, c) in text.chars().enumerate() {
        if i >= max_chars {
            out.push('…');
            break;
        }
        out.push(c);
    }
    out
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        vsync: true,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([532.0, 292.0])
            .with_resizable(false)
            .with_decorations(true),
        ..Default::default()
    };

    eframe::run_native(
        "ReAmped",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            let app = self::load_config();
            if app.fullscreen {
                cc.egui_ctx
                    .send_viewport_cmd(egui::ViewportCommand::Fullscreen(app.fullscreen));
            }
            Ok(Box::<PlayerApp>::default())
        }),
    )
}
