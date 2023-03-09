#[derive(Debug)]
pub struct Chip {
    pub memory: [u8; 4096]
}

impl Chip {
    pub fn new() -> Self {
        Chip {
            memory: [0; 4096]
        }
    }
}
