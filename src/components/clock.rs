#[cfg(test)]
use fake_clock::FakeClock as Instant;
#[cfg(not(test))]
use std::time::Instant;

use super::Clockable;

#[derive(Debug)]
pub struct Clock {
    mhz: usize,
    step_chunk: usize,
    prev_cycle_time: Instant,
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
            prev_cycle_time: Instant::now(),
        }
    }

    pub fn cycle(&mut self) -> () {
        for _a in 0..self.step_chunk {
            let nano_delta = self.prev_cycle_time.elapsed().as_nanos();
            if nano_delta > self.nano_per_cycle * self.last_cycle_count {
                self.prev_cycle_time = Instant::now();
                self.last_cycle_count = self.cpu.step() as u128;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_sleep(time: u64) {
        use fake_clock::FakeClock;
        FakeClock::advance_time(time);
    }

    #[test]
    fn initial_state() {
        #[derive(Debug)]
        struct FakeClockable {
            cycles: usize,
        };

        impl Clockable for FakeClockable {
            fn get_cycles(&self) -> usize {
                return self.cycles;
            }

            fn step(&mut self) -> usize {
                self.cycles = self.cycles.wrapping_add(2);
                return 5;
            }
        }

        let fake_clockable = FakeClockable { cycles: 0 };

        let mut clock = Clock::init(Box::new(fake_clockable), 1, 1);
        assert_eq!(0, clock.cpu.get_cycles(), "get_cycles");
        assert_eq!(1, clock.last_cycle_count, "last_cycle_count");
        assert_eq!(
            0,
            clock.prev_cycle_time.elapsed().as_nanos(),
            "prev_cycle_time"
        );

        fake_sleep(1);

        assert_eq!(
            1000000,
            clock.prev_cycle_time.elapsed().as_nanos(),
            "prev_cycle_time"
        );

        clock.cycle();

        assert_eq!(2, clock.cpu.get_cycles(), "get_cycles");
        assert_eq!(5, clock.last_cycle_count, "last_cycle_count");
        assert_eq!(
            0,
            clock.prev_cycle_time.elapsed().as_nanos(),
            "prev_cycle_time"
        );
    }

}
