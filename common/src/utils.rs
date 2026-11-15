pub type CounterType = u64;

pub struct Counter {
    i: CounterType,
    max: CounterType,
}

impl Counter {
    pub fn new(max: CounterType) -> Self {
        Self {
            i: 0,
            max,
        }
    }

    pub fn step(&mut self, count: CounterType) -> CounterType {
        self.i = (self.i + count) % self.max;
        self.i
    }
}

use std::time::{ Duration, SystemTime, UNIX_EPOCH };

pub fn epoch_time() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
}

pub fn epoch_time_millis() -> u128 {
    epoch_time().as_millis()
}

#[macro_export]
macro_rules! sleep_until {
    ($mode:ident, $timestamp:expr) => {{
        let timestamp = $timestamp;
        let duration = Duration::from_micros(500);
        // TODO use a more sophisticated method for precise sleeping,
        // such as decreasing the interval continuously
        while common::utils::epoch_time_millis() < timestamp {
            common::sleep_until!($mode, duration, internal_sleep);
        }
    }};
    (tokio, $duration:expr, internal_sleep) => {{
        tokio::time::sleep($duration).await;
    }};
    (thread, $duration:expr, internal_sleep) => {{
        std::thread::sleep($duration);
    }};
}

#[macro_export]
macro_rules! measure_time {
    ($tag:literal, $block:block) => {{
        let start = std::time::Instant::now();
        $block;
        let time = std::time::Instant::now().duration_since(start);
        let millis = time.as_millis();
        println!(
            "exec time for {}: {}ms, {}μ",
            $tag,
            millis,
            time.as_micros() - 1000 * millis,
        );
    }}
}
