/// Grand staff notation renderer for SATB drills.
///
/// Draws treble + bass staff with noteheads, stems, barlines, and playhead.
/// Soprano and Tenor get stems up; Alto and Bass get stems down.

use eframe::egui;
use crate::abc::ScoreEvent;

// ── Layout constants ──
const STAFF_LINE_SPACING: f32 = 14.0;
const STAFF_LINES: usize = 5;
const GRAND_STAFF_GAP: f32 = 84.0;
const NOTEHEAD_RX: f32 = 8.0;
const NOTEHEAD_RY: f32 = 5.5;
const STEM_LENGTH: f32 = 42.0;
const FLAG_LENGTH: f32 = 18.0;
const LEDGER_EXTEND: f32 = 6.0;
pub const BEAT_WIDTH: f32 = 60.0;
const NOTE_WIDTH: f32 = 42.0;
const BAR_WIDTH: f32 = 15.0;

// ── Colors ──
const STAFF_COLOR: egui::Color32 = egui::Color32::from_rgb(180, 180, 180);
const NOTE_COLOR: egui::Color32 = egui::Color32::from_rgb(42, 42, 42);
const ACTIVE_COLOR: egui::Color32 = egui::Color32::from_rgb(37, 99, 235);
const PLAYHEAD_COLOR: egui::Color32 = egui::Color32::from_rgba_premultiplied(37, 99, 235, 80);
const BAR_COLOR: egui::Color32 = egui::Color32::from_rgb(160, 160, 160);
const REST_COLOR: egui::Color32 = egui::Color32::from_rgb(120, 120, 120);

// ── Staff position helpers ──

/// MIDI note to diatonic staff position. Middle C (60) = 0.
const DIATONIC_IN_OCTAVE: [i32; 12] = [0, 0, 1, 1, 2, 3, 3, 4, 4, 5, 5, 6];

pub fn midi_to_staff_pos(midi: i32) -> i32 {
    let octave = midi / 12 - 5;
    let pc = (midi % 12) as usize;
    octave * 7 + DIATONIC_IN_OCTAVE[pc]
}

/// Y coordinate on treble staff. Top line = F5 (pos 10).
fn treble_y(staff_pos: i32, treble_top_y: f32) -> f32 {
    let top_line_pos = 10;
    treble_top_y + (top_line_pos - staff_pos) as f32 * (STAFF_LINE_SPACING / 2.0)
}

/// Y coordinate on bass staff. Top line = A3 (pos -2).
fn bass_y(staff_pos: i32, bass_top_y: f32) -> f32 {
    let top_line_pos = -2;
    bass_top_y + (top_line_pos - staff_pos) as f32 * (STAFF_LINE_SPACING / 2.0)
}

// ── Layout ──

pub struct NotationLayout {
    pub treble_top_y: f32,
    pub bass_top_y: f32,
    pub total_height: f32,
    pub left_margin: f32,
}

impl NotationLayout {
    pub fn new(top_y: f32, left_margin: f32) -> Self {
        let treble_top_y = top_y + 20.0; // small gap from top
        let treble_height = (STAFF_LINES - 1) as f32 * STAFF_LINE_SPACING;
        let bass_top_y = treble_top_y + treble_height + GRAND_STAFF_GAP;
        let bass_height = (STAFF_LINES - 1) as f32 * STAFF_LINE_SPACING;
        let total_height = bass_top_y + bass_height - top_y + 20.0;

        Self { treble_top_y, bass_top_y, total_height, left_margin }
    }

    pub fn prop_x(&self, cumulative_x: f32, scroll_offset: f32) -> f32 {
        self.left_margin + cumulative_x - scroll_offset
    }
}

// ── Drawing primitives ──

fn draw_ledger_lines(painter: &egui::Painter, x: f32, pos: i32, on_treble: bool, layout: &NotationLayout) {
    let lw = NOTEHEAD_RX + LEDGER_EXTEND;

    if on_treble {
        if pos <= 0 {
            let mut p = 0;
            while p >= pos {
                let y = treble_y(p, layout.treble_top_y);
                painter.line_segment(
                    [egui::Pos2::new(x - lw, y), egui::Pos2::new(x + lw, y)],
                    egui::Stroke::new(1.0, STAFF_COLOR),
                );
                p -= 2;
            }
        }
        if pos >= 12 {
            let mut p = 12;
            while p <= pos + 1 {
                let y = treble_y(p, layout.treble_top_y);
                painter.line_segment(
                    [egui::Pos2::new(x - lw, y), egui::Pos2::new(x + lw, y)],
                    egui::Stroke::new(1.0, STAFF_COLOR),
                );
                p += 2;
            }
        }
    } else {
        if pos >= 0 {
            let mut p = 0;
            while p <= pos + 1 {
                let y = bass_y(p, layout.bass_top_y);
                painter.line_segment(
                    [egui::Pos2::new(x - lw, y), egui::Pos2::new(x + lw, y)],
                    egui::Stroke::new(1.0, STAFF_COLOR),
                );
                p += 2;
            }
        }
        if pos <= -12 {
            let mut p = -12;
            while p >= pos {
                let y = bass_y(p, layout.bass_top_y);
                painter.line_segment(
                    [egui::Pos2::new(x - lw, y), egui::Pos2::new(x + lw, y)],
                    egui::Stroke::new(1.0, STAFF_COLOR),
                );
                p -= 2;
            }
        }
    }
}

fn draw_notehead(painter: &egui::Painter, x: f32, y: f32, filled: bool, color: egui::Color32) {
    let n_points = 16;
    let tilt = -0.3f32;
    let points: Vec<egui::Pos2> = (0..n_points).map(|i| {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / n_points as f32;
        let ex = NOTEHEAD_RX * angle.cos();
        let ey = NOTEHEAD_RY * angle.sin();
        let rx = ex * tilt.cos() - ey * tilt.sin();
        let ry = ex * tilt.sin() + ey * tilt.cos();
        egui::Pos2::new(x + rx, y + ry)
    }).collect();

    if filled {
        painter.add(egui::Shape::convex_polygon(points, color, egui::Stroke::NONE));
    } else {
        painter.add(egui::Shape::convex_polygon(points, egui::Color32::TRANSPARENT, egui::Stroke::new(1.5, color)));
    }
}

fn draw_note_stem_up(
    painter: &egui::Painter, layout: &NotationLayout,
    x: f32, pos: i32, on_treble: bool, beats: f32, is_active: bool,
) {
    let color = if is_active { ACTIVE_COLOR } else { NOTE_COLOR };
    let y = if on_treble { treble_y(pos, layout.treble_top_y) } else { bass_y(pos, layout.bass_top_y) };
    let filled = beats < 2.0;
    let has_stem = beats < 4.0;
    let flags = if beats <= 0.25 { 2 } else if beats <= 0.5 { 1 } else { 0 };

    draw_ledger_lines(painter, x, pos, on_treble, layout);
    draw_notehead(painter, x, y, filled, color);

    if has_stem {
        let stem_x = x + NOTEHEAD_RX - 0.5;
        let stem_y2 = y - STEM_LENGTH;
        painter.line_segment(
            [egui::Pos2::new(stem_x, y), egui::Pos2::new(stem_x, stem_y2)],
            egui::Stroke::new(1.2, color),
        );
        for f in 0..flags {
            let flag_y = stem_y2 + f as f32 * 6.0;
            let pts = [
                egui::Pos2::new(stem_x, flag_y),
                egui::Pos2::new(stem_x + 8.0, flag_y + FLAG_LENGTH * 0.5),
                egui::Pos2::new(stem_x + 4.0, flag_y + FLAG_LENGTH),
            ];
            painter.line_segment([pts[0], pts[1]], egui::Stroke::new(1.5, color));
            painter.line_segment([pts[1], pts[2]], egui::Stroke::new(1.5, color));
        }
    }
}

fn draw_note_stem_down(
    painter: &egui::Painter, layout: &NotationLayout,
    x: f32, pos: i32, on_treble: bool, beats: f32, is_active: bool,
) {
    let color = if is_active { ACTIVE_COLOR } else { NOTE_COLOR };
    let y = if on_treble { treble_y(pos, layout.treble_top_y) } else { bass_y(pos, layout.bass_top_y) };
    let filled = beats < 2.0;
    let has_stem = beats < 4.0;
    let flags = if beats <= 0.25 { 2 } else if beats <= 0.5 { 1 } else { 0 };

    draw_ledger_lines(painter, x, pos, on_treble, layout);
    draw_notehead(painter, x, y, filled, color);

    if has_stem {
        let stem_x = x - NOTEHEAD_RX + 0.5;
        let stem_y2 = y + STEM_LENGTH;
        painter.line_segment(
            [egui::Pos2::new(stem_x, y), egui::Pos2::new(stem_x, stem_y2)],
            egui::Stroke::new(1.2, color),
        );
        for f in 0..flags {
            let flag_y = stem_y2 - f as f32 * 6.0;
            let pts = [
                egui::Pos2::new(stem_x, flag_y),
                egui::Pos2::new(stem_x - 8.0, flag_y - FLAG_LENGTH * 0.5),
                egui::Pos2::new(stem_x - 4.0, flag_y - FLAG_LENGTH),
            ];
            painter.line_segment([pts[0], pts[1]], egui::Stroke::new(1.5, color));
            painter.line_segment([pts[1], pts[2]], egui::Stroke::new(1.5, color));
        }
    }
}

fn draw_staff(painter: &egui::Painter, layout: &NotationLayout, width: f32) {
    let x0 = 0.0;
    let x1 = width;

    for i in 0..STAFF_LINES {
        let y = layout.treble_top_y + i as f32 * STAFF_LINE_SPACING;
        painter.line_segment(
            [egui::Pos2::new(x0, y), egui::Pos2::new(x1, y)],
            egui::Stroke::new(1.0, STAFF_COLOR),
        );
    }

    for i in 0..STAFF_LINES {
        let y = layout.bass_top_y + i as f32 * STAFF_LINE_SPACING;
        painter.line_segment(
            [egui::Pos2::new(x0, y), egui::Pos2::new(x1, y)],
            egui::Stroke::new(1.0, STAFF_COLOR),
        );
    }

    // Brace connecting staves
    let top = layout.treble_top_y;
    let bottom = layout.bass_top_y + (STAFF_LINES - 1) as f32 * STAFF_LINE_SPACING;
    painter.line_segment(
        [egui::Pos2::new(x0, top), egui::Pos2::new(x0, bottom)],
        egui::Stroke::new(2.0, NOTE_COLOR),
    );
}

fn draw_clefs(painter: &egui::Painter, layout: &NotationLayout) {
    // Draw stylized "G" and "F" clef labels (Unicode music symbols don't render on Android)
    let treble_center_y = layout.treble_top_y + 2.0 * STAFF_LINE_SPACING;
    painter.text(
        egui::Pos2::new(8.0, treble_center_y),
        egui::Align2::LEFT_CENTER,
        "G",
        egui::FontId::proportional(32.0),
        NOTE_COLOR,
    );

    let bass_center_y = layout.bass_top_y + 1.5 * STAFF_LINE_SPACING;
    painter.text(
        egui::Pos2::new(8.0, bass_center_y),
        egui::Align2::LEFT_CENTER,
        "F",
        egui::FontId::proportional(32.0),
        NOTE_COLOR,
    );
}

fn draw_rest(painter: &egui::Painter, x: f32, beats: f32, layout: &NotationLayout) {
    // Draw rests as simple geometric shapes (Unicode rest symbols don't render on Android)
    let staff_mid = layout.treble_top_y + 2.0 * STAFF_LINE_SPACING;

    if beats >= 4.0 {
        // Whole rest: filled rectangle hanging from line 4
        let line4_y = layout.treble_top_y + STAFF_LINE_SPACING;
        let rect = egui::Rect::from_min_size(
            egui::Pos2::new(x - 6.0, line4_y),
            egui::Vec2::new(12.0, STAFF_LINE_SPACING * 0.5),
        );
        painter.rect_filled(rect, 0.0, REST_COLOR);
    } else if beats >= 2.0 {
        // Half rest: filled rectangle sitting on line 3
        let line3_y = layout.treble_top_y + 2.0 * STAFF_LINE_SPACING;
        let rect = egui::Rect::from_min_size(
            egui::Pos2::new(x - 6.0, line3_y - STAFF_LINE_SPACING * 0.5),
            egui::Vec2::new(12.0, STAFF_LINE_SPACING * 0.5),
        );
        painter.rect_filled(rect, 0.0, REST_COLOR);
    } else {
        // Quarter/eighth rest: simple text dash
        painter.text(
            egui::Pos2::new(x, staff_mid),
            egui::Align2::CENTER_CENTER,
            "-",
            egui::FontId::monospace(18.0),
            REST_COLOR,
        );
    }
}

fn draw_barline(painter: &egui::Painter, x: f32, layout: &NotationLayout) {
    let top = layout.treble_top_y;
    let bottom = layout.bass_top_y + (STAFF_LINES - 1) as f32 * STAFF_LINE_SPACING;
    painter.line_segment(
        [egui::Pos2::new(x, top), egui::Pos2::new(x, bottom)],
        egui::Stroke::new(1.0, BAR_COLOR),
    );
}

fn draw_playhead(painter: &egui::Painter, x: f32, layout: &NotationLayout) {
    let top = layout.treble_top_y - 10.0;
    let bottom = layout.bass_top_y + (STAFF_LINES - 1) as f32 * STAFF_LINE_SPACING + 10.0;

    let rect = egui::Rect::from_min_max(
        egui::Pos2::new(x - 3.0, top),
        egui::Pos2::new(x + 3.0, bottom),
    );
    painter.rect_filled(rect, 0.0, PLAYHEAD_COLOR);

    painter.line_segment(
        [egui::Pos2::new(x, top), egui::Pos2::new(x, bottom)],
        egui::Stroke::new(2.0, ACTIVE_COLOR),
    );
}

// ── Main render function ──

pub fn render_score(
    painter: &egui::Painter,
    layout: &NotationLayout,
    events: &[ScoreEvent],
    scroll_offset: f32,
    current_beat: f32,
    view_width: f32,
) {
    draw_staff(painter, layout, view_width);
    draw_clefs(painter, layout);

    // Build cumulative x positions and beat->x mapping for proportional spacing
    let mut cum_x_positions: Vec<f32> = Vec::new();
    let mut beat_to_x: Vec<(f32, f32)> = Vec::new();
    {
        let mut cx = 0.0f32;
        let mut bt = 0.0f32;
        for event in events {
            match event {
                ScoreEvent::Note { beats, .. } => {
                    beat_to_x.push((bt, cx));
                    cum_x_positions.push(cx);
                    cx += NOTE_WIDTH;
                    bt += beats;
                }
                ScoreEvent::Rest { beats } => {
                    beat_to_x.push((bt, cx));
                    cum_x_positions.push(cx);
                    cx += NOTE_WIDTH;
                    bt += beats;
                }
                ScoreEvent::Bar => {
                    cum_x_positions.push(cx);
                    cx += BAR_WIDTH;
                }
            }
        }
        beat_to_x.push((bt, cx));
    }

    let beat_to_cx = |beat: f32| -> f32 {
        for w in beat_to_x.windows(2) {
            let (bt0, cx0) = w[0];
            let (bt1, cx1) = w[1];
            if beat >= bt0 && beat < bt1 {
                let frac = if bt1 > bt0 { (beat - bt0) / (bt1 - bt0) } else { 0.0 };
                return cx0 + frac * (cx1 - cx0);
            }
        }
        beat_to_x.last().map(|&(_, cx)| cx).unwrap_or(0.0)
    };

    // Convert beat-based scroll to proportional scroll
    let scroll_beat = scroll_offset / BEAT_WIDTH;
    let prop_scroll = beat_to_cx(scroll_beat);

    // Draw playhead
    let playhead_cx = beat_to_cx(current_beat);
    let playhead_x = if current_beat < 0.01 { 10.0 } else { layout.prop_x(playhead_cx, prop_scroll) };
    if playhead_x > 0.0 && playhead_x < view_width {
        draw_playhead(painter, playhead_x, layout);
    }

    // Draw events
    let mut beat_time: f32 = 0.0;

    for (ei, event) in events.iter().enumerate() {
        let cx = cum_x_positions.get(ei).copied().unwrap_or(0.0);
        let x = layout.prop_x(cx, prop_scroll);

        match event {
            ScoreEvent::Note { voices, beats } => {
                let is_active = current_beat >= beat_time && current_beat < beat_time + beats;

                if x > -50.0 && x < view_width + 50.0 {
                    // Voice 0 = Soprano: treble staff, stems up
                    if let Some(&midi) = voices.get(0) {
                        if midi > 0 {
                            let pos = midi_to_staff_pos(midi);
                            draw_note_stem_up(painter, layout, x, pos, true, *beats, is_active);
                        }
                    }
                    // Voice 1 = Alto: treble staff, stems down
                    if let Some(&midi) = voices.get(1) {
                        if midi > 0 {
                            let pos = midi_to_staff_pos(midi);
                            draw_note_stem_down(painter, layout, x, pos, true, *beats, is_active);
                        }
                    }
                    // Voice 2 = Tenor: bass staff, stems up
                    if let Some(&midi) = voices.get(2) {
                        if midi > 0 {
                            let pos = midi_to_staff_pos(midi);
                            draw_note_stem_up(painter, layout, x, pos, false, *beats, is_active);
                        }
                    }
                    // Voice 3 = Bass: bass staff, stems down
                    if let Some(&midi) = voices.get(3) {
                        if midi > 0 {
                            let pos = midi_to_staff_pos(midi);
                            draw_note_stem_down(painter, layout, x, pos, false, *beats, is_active);
                        }
                    }
                }
                beat_time += beats;
            }
            ScoreEvent::Rest { beats } => {
                if x > -50.0 && x < view_width + 50.0 {
                    draw_rest(painter, x, *beats, layout);
                }
                beat_time += beats;
            }
            ScoreEvent::Bar => {
                let bx = x - BAR_WIDTH * 0.5;
                if bx > 0.0 && bx < view_width {
                    draw_barline(painter, bx, layout);
                }
            }
        }
    }
}
