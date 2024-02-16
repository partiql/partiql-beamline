use partiql_value::Value;
use std::cmp::Ordering;

/// A simulation 'tick'; equivalent to 1 ms
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Tick(pub u128);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Sample {
    pub tick: Tick,
    pub value: Value,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProcessId(pub usize);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Event {
    pub pid: ProcessId,
    pub sample: Sample,
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        // Flip ordering to implemnt min-heap
        other
            .sample
            .tick
            .cmp(&self.sample.tick)
            .then_with(|| other.pid.cmp(&self.pid))
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
