use crate::note::NoteState;

pub struct Envelope {
    pub attack_duration: f32,
    pub attack_amplitude: f32,

    pub decay_duration: f32,

    pub sustain_amplitude: f32,

    pub release_duration: f32,
}

impl Envelope {
    pub fn amplitude(&self, state: NoteState) -> f32 {
        match state {
            NoteState::Held(elapsed) => {
                if elapsed < self.attack_duration {
                    self.attack_amplitude(elapsed)
                } else if elapsed < self.attack_duration + self.decay_duration {
                    self.decay_amplitude(elapsed - self.attack_duration)
                } else {
                    self.sustain_amplitude
                }
            }
            NoteState::Released(elapsed) => {
                if elapsed < self.release_duration {
                    self.release_amplitude(elapsed)
                } else {
                    0.0
                }
            }
        }
    }

    fn attack_amplitude(&self, t: f32) -> f32 {
        (self.attack_amplitude / self.attack_duration) * t
    }

    fn decay_amplitude(&self, t: f32) -> f32 {
        let grad = gradient(
            0.0,
            self.attack_amplitude,
            self.decay_duration,
            self.sustain_amplitude,
        );
        grad * t + self.attack_amplitude
    }

    fn release_amplitude(&self, t: f32) -> f32 {
        let grad = gradient(0.0, self.sustain_amplitude, self.release_duration, 0.0);
        grad * t + self.sustain_amplitude
    }
}

fn gradient(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    (y2 - y1) / (x2 - x1)
}
