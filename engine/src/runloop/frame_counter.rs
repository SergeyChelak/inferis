use std::collections::HashMap;

#[derive(Default)]
pub struct AggregatedFrameCounter {
    map: HashMap<String, FrameCounter>,
}

impl AggregatedFrameCounter {
    pub fn add_counter(&mut self, counter_id: impl Into<String>, duration: usize) {
        let item = FrameCounter::new(duration);
        self.map.insert(counter_id.into(), item);
    }

    pub fn teak(&mut self) {
        self.map.iter_mut().for_each(|(_, val)| {
            val.teak();
        })
    }

    pub fn state(&self, counter_id: &str) -> Option<FrameCounterState> {
        let counter = self.map.get(counter_id)?;
        use FrameCounterState::*;
        if counter.is_completed() {
            return Some(Completed);
        }
        if counter.is_performing() {
            return Some(InProgress(counter.progress));
        }
        Some(Ready)
    }

    pub fn is_completed(&self, counter_id: &str) -> bool {
        self.state(counter_id)
            .map(|x| matches!(x, FrameCounterState::Completed))
            .unwrap_or(false)
    }

    pub fn remove(&mut self, counter_id: &str) -> bool {
        self.map.remove(counter_id).is_some()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum FrameCounterState {
    Ready,
    InProgress(usize),
    Completed,
}

#[derive(Clone, Copy, Debug)]
pub struct FrameCounter {
    duration: usize,
    progress: usize,
}

impl FrameCounter {
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
