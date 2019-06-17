
pub struct Clock {
    pub time: u64,
}

impl Clock {
    pub fn now(&self) -> u64 {
        self.time
    }
    pub fn step(&mut self, step: u64) {
        self.time += step;
    }
}