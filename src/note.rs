pub struct Note {
    freq: f32,
    envelope: Envelope,
}

impl Note {
    pub fn new(freq: f32, play_until: f32) -> Self {
        Self {
            freq,
            envelope: Envelope::new(play_until),
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
    play_until: f32,
    released: bool,
}

impl Envelope {
    fn new(play_until: f32) -> Self {
        Self {
            play_until,
            released: false,
        }
    }

    fn amplitude(&self, t: f32) -> Option<f32> {
        if !self.released && t < self.play_until {
            Some(1.0)
        } else {
            None
        }
    }

    fn release(&mut self) {
        self.released = true;
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
