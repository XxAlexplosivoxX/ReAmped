use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, epaint::{Mesh, Vertex}};
use player_core::audio::viz_source::SharedSamples;
use player_core::config::AppConfig;
use player_core::viz::spectrum::{log_frequency_bands, smooth_spatial, spectrum};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct SpectrumVisualizer {
    state: SpectrumState,
    config: Arc<Mutex<AppConfig>>,
}

#[derive(Clone, Debug)]
pub struct SpectrumState {
    smooth: Vec<f32>,
    max_energy: f32,
}

impl SpectrumVisualizer {
    pub fn new(config: Arc<Mutex<AppConfig>>) -> Self {
        let bars = config.lock().unwrap().spectrum_bars_quantity;

        Self {
            state: SpectrumState {
                smooth: vec![0.0; bars],
                max_energy: 0.01,
            },
            config,
        }
    }

    pub fn draw_spectrum(
        &mut self,
        ui: &mut egui::Ui,
        samples: &SharedSamples,
        r: u8,
        g: u8,
        b: u8,
    ) {

        let (bands_quantity, smooth_enabled, fft_size, spectrum_mode_line, old_style) = {
            let cfg = self.config.lock().unwrap();
            (
                cfg.spectrum_bars_quantity,
                cfg.spectrum_smooth,
                cfg.fft_size,
                cfg.line_mode,
                cfg.old_style,
            )
        };

        if self.state.smooth.len() != bands_quantity {
            self.state = SpectrumState {
                smooth: vec![0.0; bands_quantity],
                max_energy: 0.01,
            };
        }

        if old_style {
            let raw = spectrum(samples.clone(), fft_size);
            let mut bands =
                log_frequency_bands(&raw, bands_quantity, 44100.0, fft_size, 20.0, 8_000.0);

            // --- suavizado ---
            let alpha = 0.65;
            if smooth_enabled {
                bands = smooth_spatial(&bands);
            }

            for (s, &v) in self.state.smooth.iter_mut().zip(bands.iter()) {
                *s = *s * alpha + v * (1.0 - alpha);
            }

            let frame_max = self.state.smooth.iter().copied().fold(0.0, f32::max);

            let attack = 0.25;
            let release = 0.02;

            if frame_max > self.state.max_energy {
                self.state.max_energy = self.state.max_energy * (1.0 - attack) + frame_max * attack;
            } else {
                self.state.max_energy =
                    self.state.max_energy * (1.0 - release) + frame_max * release;
            }
            let size = egui::vec2(ui.available_width(), ui.available_height());
            let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

            let painter = ui.painter_at(rect).with_clip_rect(rect);

            painter.rect_filled(rect, 6.0, Color32::TRANSPARENT);
            let bars = self.state.smooth.len();
            let bar_width = rect.width() / bars as f32;

            let min_h = 2.0;

            for (i, v) in self.state.smooth.iter().enumerate() {
                let norm = v / self.state.max_energy.max(1e-6);

                let h = (norm.clamp(0.0, 1.7).powf(0.7) * rect.height() * 1.0).max(min_h);

                let bar_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.left() + i as f32 * bar_width, rect.bottom() - h),
                    egui::vec2(bar_width - 1.0, h),
                );

                let slant = 0.5;

                let points = vec![
                    Pos2::new(bar_rect.left(), bar_rect.bottom()),
                    Pos2::new(bar_rect.right(), bar_rect.bottom()),
                    Pos2::new(bar_rect.right(), bar_rect.top() + slant),
                    Pos2::new(bar_rect.left(), bar_rect.top() - slant),
                ];

                painter.add(Shape::convex_polygon(
                    points,
                    egui::Color32::from_rgb(r, g, b),
                    egui::Stroke::NONE,
                ));
            }
        } else {
            let raw = spectrum(samples.clone(), fft_size);

            let mut bands =
                log_frequency_bands(&raw, bands_quantity, 44100.0, fft_size, 20.0, 8000.0);

            let alpha = 0.65;
            if smooth_enabled {
                bands = smooth_spatial(&bands);
            }

            for (s, &v) in self.state.smooth.iter_mut().zip(bands.iter()) {
                *s = *s * alpha + v * (1.0 - alpha);
            }

            let frame_max = self.state.smooth.iter().copied().fold(0.0, f32::max);

            let attack = 0.25;
            let release = 0.02;

            if frame_max > self.state.max_energy {
                self.state.max_energy = self.state.max_energy * (1.0 - attack) + frame_max * attack;
            } else {
                self.state.max_energy =
                    self.state.max_energy * (1.0 - release) + frame_max * release;
            }

            let size = egui::vec2(ui.available_width(), ui.available_height());
            let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

            let painter = ui.painter_at(rect).with_clip_rect(rect);

            painter.rect_filled(rect, 6.0, Color32::TRANSPARENT);

            let bars = self.state.smooth.len();
            let bar_width = rect.width() / bars as f32;

            let min_h = 2.0;
            if spectrum_mode_line {
                let mut points: Vec<Pos2> = Vec::with_capacity(bars);

                for (i, v) in self.state.smooth.iter().enumerate() {
                    let norm = v / self.state.max_energy.max(1e-6);

                    let h = (norm.clamp(0.0, 1.7).powf(0.7) * rect.height()).max(min_h);

                    let x = rect.left() + i as f32 * bar_width;
                    let y = rect.bottom() - h;

                    points.push(Pos2::new(x, y));
                }

                for i in 1..points.len() {
                    let norm = self.state.smooth[i] / self.state.max_energy.max(1e-6);
                    let t = norm.clamp(0.0, 1.0);

                    let color = Color32::from_rgb(
                        (r as f32 * t * 1.3) as u8,
                        (g as f32 * (0.6 + t * 0.4)) as u8,
                        (b as f32 * (1.0 - t * 0.4)) as u8,
                    );

                    painter.line_segment([points[i - 1], points[i]], egui::Stroke::new(2.0, color));
                }

                return;
            }

            let mut mesh = Mesh::default();
            let base_color = ui.visuals().text_color(); 
            let peak_color = ui.visuals().widgets.active.bg_fill.linear_multiply(1.2); 
            for (i, v) in self.state.smooth.iter().enumerate() {
                let norm = v / self.state.max_energy.max(1e-6);
                let t = norm.clamp(0.0, 1.0);

                let h = (t.powf(0.7) * rect.height()).max(min_h);

                let x0 = rect.left() + i as f32 * bar_width;
                let x1 = x0 + bar_width;

                let y0 = rect.bottom();
                let y1 = rect.bottom() - h;

                let color = lerp_color(base_color, peak_color, t);

                let base = mesh.vertices.len() as u32;

                mesh.vertices.push(Vertex {
                    pos: Pos2::new(x0, y0),
                    uv: Default::default(),
                    color,
                });

                mesh.vertices.push(Vertex {
                    pos: Pos2::new(x1, y0),
                    uv: Default::default(),
                    color,
                });

                mesh.vertices.push(Vertex {
                    pos: Pos2::new(x1, y1),
                    uv: Default::default(),
                    color,
                });

                mesh.vertices.push(Vertex {
                    pos: Pos2::new(x0, y1),
                    uv: Default::default(),
                    color,
                });

                mesh.indices.extend_from_slice(&[
                    base,
                    base + 1,
                    base + 2,
                    base,
                    base + 2,
                    base + 3,
                ]);
            }

            painter.add(Shape::mesh(mesh));
        }
    }
}

pub fn _draw_waveform(ui: &mut egui::Ui, samples: &[f32], color: egui::Color32) {
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter_at(rect).with_clip_rect(rect);

    let w = rect.width();
    let h = rect.height();
    let center_y = rect.center().y;

    let len = samples.len().max(1);

    let step_x = w / (len - 1) as f32;
    let amp = h * 0.45; // altura usable

    let mut last = None;

    for (i, &s) in samples.iter().enumerate() {
        let x = rect.left() + i as f32 * step_x;
        let y = center_y - s * amp;

        let p = egui::pos2(x, y);

        if let Some(prev) = last {
            painter.line_segment([prev, p], egui::Stroke::new(1.2, color));
        }

        last = Some(p);
    }
    ui.allocate_rect(rect, egui::Sense::hover());
}

pub fn draw_waveform_raw(
    painter: &Painter,
    rect: Rect,
    samples: &[f32],
    color: Color32,
    fg_color: Color32,
) {
    painter.rect_filled(rect, 6.0, fg_color);
    let w = rect.width();
    let h = rect.height();
    let center_y = rect.center().y;

    let len = samples.len().max(1);
    if len < 2 {
        return;
    }

    let step_x = w / (len - 1) as f32;

    let padding = h * 0.15;
    let amp = (h * 0.6) - padding;

    let mut last: Option<egui::Pos2> = None;
    for (i, &s) in samples.iter().enumerate() {
        let x = rect.left() + i as f32 * step_x;

        let s = s.clamp(-1.0, 1.0);

        let y = center_y - s * amp;
        let p = pos2(x, y);

        if let Some(prev) = last {
            painter.line_segment([prev, p], Stroke::new(1.0, color));
        }

        last = Some(p);
    }
}

fn lerp_color(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);

    let r = a.r() as f32 + (b.r() as f32 - a.r() as f32) * t;
    let g = a.g() as f32 + (b.g() as f32 - a.g() as f32) * t;
    let b_ = a.b() as f32 + (b.b() as f32 - a.b() as f32) * t;

    egui::Color32::from_rgb(r as u8, g as u8, b_ as u8)
}
