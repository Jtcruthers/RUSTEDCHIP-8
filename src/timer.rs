pub struct Timer {
    value: u8,
}

impl Timer {
    pub fn new(hz: u8) -> Self {
        Self {
            value: 0,
        }
    }

    pub fn set(&mut self, time: u8) {
        self.value = time;
    }

    pub fn get(&self) -> u8 {
        self.value
    }
}

