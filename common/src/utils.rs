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
