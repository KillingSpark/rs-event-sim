pub struct Clock {
    time: u64,
}

pub fn new() -> Clock {
    Clock { time: 0 }
}

static YEARS: u64 = 365*DAYS;
static WEEKS: u64 = 7*DAYS;
static DAYS: u64 = 24*HOURS;
static HOURS: u64 = 60 * MINUTES;
static MINUTES: u64 = 60*SECONDS;
static SECONDS: u64 = 1000*MILLI_SECONDS;
static MILLI_SECONDS: u64 = 1000*MICRO_SECONDS;
static MICRO_SECONDS: u64 = 1000;

impl Clock {
    pub fn set(&mut self, time: u64) -> Result<(), Box<std::error::Error>> {
        if self.time > time {
            panic!("Dont revert time Morty. Thats bad *burp* manners.")
        }
        self.time = time;
        Ok(())
    }

    pub fn now(&self) -> u64 {
        self.time
    }

    pub fn micros(&self) -> u64 {
        self.time / MICRO_SECONDS
    }
    pub fn millis(&self) -> u64 {
        self.time / MILLI_SECONDS
    }
    pub fn secs(&self) -> u64 {
        self.time / SECONDS
    }
    pub fn mins(&self) -> u64 {
        self.time / MINUTES
    }
    pub fn hours(&self) -> u64 {
        self.time / HOURS
    }
    pub fn days(&self) -> u64 {
        self.time / DAYS
    }
    pub fn weeks(&self) -> u64 {
        self.time / WEEKS
    }
    pub fn years(&self) -> u64 {
        self.time / YEARS
    }
}
