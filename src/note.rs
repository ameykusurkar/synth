use crate::envelope::Envelope;

pub struct Note {
    start: f32,
    freq: f32,
    released_at: Option<f32>,
}

impl Note {
    pub fn new(freq: f32, t: f32) -> Self {
        Self {
            start: t,
            freq,
            released_at: None,
        }
    }

    pub fn release(&mut self, t: f32) {
        self.released_at = Some(t);
    }

    pub fn sample(&mut self, t: f32, envelope: &Envelope) -> f32 {
        let amp = envelope.amplitude(self.note_state(t));
        amp * sawtooth(self.freq, t)
    }

    fn note_state(&self, t: f32) -> NoteState {
        if let Some(released_at) = self.released_at {
            NoteState::Released(t - released_at)
        } else {
            NoteState::Held(t - self.start)
        }
    }
}

#[derive(Copy, Clone)]
pub enum NoteState {
    Held(f32),
    Released(f32),
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
