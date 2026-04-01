/// ABC parser for SATB harp drills.
///
/// Parses the `[V: S1V1]` inline voice format used in harp_drills.abc.
/// Produces raw MIDI events per voice — no voicing engine, no chord analysis.

use crate::music::key_sig_accidentals;

// ── Data types ──

#[derive(Debug, Clone)]
pub struct Score {
    pub title: String,
    pub number: String,
    pub key: String,
    pub meter_num: u8,
    pub meter_den: u8,
    pub tempo: u16,
    pub events: Vec<ScoreEvent>,
}

/// A simultaneous beat across all voices.
#[derive(Debug, Clone)]
pub enum ScoreEvent {
    /// Up to 4 MIDI notes sounding together (soprano, alto, tenor, bass).
    /// Missing voices use midi = 0 (rest).
    Note {
        voices: Vec<i32>, // MIDI notes, index 0=soprano, 1=alto, 2=tenor, 3=bass
        beats: f32,
    },
    Rest {
        beats: f32,
    },
    Bar,
}

// ── Embedded ABC data ──

const DRILLS_ABC: &[u8] = include_bytes!("../../data/harp_drills.abc");

pub fn load_scores() -> Vec<Score> {
    let text = String::from_utf8_lossy(DRILLS_ABC);
    parse_all(&text)
}

// ── ABC note parsing ──

const ABC_BASE: [(char, i32); 7] = [
    ('C', 60), ('D', 62), ('E', 64), ('F', 65), ('G', 67), ('A', 69), ('B', 71),
];

fn abc_note_to_midi(token: &str, key_sig_acc: &[i32; 7]) -> Option<i32> {
    let bytes = token.as_bytes();
    let mut acc: Option<i32> = None;
    let mut i = 0;

    while i < bytes.len() && matches!(bytes[i], b'^' | b'_' | b'=') {
        if acc.is_none() { acc = Some(0); }
        match bytes[i] {
            b'^' => *acc.as_mut().unwrap() += 1,
            b'_' => *acc.as_mut().unwrap() -= 1,
            b'=' => acc = Some(0),
            _ => {}
        }
        i += 1;
    }

    if i >= bytes.len() { return None; }
    let ch = bytes[i] as char;
    let is_lower = ch.is_ascii_lowercase();
    let letter = ch.to_ascii_uppercase();

    let base = ABC_BASE.iter().find(|(l, _)| *l == letter)?.1;
    let letter_idx = match letter {
        'C' => 0, 'D' => 1, 'E' => 2, 'F' => 3, 'G' => 4, 'A' => 5, 'B' => 6,
        _ => return None,
    };
    let adjustment = acc.unwrap_or(key_sig_acc[letter_idx]);
    let mut midi = base + adjustment;
    if is_lower { midi += 12; }
    i += 1;

    while i < bytes.len() {
        match bytes[i] {
            b',' => midi -= 12,
            b'\'' => midi += 12,
            _ => break,
        }
        i += 1;
    }

    Some(midi)
}

fn parse_duration(s: &str) -> f32 {
    if s.is_empty() { return 1.0; }
    if let Some(rest) = s.strip_prefix('/') {
        let den: f32 = rest.parse().unwrap_or(2.0);
        return 1.0 / den;
    }
    if s.contains('/') {
        let parts: Vec<&str> = s.splitn(2, '/').collect();
        let num: f32 = parts[0].parse().unwrap_or(1.0);
        let den: f32 = parts[1].parse().unwrap_or(1.0);
        return num / den;
    }
    s.parse().unwrap_or(1.0)
}

// ── Token ──

#[derive(Debug)]
enum Token {
    Bar,
    Rest(f32),
    Note(i32, f32), // midi, duration_multiplier
}

fn tokenize_voice(music: &str, key_sig_acc: &[i32; 7]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = music.chars().peekable();
    let mut note_buf = String::new();
    let mut dur_buf = String::new();

    while let Some(&ch) = chars.peek() {
        if ch == '|' {
            tokens.push(Token::Bar);
            chars.next();
            while chars.peek().is_some_and(|&c| matches!(c, '|' | ']' | '[' | ':')) {
                chars.next();
            }
        } else if ch == 'z' || ch == 'Z' {
            chars.next();
            dur_buf.clear();
            while chars.peek().is_some_and(|c| c.is_ascii_digit() || *c == '/') {
                dur_buf.push(chars.next().unwrap());
            }
            tokens.push(Token::Rest(parse_duration(&dur_buf)));
        } else if matches!(ch, '^' | '_' | '=') || ch.is_ascii_alphabetic() {
            note_buf.clear();
            dur_buf.clear();
            while chars.peek().is_some_and(|&c| matches!(c, '^' | '_' | '=')) {
                note_buf.push(chars.next().unwrap());
            }
            if let Some(&c) = chars.peek() {
                if c.is_ascii_alphabetic() && !"zZxXwWhH".contains(c) {
                    note_buf.push(chars.next().unwrap());
                    while chars.peek().is_some_and(|&c| c == ',' || c == '\'') {
                        note_buf.push(chars.next().unwrap());
                    }
                    while chars.peek().is_some_and(|c| c.is_ascii_digit() || *c == '/') {
                        dur_buf.push(chars.next().unwrap());
                    }
                    if let Some(midi) = abc_note_to_midi(&note_buf, key_sig_acc) {
                        tokens.push(Token::Note(midi, parse_duration(&dur_buf)));
                    }
                } else {
                    chars.next();
                }
            }
        } else if ch == '"' {
            chars.next();
            while chars.peek().is_some_and(|&c| c != '"') { chars.next(); }
            chars.next();
        } else if ch == '!' {
            // Skip decorations like !accent!
            chars.next();
            while chars.peek().is_some_and(|&c| c != '!') { chars.next(); }
            if chars.peek().is_some() { chars.next(); }
        } else {
            chars.next();
        }
    }

    tokens
}

// ── Tune parsing ──

fn parse_all(text: &str) -> Vec<Score> {
    let mut scores = Vec::new();

    // Split on "X:" at beginning of line
    let tunes: Vec<&str> = text.split("\nX:").collect();
    for (i, chunk) in tunes.iter().enumerate() {
        let tune_text = if i == 0 {
            // First chunk may have X: at the very start
            if let Some(rest) = chunk.strip_prefix("X:") {
                format!("X:{}", rest)
            } else if let Some(pos) = chunk.find("X:") {
                chunk[pos..].to_string()
            } else {
                continue;
            }
        } else {
            format!("X:{}", chunk)
        };

        if let Some(score) = parse_tune(&tune_text) {
            scores.push(score);
        }
    }

    scores
}

fn parse_tune(text: &str) -> Option<Score> {
    let mut title = String::new();
    let mut number = String::new();
    let mut key = String::from("C");
    let mut meter_num: u8 = 4;
    let mut meter_den: u8 = 4;
    let mut tempo: u16 = 80;
    let mut unit_len: f32 = 0.25; // L:1/4 default

    // Voice music lines: map voice name -> music string
    let mut voice_music: Vec<(String, String)> = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('%') && !line.starts_with("%%") { continue; }

        if let Some(rest) = line.strip_prefix("X:") {
            number = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("T:") {
            title = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("M:") {
            let m = rest.trim();
            if let Some((n, d)) = m.split_once('/') {
                meter_num = n.trim().parse().unwrap_or(4);
                meter_den = d.trim().parse().unwrap_or(4);
            }
        } else if let Some(rest) = line.strip_prefix("L:") {
            let l = rest.trim();
            if let Some((n, d)) = l.split_once('/') {
                let num: f32 = n.trim().parse().unwrap_or(1.0);
                let den: f32 = d.trim().parse().unwrap_or(4.0);
                unit_len = num / den;
            }
        } else if let Some(rest) = line.strip_prefix("K:") {
            key = rest.trim().split_whitespace().next().unwrap_or("C").to_string();
        } else if line.starts_with("[V:") {
            // Inline voice line: [V: S1V1] music...
            // May contain [Q:...] tempo inline
            if let Some(end_bracket) = line.find(']') {
                let voice_tag = &line[3..end_bracket].trim().to_string();
                let voice_name = voice_tag.split_whitespace().next().unwrap_or("").to_string();
                let mut music = line[end_bracket + 1..].to_string();

                // Extract inline tempo [Q:1/4=80]
                while let Some(q_start) = music.find("[Q:") {
                    if let Some(q_end) = music[q_start..].find(']') {
                        let q_str = &music[q_start + 3..q_start + q_end];
                        if let Some(eq) = q_str.find('=') {
                            tempo = q_str[eq + 1..].trim().parse().unwrap_or(tempo);
                        }
                        music = format!("{}{}", &music[..q_start], &music[q_start + q_end + 1..]);
                    } else {
                        break;
                    }
                }

                voice_music.push((voice_name, music));
            }
        } else if let Some(rest) = line.strip_prefix("Q:") {
            // Header tempo: Q:1/4=80
            let q = rest.trim();
            if let Some(eq) = q.find('=') {
                tempo = q[eq + 1..].trim().parse().unwrap_or(tempo);
            }
        }
    }

    if voice_music.is_empty() { return None; }

    let key_acc = key_sig_accidentals(&key);

    // Parse each voice into tokens
    let mut parsed_voices: Vec<Vec<Token>> = Vec::new();
    for (_name, music) in &voice_music {
        parsed_voices.push(tokenize_voice(music, &key_acc));
    }

    // Merge voices into simultaneous events.
    // Walk each voice's token stream in parallel by note index.
    let events = merge_voices(&parsed_voices, unit_len);

    Some(Score {
        title,
        number,
        key,
        meter_num,
        meter_den,
        tempo,
        events,
    })
}

/// Merge parallel voice token streams into ScoreEvents.
/// All voices share the same barline structure, so we advance all voices
/// together note-by-note.
fn merge_voices(voices: &[Vec<Token>], unit_len: f32) -> Vec<ScoreEvent> {
    let num_voices = voices.len();
    let mut indices = vec![0usize; num_voices];
    let mut events = Vec::new();

    // Beats per unit: unit_len is fraction of whole note (e.g. 0.25 for quarter).
    // A quarter note = 1 beat when unit_len = 1/4.
    let beats_per_unit = unit_len * 4.0; // quarter note = 1 beat

    loop {
        // Check if all voices are exhausted
        let any_remaining = indices.iter().enumerate().any(|(v, &idx)| idx < voices[v].len());
        if !any_remaining { break; }

        // Check if the next token in any voice is a Bar
        let any_bar = indices.iter().enumerate().any(|(v, &idx)| {
            matches!(voices[v].get(idx), Some(Token::Bar))
        });

        if any_bar {
            events.push(ScoreEvent::Bar);
            // Advance past Bar in all voices that have one at current position
            for v in 0..num_voices {
                if matches!(voices[v].get(indices[v]), Some(Token::Bar)) {
                    indices[v] += 1;
                }
            }
            continue;
        }

        // Collect next note/rest from each voice
        let mut midis = vec![0i32; num_voices];
        let mut beats = 0.0f32;
        let mut got_note = false;

        for v in 0..num_voices {
            match voices[v].get(indices[v]) {
                Some(Token::Note(midi, dur)) => {
                    midis[v] = *midi;
                    let b = dur * beats_per_unit;
                    if b > beats { beats = b; }
                    got_note = true;
                    indices[v] += 1;
                }
                Some(Token::Rest(dur)) => {
                    midis[v] = 0;
                    let b = dur * beats_per_unit;
                    if b > beats { beats = b; }
                    indices[v] += 1;
                }
                _ => {
                    midis[v] = 0;
                }
            }
        }

        if beats <= 0.0 { beats = 1.0; } // safety

        if got_note {
            events.push(ScoreEvent::Note { voices: midis, beats });
        } else {
            events.push(ScoreEvent::Rest { beats });
        }
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_all_drills() {
        let scores = load_scores();
        assert!(scores.len() >= 9, "Expected at least 9 drills, got {}", scores.len());
        for s in &scores {
            assert!(!s.title.is_empty(), "Drill {} has empty title", s.number);
            assert!(!s.events.is_empty(), "Drill {} has no events", s.number);
        }
    }

    #[test]
    fn test_drill_i_structure() {
        let scores = load_scores();
        let drill_i = scores.iter().find(|s| s.number == "1001").expect("Drill 1001 not found");
        assert_eq!(drill_i.title, "Drill: I Placing (Full)");
        assert_eq!(drill_i.meter_num, 4);
        assert_eq!(drill_i.tempo, 80);

        // Should have notes with 4 voices
        let first_note = drill_i.events.iter().find(|e| matches!(e, ScoreEvent::Note { .. }));
        if let Some(ScoreEvent::Note { voices, .. }) = first_note {
            assert_eq!(voices.len(), 4, "Expected 4 voices");
            // All should be non-zero for the first note
            assert!(voices.iter().all(|&m| m > 0), "All voices should have notes: {:?}", voices);
        } else {
            panic!("No note found in drill I");
        }
    }
}
