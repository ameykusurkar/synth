pub struct Envelope {
    start: f32,
    attack_end: f32,
    attack_peak: f32,

    note_end: f32,
    released: bool,
}

impl Envelope {
    pub fn new(t: f32, duration: f32, attack_duration: f32) -> Self {
        Self {
            start: t,
            attack_end: t + attack_duration,
            attack_peak: 1.0,
            note_end: t + duration,
            released: false,
        }
    }

    pub fn amplitude(&self, t: f32) -> Option<f32> {
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

    pub fn release(&mut self) {
        self.released = true;
    }
}

fn gradient(x_min: f32, y_min: f32, x_max: f32, y_max: f32) -> f32 {
    (y_max - y_min) / (x_max - x_min)
}
