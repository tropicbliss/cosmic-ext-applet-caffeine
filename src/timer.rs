use std::time::{Duration, Instant};

pub struct Timer {
    ending_time: Option<Instant>,
    just_completed: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    pub fn new() -> Self {
        Self {
            ending_time: None,
            just_completed: false,
        }
    }

    pub fn start(&mut self, duration: Duration) {
        self.ending_time = Some(Instant::now() + duration);
    }

    pub fn get_formatted_time(&self) -> Option<String> {
        self.ending_time.as_ref().map(|ending_time| {
            let seconds = (*ending_time - Instant::now()).as_secs();
            let hours = seconds / 3600;
            let minutes = (seconds % 3600) / 60;
            let secs = seconds % 60;
            if hours > 0 {
                format!("{:02}:{:02}:{:02}", hours, minutes, secs)
            } else {
                format!("{:02}:{:02}", minutes, secs)
            }
        })
    }

    pub fn is_started(&self) -> bool {
        self.ending_time.is_some()
    }

    pub fn timer_just_ended(&mut self) -> bool {
        let result = self.just_completed;
        if self.just_completed {
            self.just_completed = false;
        }
        result
    }

    pub fn cancel(&mut self) {
        self.ending_time = None;
    }

    pub fn tick(&mut self) {
        if let Some(ending_time) = &self.ending_time {
            if Instant::now() > *ending_time {
                self.ending_time = None;
                self.just_completed = true;
            }
        }
    }
}
