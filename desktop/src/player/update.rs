
use egui::{Color32, style::HandleShape};
use player_core::{PlayerCommand, viz::waveform::waveform};

use crate::{
    player::player_app_init::PlayerApp,
    ui_elements::{
        buttons::show_buttons_and_title, config_window::show_config_window, cover_view::show_cover, order_buttons::show_order_buttons, search_and_miniplaylist::show_search_and_miniplaylist, volume_bar::show_volume_bar
    },
    utils::{background::draw_slanted_vertical_gradient, visualizer::draw_waveform_raw},
};

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
        show_config_window(self, ctx, accent);

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter();
            draw_slanted_vertical_gradient(
                painter,
                rect,
                Color32::from_rgb(palette[2][0], palette[2][1], palette[2][2]),
                Color32::from_rgb(palette[1][0], palette[1][1], palette[1][2]),
                -6.0,
            );
            ui.horizontal(|ui| {
                show_cover(ui, self);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            // ui.horizontal(|ui| {
                            //     let plugins = self.player.plugins_info();
                            //     let plugins = plugins.lock().unwrap();
                            //     let value1 = plugins.get_key_value("VU Meter");
                            //     let value2 = plugins.get_key_value("RMS Meter");
                            //     ui.vertical(|ui| {
                            //         if value1.is_some() {
                            //             draw_meter(ui, value1.unwrap().1.clone(), accent, text);
                            //             ui.label(format!("{:.1}", *value1.unwrap().1));
                            //         } else {
                            //             draw_meter(ui, 0.0, accent, text);
                            //             ui.label(format!("{:.1}", 0.0));
                            //         }
                            //     });
                            //     ui.vertical(|ui| {
                            //         if value2.is_some() {
                            //             draw_meter(ui, value2.unwrap().1.clone(), accent, text);
                            //             ui.label(format!("{:.1}", *value2.unwrap().1));
                            //         } else {
                            //             draw_meter(ui, 0.0, accent, text);
                            //             ui.label(format!("{:.1}", 0.0));
                            //         }
                            //     });
                            // });
                            show_buttons_and_title(ui, ctx, self, self.text_color.clone(), accent);
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    show_volume_bar(ui, self);
                                    show_order_buttons(ui, self);
                                });
                                show_search_and_miniplaylist(ui, self, accent);
                            });
                        });
                    });
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