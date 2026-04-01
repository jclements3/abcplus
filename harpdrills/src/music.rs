/// Minimal music theory helpers for ABC parsing.

pub const MAJOR_SCALE: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
pub const PC_NAMES: [&str; 12] = ["C","Db","D","Eb","E","F","F#","G","Ab","A","Bb","B"];

pub fn pitch_class(midi: i32) -> i32 {
    ((midi % 12) + 12) % 12
}

pub fn key_to_pc(key: &str) -> i32 {
    match key.trim() {
        "C" => 0, "Db" | "C#" => 1, "D" => 2, "Eb" | "D#" => 3,
        "E" => 4, "F" => 5, "F#" | "Gb" => 6, "G" => 7,
        "Ab" | "G#" => 8, "A" => 9, "Bb" | "A#" => 10, "B" => 11,
        _ => 0,
    }
}

/// Key signature accidentals per diatonic degree (C D E F G A B).
/// Returns array of semitone adjustments applied by the key signature.
pub fn key_sig_accidentals(key: &str) -> [i32; 7] {
    let k = key.trim();
    match k {
        "C" | "Am" => [0, 0, 0, 0, 0, 0, 0],
        "G" | "Em" => [0, 0, 0, 1, 0, 0, 0],       // F#
        "D" | "Bm" => [1, 0, 0, 1, 0, 0, 0],       // F# C#
        "A" | "F#m" => [1, 0, 0, 1, 1, 0, 0],      // F# C# G#
        "E" | "C#m" => [1, 1, 0, 1, 1, 0, 0],      // F# C# G# D#
        "B" | "G#m" => [1, 1, 0, 1, 1, 1, 0],      // F# C# G# D# A#
        "F#" | "D#m" => [1, 1, 1, 1, 1, 1, 0],     // F# C# G# D# A# E#
        "F" | "Dm" => [0, 0, 0, 0, 0, 0, -1],       // Bb
        "Bb" | "Gm" => [0, 0, -1, 0, 0, 0, -1],    // Bb Eb
        "Eb" | "Cm" => [0, 0, -1, 0, 0, -1, -1],   // Bb Eb Ab
        "Ab" | "Fm" => [0, -1, -1, 0, 0, -1, -1],  // Bb Eb Ab Db
        "Db" | "Bbm" => [0, -1, -1, 0, -1, -1, -1], // Bb Eb Ab Db Gb
        "Gb" | "Ebm" => [-1, -1, -1, 0, -1, -1, -1], // Bb Eb Ab Db Gb Cb
        _ => [0, 0, 0, 0, 0, 0, 0],
    }
}
