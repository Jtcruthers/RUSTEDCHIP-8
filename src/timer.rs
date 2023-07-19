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
        let amount_to_decrement = elapsed_ms_since_last_decrement / self.ms_per_cycle;
        if amount_to_decrement > self.value as u128 {
            self.value = 0;
            self.last_decremented = SystemTime::now();
        } else if amount_to_decrement > 0 {
            self.value = self.value - amount_to_decrement as u8;
            self.last_decremented = SystemTime::now();
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

