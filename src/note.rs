use crate::envelope::Envelope;

pub struct Note {
    freq: f32,
    envelope: Envelope,
}

impl Note {
    pub fn new(freq: f32, t: f32, duration: f32, attack_duration: f32) -> Self {
        Self {
            freq,
            envelope: Envelope::new(t, duration, attack_duration),
        }
    }

    pub fn release(&mut self) {
        self.envelope.release();
    }

    pub fn sample(&mut self, t: f32) -> Option<f32> {
        self.envelope.amplitude(t).map(|amp| amp * sawtooth(self.freq, t))
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
