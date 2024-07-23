use std::time::{Duration, Instant};

use crate::DoTick;

#[derive(Clone, Copy)]
pub struct Timer {
    freq: u128,
    exec: DoTick,
    last: Instant,
}

impl Timer {
    pub fn new(hz: u32, exec: DoTick) -> Timer {
        Timer {
            freq: 1_000_000 / hz as u128,
            exec,
            last: Instant::now(),
        }
    }

    fn try_tick(&mut self) -> bool {
        if self.last.elapsed().as_micros() >= self.freq {
            self.last = Instant::now();
            true
        } else {
            false
        }
    }
}

pub fn check_timers(timers: &mut [Timer]) -> Vec<DoTick> {
    let mut res = vec![];
    for timer in timers {
        if timer.try_tick() {
            res.push(timer.exec)
        }
    }
    res
}
