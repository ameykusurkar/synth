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

struct Envelope {
    start: f32,
    attack_end: f32,
    attack_peak: f32,

    note_end: f32,
    released: bool,
}

impl Envelope {
    fn new(t: f32, duration: f32, attack_duration: f32) -> Self {
        Self {
            start: t,
            attack_end: t + attack_duration,
            attack_peak: 1.0,
            note_end: t + duration,
            released: false,
        }
    }

    fn amplitude(&self, t: f32) -> Option<f32> {
        if self.released { return None };

        if t < self.attack_end {
            let grad = gradient(self.start, 0.0, self.attack_end, self.attack_peak);
            Some(grad * (t - self.start))
        } else if t < self.note_end {
            Some(self.attack_peak)
        } else {
            None
        }
    }

    fn release(&mut self) {
        self.released = true;
    }
}

fn gradient(x_min: f32, y_min: f32, x_max: f32, y_max: f32) -> f32 {
    (y_max - y_min) / (x_max - x_min)
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
