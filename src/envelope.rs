pub struct Envelope {
    attack_duration: f32,
    attack_peak: f32,
    duration: f32,
}

impl Envelope {
    pub fn new(duration: f32, attack_duration: f32) -> Self {
        Self {
            attack_duration,
            duration,
            attack_peak: 1.0,
        }
    }

    pub fn amplitude(&self, elapsed: f32, released: bool) -> Option<f32> {
        if released { return None };

        if elapsed < self.attack_duration {
            let grad = self.attack_peak / self.attack_duration;
            Some(grad * elapsed)
        } else if elapsed < self.duration {
            Some(self.attack_peak)
        } else {
            None
        }
    }
}
