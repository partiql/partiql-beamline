use partiql_value::Value;
use std::cmp::Ordering;

/// A simulation 'tick'; equivalent to 1 ms.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Tick(pub u128);

/// A single sample of a random process.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Sample {
    pub tick: Tick,
    pub value: Value,
}

/// A random process's id within the simulation.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProcessId(pub usize);

/// An event; a [`Sample`] for a [`Process`] (as specified by its [`ProcessId`]).
///
/// Events are ordered by the [`Tick`] of the [`Sample`] and ties are broken based on
/// the [`Process`]'s [`ProcessId`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Event {
    pub pid: ProcessId,
    pub sample: Sample,
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sample
            .tick
            .cmp(&other.sample.tick)
            .then_with(|| self.pid.cmp(&other.pid))
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
