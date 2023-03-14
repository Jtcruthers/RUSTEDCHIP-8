use std::time::SystemTime;

pub struct Timer {
    value: u8,
    last_decremented: SystemTime,
    ms_per_cycle: u128
}

impl Timer {
    pub fn new(hz: u8) -> Self {
        Self {
            value: 0,
            last_decremented: SystemTime::now(),
            ms_per_cycle: 1000 / hz as u128
        }
    }

    pub fn check_decrement(&mut self) {
        let elapsed_ms_since_last_decrement = self.last_decremented
                                                .elapsed()
                                                .expect("Couldn't get elapsed time")
                                                .as_millis();
        if elapsed_ms_since_last_decrement > self.ms_per_cycle {
            self.last_decremented = SystemTime::now();
            self.value = if self.value > 0 { self.value - 1} else { 0 };
        }
    }

    pub fn set(&mut self, time: u8) {
        self.value = time;
        self.last_decremented = SystemTime::now();
    }

    pub fn get(&self) -> u8 {
        self.value
    }
}

