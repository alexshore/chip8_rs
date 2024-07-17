use std::time::Instant;

use crate::Execute;

#[derive(Clone, Copy)]
pub struct Timer {
    freq: u32,
    exec: Execute,
    last: Instant,
}

impl Timer {
    pub fn new(freq: u32, exec: Execute) -> Timer {
        Timer {
            freq,
            exec,
            last: Instant::now(),
        }
    }

    fn try_tick(&mut self) -> bool {
        true
    }
}

/* TODO
   1. take in list of timers
   2. select next timer in list
   3. if current time is greater than timers last tick time + (1 / freq) then add relevant Execute to `res` vec (create method on Timer struct)
   4. repeat steps 2-3 until end of list
   5. return list
*/
pub fn check_timers(timers: &mut [Timer]) -> Vec<Execute> {
    let mut res = vec![];
    for timer in timers {
        if timer.try_tick() {
            res.push(timer.exec)
        }
    }
    res
}
