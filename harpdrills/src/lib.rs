pub mod abc;
pub mod music;
pub mod notation;

use eframe::egui;
use abc::{Score, ScoreEvent, load_scores};

// ── Colors ──
const BG: egui::Color32 = egui::Color32::from_rgb(248, 246, 241);
const CARD_BG: egui::Color32 = egui::Color32::WHITE;
const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(42, 42, 42);
const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(136, 136, 136);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(37, 99, 235);
const BORDER: egui::Color32 = egui::Color32::from_rgb(204, 204, 204);

pub fn create_native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 1000.0]),
        ..Default::default()
    }
}

pub fn run_app(options: eframe::NativeOptions) -> eframe::Result {
    eframe::run_native(
        "Harp Drills",
        options,
        Box::new(|cc| {
            apply_style(&cc.egui_ctx);
            Ok(Box::new(DrillApp::new()))
        }),
    )
}

fn apply_style(ctx: &egui::Context) {
    ctx.set_pixels_per_point(1.5);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::Vec2::new(6.0, 4.0);
    style.spacing.button_padding = egui::Vec2::new(10.0, 6.0);
    style.visuals.window_fill = BG;
    style.visuals.panel_fill = BG;
    style.visuals.extreme_bg_color = CARD_BG;

    let r8 = egui::CornerRadius::same(8);
    style.visuals.widgets.inactive.corner_radius = r8;
    style.visuals.widgets.hovered.corner_radius = r8;
    style.visuals.widgets.active.corner_radius = r8;

    style.visuals.widgets.inactive.bg_fill = CARD_BG;
    style.visuals.widgets.inactive.weak_bg_fill = CARD_BG;
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(2.0, BORDER);
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);

    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(240, 240, 240);
    style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(240, 240, 240);

    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(219, 234, 254);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, ACCENT);

    style.visuals.selection.bg_fill = egui::Color32::from_rgb(219, 234, 254);
    style.visuals.selection.stroke = egui::Stroke::new(2.0, ACCENT);

    ctx.set_style(style);
}

struct DrillApp {
    scores: Vec<Score>,
    current_score: usize,

    // Playback
    playing: bool,
    current_beat: f32,
    tempo_bpm: f32,
    last_frame_time: Option<f64>,

    // Scroll
    scroll_offset: f32,
    playhead_fraction: f32,
}

impl DrillApp {
    fn new() -> Self {
        let scores = load_scores();
        let tempo = scores.first().map(|s| s.tempo as f32).unwrap_or(80.0);
        Self {
            scores,
            current_score: 0,
            playing: false,
            current_beat: 0.0,
            tempo_bpm: tempo,
            last_frame_time: None,
            scroll_offset: 0.0,
            playhead_fraction: 0.20,
        }
    }

    fn current_events(&self) -> &[ScoreEvent] {
        self.scores.get(self.current_score)
            .map(|s| s.events.as_slice())
            .unwrap_or(&[])
    }

    fn total_beats(&self) -> f32 {
        let mut total = 0.0;
        for ev in self.current_events() {
            match ev {
                ScoreEvent::Note { beats, .. } => total += beats,
                ScoreEvent::Rest { beats } => total += beats,
                ScoreEvent::Bar => {}
            }
        }
        total
    }
}

impl eframe::App for DrillApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Advance playback
        if self.playing {
            let now = ctx.input(|i| i.time);
            if let Some(last) = self.last_frame_time {
                let dt = (now - last) as f32;
                let beats_per_sec = self.tempo_bpm / 60.0;
                self.current_beat += dt * beats_per_sec;

                let total = self.total_beats();
                if self.current_beat >= total {
                    self.current_beat = 0.0;
                    self.playing = false;
                    self.last_frame_time = None;
                }
            }
            self.last_frame_time = Some(now);
            ctx.request_repaint();
        }

        egui::TopBottomPanel::top("title_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let n = self.scores.len();
                let title = self.scores.get(self.current_score)
                    .map(|s| s.title.as_str())
                    .unwrap_or("No drills loaded");

                ui.label(egui::RichText::new(title)
                    .size(18.0)
                    .color(TEXT_PRIMARY)
                    .strong());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("{}/{}", self.current_score + 1, n))
                        .size(14.0)
                        .color(TEXT_MUTED));
                });
            });
        });

        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button(egui::RichText::new("<< Prev").size(16.0)).clicked() {
                    if self.current_score > 0 {
                        self.current_score -= 1;
                    } else {
                        self.current_score = self.scores.len().saturating_sub(1);
                    }
                    self.current_beat = 0.0;
                    self.playing = false;
                    self.last_frame_time = None;
                    if let Some(s) = self.scores.get(self.current_score) {
                        self.tempo_bpm = s.tempo as f32;
                    }
                }

                let play_label = if self.playing { "Pause" } else { "Play" };
                if ui.button(egui::RichText::new(play_label).size(16.0)).clicked() {
                    self.playing = !self.playing;
                    if self.playing {
                        self.last_frame_time = None;
                        if self.current_beat >= self.total_beats() {
                            self.current_beat = 0.0;
                        }
                    }
                }

                if ui.button(egui::RichText::new("Next >>").size(16.0)).clicked() {
                    if self.current_score < self.scores.len().saturating_sub(1) {
                        self.current_score += 1;
                    } else {
                        self.current_score = 0;
                    }
                    self.current_beat = 0.0;
                    self.playing = false;
                    self.last_frame_time = None;
                    if let Some(s) = self.scores.get(self.current_score) {
                        self.tempo_bpm = s.tempo as f32;
                    }
                }

                ui.add_space(16.0);
                ui.label(egui::RichText::new("BPM:").size(14.0).color(TEXT_MUTED));
                if ui.button(egui::RichText::new("-").size(16.0)).clicked() {
                    self.tempo_bpm = (self.tempo_bpm - 5.0).max(10.0);
                }
                ui.label(egui::RichText::new(format!("{:.0}", self.tempo_bpm))
                    .size(16.0)
                    .color(TEXT_PRIMARY)
                    .strong());
                if ui.button(egui::RichText::new("+").size(16.0)).clicked() {
                    self.tempo_bpm = (self.tempo_bpm + 5.0).min(300.0);
                }
            });
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
            let painter = ui.painter_at(rect);

            let layout = notation::NotationLayout::new(rect.top() + 8.0, 50.0);
            let view_width = rect.width();

            // Drag to scrub when paused
            if !self.playing && response.dragged() {
                let drag_beats = response.drag_delta().x / notation::BEAT_WIDTH;
                self.current_beat = (self.current_beat - drag_beats).max(0.0);
            }

            // Compute scroll offset so playhead is at playhead_fraction of view
            let playhead_screen_x = view_width * self.playhead_fraction;
            self.scroll_offset = (self.current_beat * notation::BEAT_WIDTH - playhead_screen_x).max(0.0);

            let events = self.current_events().to_vec();
            notation::render_score(
                &painter,
                &layout,
                &events,
                self.scroll_offset,
                self.current_beat,
                view_width,
            );
        });
    }
}

// ── Android entry point ──

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let options = eframe::NativeOptions {
        android_app: Some(app),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    run_app(options).unwrap();
}
