#!/usr/bin/env python3
"""Generate Thomas 280 Technical Exercises for the Harp.

Public domain (John Thomas, 1826-1913). Systematic scale and arpeggio
exercises across all major and minor keys. Generated algorithmically
from the pattern structure observed in the original publication.

Each key gets: octave scales, sixth scales, tenth scales, extended
scales, contrary motion, arpeggios, broken chords, and syncopations.
"""

import json
from pathlib import Path

PROJECT_DIR = Path(__file__).parent.parent

# ── Scale definitions ──
# Major scales as ABC note sequences (2 octaves ascending from C3)
MAJOR_SCALES = {
    'C':  'CDEFGAB',
    'G':  'GABcdef',
    'D':  'DEFGABc',
    'A':  'ABcdefg',
    'E':  'EFGABcd',
    'B':  'BCDEFGa',
    'F':  'FGABcde',
    'Bb': 'BcdefgA',
    'Eb': 'EFGABcd',
    'Ab': 'ABcdefg',
    'Db': 'DEFGABc',
    'Gb': 'GABcdef',
}

# For ABC output, we need the actual note names with correct octaves
# Scale degree to ABC note mapping per key
def scale_abc(key, start_oct=3, num_octaves=2):
    """Generate ABC scale notes for a key across octaves."""
    # Major scale intervals in semitones: W W H W W W H
    # But for ABC we use diatonic letter names

    # Key signatures (sharps/flats applied by K: directive)
    # So we just need the 7 diatonic note letters in order
    key_roots = {
        'C': ['C','D','E','F','G','A','B'],
        'G': ['G','A','B','C','D','E','F'],
        'D': ['D','E','F','G','A','B','C'],
        'A': ['A','B','C','D','E','F','G'],
        'E': ['E','F','G','A','B','C','D'],
        'B': ['B','C','D','E','F','G','A'],
        'F': ['F','G','A','B','C','D','E'],
        'Bb': ['B','C','D','E','F','G','A'],
        'Eb': ['E','F','G','A','B','C','D'],
        'Ab': ['A','B','C','D','E','F','G'],
        'Db': ['D','E','F','G','A','B','C'],
        'Gb': ['G','A','B','C','D','E','F'],
    }

    letters = key_roots.get(key, key_roots['C'])
    notes = []
    oct = start_oct
    for o in range(num_octaves):
        for i, letter in enumerate(letters):
            # Detect octave wrap (when letter index goes past B->C)
            if i > 0 and 'CDEFGAB'.index(letter) < 'CDEFGAB'.index(letters[i-1]):
                oct += 1
            notes.append(note_to_abc(letter, oct))
        oct += 1  # next octave
    # Add the top note
    notes.append(note_to_abc(letters[0], oct))
    return notes

def note_to_abc(letter, octave):
    """Convert note letter + octave to ABC notation."""
    if octave >= 5:
        return letter.lower() + "'" * (octave - 5)
    elif octave == 4:
        return letter
    else:
        return letter + ',' * (4 - octave)

def make_scale_asc(notes):
    """Ascending scale from note list."""
    return ' '.join(notes)

def make_scale_desc(notes):
    """Descending scale from note list."""
    return ' '.join(reversed(notes))

def make_scale_asc_desc(notes):
    """Ascending then descending."""
    return ' '.join(notes) + ' | ' + ' '.join(reversed(notes[:-1]))

# ── Exercise pattern generators ──

def ex_octaves(key, start_oct=3):
    """Scale in octaves - both hands play the same notes 1 octave apart."""
    notes = scale_abc(key, start_oct, 2)
    notes_low = scale_abc(key, start_oct - 1, 2)
    rh = make_scale_asc_desc(notes) + ' |]'
    lh = make_scale_asc_desc(notes_low) + ' |]'
    return rh, lh

def ex_sixths(key, start_oct=3):
    """Scale in sixths - hands play a sixth apart."""
    notes = scale_abc(key, start_oct, 2)
    # Sixth below = 5 diatonic steps down
    notes_6th = scale_abc(key, start_oct, 2)
    rh_notes = notes[5:] + [notes[-1]]  # start from 6th degree
    lh_notes = notes[:len(rh_notes)]
    rh = ' '.join(rh_notes) + ' |]'
    lh = ' '.join(lh_notes) + ' |]'
    return rh, lh

def ex_tenths(key, start_oct=3):
    """Scale in tenths - hands play a tenth apart."""
    notes_hi = scale_abc(key, start_oct + 1, 1)
    notes_lo = scale_abc(key, start_oct - 1, 1)
    rh = ' '.join(notes_hi) + ' | ' + ' '.join(reversed(notes_hi)) + ' |]'
    lh = ' '.join(notes_lo) + ' | ' + ' '.join(reversed(notes_lo)) + ' |]'
    return rh, lh

def ex_contrary(key, start_oct=4):
    """Contrary motion - hands move in opposite directions from unison."""
    notes_up = scale_abc(key, start_oct, 1)
    notes_down = list(reversed(scale_abc(key, start_oct - 1, 1)))
    max_len = min(len(notes_up), len(notes_down))
    rh = ' '.join(notes_up[:max_len]) + ' | ' + ' '.join(reversed(notes_up[:max_len])) + ' |]'
    lh = ' '.join(notes_down[:max_len]) + ' | ' + ' '.join(reversed(notes_down[:max_len])) + ' |]'
    return rh, lh

def ex_arpeggio(key, start_oct=3):
    """Broken chord arpeggios - root, 3rd, 5th, octave."""
    notes = scale_abc(key, start_oct, 2)
    # Pick chord tones: 0, 2, 4, 7 (root, 3rd, 5th, octave)
    if len(notes) >= 15:
        rh = f'{notes[7]} {notes[9]} {notes[11]} {notes[14]} | {notes[14]} {notes[11]} {notes[9]} {notes[7]} |]'
        lh = f'{notes[0]} {notes[2]} {notes[4]} {notes[7]} | {notes[7]} {notes[4]} {notes[2]} {notes[0]} |]'
    else:
        rh = f'{notes[0]} {notes[2]} {notes[4]} {notes[0]} |]'
        lh = rh
    return rh, lh

def ex_broken_chord(key, start_oct=3):
    """Broken chord patterns - alternating chord tones."""
    notes = scale_abc(key, start_oct, 2)
    if len(notes) >= 8:
        rh = f'{notes[7]}{notes[9]}{notes[7]}{notes[11]} {notes[9]}{notes[11]}{notes[9]}{notes[14]} | {notes[14]}{notes[11]}{notes[14]}{notes[9]} {notes[11]}{notes[9]}{notes[11]}{notes[7]} |]'
        lh = f'{notes[0]}{notes[2]}{notes[0]}{notes[4]} {notes[2]}{notes[4]}{notes[2]}{notes[7]} | {notes[7]}{notes[4]}{notes[7]}{notes[2]} {notes[4]}{notes[2]}{notes[4]}{notes[0]} |]'
    else:
        rh = ' '.join(notes) + ' |]'
        lh = rh
    return rh, lh

def ex_extended(key, start_oct=2):
    """Extended scale spanning 3+ octaves."""
    notes = scale_abc(key, start_oct, 3)
    half = len(notes) // 2
    rh = ' '.join(notes[half:]) + ' | ' + ' '.join(reversed(notes[half:])) + ' |]'
    lh = ' '.join(notes[:half+1]) + ' | ' + ' '.join(reversed(notes[:half+1])) + ' |]'
    return rh, lh

def ex_syncopation(key, start_oct=3):
    """Syncopated scale - offset rhythms between hands."""
    notes = scale_abc(key, start_oct, 2)
    notes_lo = scale_abc(key, start_oct - 1, 2)
    max_n = min(len(notes), len(notes_lo), 8)
    rh_parts = []
    lh_parts = []
    for i in range(max_n):
        rh_parts.append(f'z/{notes[i]}')
        lh_parts.append(f'{notes_lo[i]}z/')
    rh = ' '.join(rh_parts) + ' |]'
    lh = ' '.join(lh_parts) + ' |]'
    return rh, lh

# ── Build exercises ──

def abc_drill(xnum, title, meter, unit, rh, lh, key='C', tempo=72):
    abc = (
        f'X: {xnum}\nT: {title}\nM: {meter}\nL: {unit}\n'
        f'%%pagewidth 200cm\n%%continueall 1\n%%leftmargin 0.5cm\n%%rightmargin 0.5cm\n'
        f'%%topspace 0\n%%musicspace 0\n%%writefields Q 0\n'
        f'V: RH clef=treble name="RH"\nV: LH clef=bass name="LH"\n'
        f'K: {key}\n'
        f'[V: RH] [Q:1/4={tempo}] {rh}\n[V: LH] {lh}\n'
    )
    return {'n': str(xnum), 't': title, 'abc': abc}

exercises = []
xnum = 1400  # Thomas 280 number range

# Keys in order of the book (circle of fifths)
KEYS_MAJOR = ['C', 'G', 'D', 'A', 'E', 'B', 'F', 'Bb', 'Eb', 'Ab', 'Db', 'Gb']

# Pattern types for each key
PATTERNS = [
    ('Octaves', 'C', '1/4', ex_octaves),
    ('Sixths', 'C', '1/4', ex_sixths),
    ('Tenths', 'C', '1/4', ex_tenths),
    ('Extended', 'C', '1/4', ex_extended),
    ('Contrary Octave', 'C', '1/4', ex_contrary),
    ('Arpeggios', 'C', '1/4', ex_arpeggio),
    ('Broken Chords', 'C', '1/8', ex_broken_chord),
    ('Syncopation', 'C', '1/8', ex_syncopation),
]

# First 5 exercises: accent/technique in C
accent_exercises = [
    ("Thomas 1: Accent 3rd finger", "4/4", "1/8",
     "CDEF GABc DEFG ABcd | EFGa bcde fgab c'8 |]",
     "C,D,E,F, G,A,B,C DEFG | ABcd efga b8 |]"),
    ("Thomas 2: Accent 2nd finger", "4/4", "1/8",
     "CDEF GABc DEFG ABcd | EFGa bcde fgab c'8 |]",
     "C,D,E,F, G,A,B,C DEFG | ABcd efga b8 |]"),
    ("Thomas 3: Accent 1st finger", "4/4", "1/8",
     "CDEF GABc DEFG ABcd | EFGa bcde fgab c'8 |]",
     "C,D,E,F, G,A,B,C DEFG | ABcd efga b8 |]"),
    ("Thomas 4: Accent thumb", "4/4", "1/8",
     "CDEF GABc DEFG ABcd | EFGa bcde fgab c'8 |]",
     "C,D,E,F, G,A,B,C DEFG | ABcd efga b8 |]"),
    ("Thomas 5: Even touch", "C", "1/8",
     "CDEF GABc defg abc'd' | c'bag fedc BAGF EDCB, | C8 |]",
     "C,D,E,F, G,A,B,C DEFG | ABcd cBAG FEDC B,A,G,F, | C,8 |]"),
]

for title, meter, unit, rh, lh in accent_exercises:
    exercises.append(abc_drill(xnum, title, meter, unit, rh, lh))
    xnum += 1

# Generate pattern exercises for each key
for key in KEYS_MAJOR:
    for pat_name, _, unit, pat_func in PATTERNS:
        title = f"Thomas: {key} {pat_name}"
        try:
            rh, lh = pat_func(key)
            exercises.append(abc_drill(xnum, title, "C", unit, rh, lh, key=key))
        except Exception as e:
            print(f"  Skip {title}: {e}")
        xnum += 1

# Minor keys (harmonic minor)
KEYS_MINOR = ['Am', 'Em', 'Bm', 'Dm', 'Gm', 'Cm', 'Fm']

for key in KEYS_MINOR:
    base_key = key[:-1]  # 'Am' -> 'A'
    for pat_name, _, unit, pat_func in PATTERNS[:5]:  # fewer patterns for minor
        title = f"Thomas: {key} {pat_name}"
        try:
            rh, lh = pat_func(base_key)
            exercises.append(abc_drill(xnum, title, "C", unit, rh, lh, key=key))
        except Exception as e:
            print(f"  Skip {title}: {e}")
        xnum += 1

# Glissando exercises per key (scale + glissando pattern)
for key in KEYS_MAJOR:
    notes = scale_abc(key, 3, 2)
    if len(notes) >= 8:
        rh = ' '.join(notes[:8]) + ' | ' + ' '.join(notes[:8]) + ' | ' + notes[-1] + '4 |]'
        lh_notes = scale_abc(key, 2, 2)
        lh = ' '.join(lh_notes[:8]) + ' | ' + ' '.join(lh_notes[:8]) + ' | ' + lh_notes[-1] + '4 |]'
    else:
        rh = ' '.join(notes) + ' |]'
        lh = rh
    title = f"Thomas: {key} Glissando"
    exercises.append(abc_drill(xnum, title, "C", "1/8", rh, lh, key=key))
    xnum += 1

# Save
outpath = PROJECT_DIR / 'abc2stripchart/thomas_280.json'
with open(outpath, 'w') as f:
    json.dump(exercises, f, separators=(',', ':'))

print(f"Generated {len(exercises)} Thomas exercises (range 1400-{xnum-1})")
print(f"Saved to {outpath}")
