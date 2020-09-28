use crate::envelope::Envelope;

pub struct Note {
    note_start: f32,
    freq: f32,
    envelope: Envelope,
    released: bool,
}

impl Note {
    pub fn new(freq: f32, t: f32, duration: f32, attack_duration: f32) -> Self {
        Self {
            note_start: t,
            freq,
            envelope: Envelope::new(duration, attack_duration),
            released: false,
        }
    }

    pub fn release(&mut self) {
        self.released = true;
    }

    pub fn sample(&mut self, t: f32) -> Option<f32> {
        self.envelope
            .amplitude(t - self.note_start, self.released)
            .map(|amp| amp * sawtooth(self.freq, t))
    }
}

fn sin(freq: f32, t: f32) -> f32 {
    (2.0 * 3.14159 * freq * t).sin()
}

fn square(freq: f32, t: f32) -> f32 {
    if (2.0 * 3.14159 * freq * t).sin() > 0.0 {
        1.0
    } else {
        -1.0
    }
}

fn sawtooth(freq: f32, t: f32) -> f32 {
    let period = 1.0 / freq;
    2.0 * (t / period - (0.5 + t / period).floor())
}
