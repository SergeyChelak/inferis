#[derive(Clone, Copy, Debug)]
pub struct FrameDuration {
    duration: usize,
    progress: usize,
}

impl FrameDuration {
    pub fn new(duration: usize) -> Self {
        Self {
            duration,
            progress: 0,
        }
    }

    pub fn infinite() -> Self {
        Self::new(usize::MAX)
    }

    pub fn teak(&mut self) -> bool {
        if self.is_completed() {
            return false;
        }
        self.progress += 1;
        true
    }

    pub fn is_completed(&self) -> bool {
        self.duration == self.progress
    }

    pub fn is_performing(&self) -> bool {
        self.progress > 0
    }
}
