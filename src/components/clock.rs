use std::time::SystemTime;

use super::Clockable;

#[derive(Debug)]
pub struct Clock {
    mhz: usize,
    step_chunk: usize,
    prev_cycle_time: SystemTime,
    nano_per_cycle: u128,
    last_cycle_count: u128,
    cpu: Box<dyn Clockable>,
}

impl Clock {
    pub fn init(cpu: Box<dyn Clockable>, mhz: usize, step_chunk: usize) -> Clock {
        Clock {
            cpu: cpu,
            mhz: mhz,
            step_chunk: step_chunk,
            last_cycle_count: 1,
            nano_per_cycle: 1000 / mhz as u128,
            prev_cycle_time: SystemTime::now(),
        }
    }

    pub fn cycle(&mut self) -> () {
        for _a in 0..self.step_chunk {
            let nano_delta = SystemTime::now()
                .duration_since(self.prev_cycle_time)
                .unwrap()
                .as_nanos();
            if nano_delta > self.nano_per_cycle * self.last_cycle_count {
                self.prev_cycle_time = SystemTime::now();
                self.last_cycle_count = self.cpu.step() as u128;
            }
        }
    }
}
