
pub struct Clock {
    pub time: i64,
}

impl Clock {
    pub fn now(&self) -> i64 {
        self.time
    }
    pub fn step(&mut self, step: i64) {
        self.time += step;
    }
}