use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::default::Default;

use crate::primitives::{Event, Tick};

#[derive(Debug, Eq, PartialEq)]
struct TimelineEvent(Event);

// [`BinaryHeap`] depends on `Ord` and implements a max-heap.
// Here, we reverse the 'natural' ordering of [`Event`] so the queue becomes a min-heap.
impl Ord for TimelineEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Flip ordering to implement min-heap
        other.0.cmp(&self.0)
    }
}

impl PartialOrd for TimelineEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<Event> for TimelineEvent {
    fn from(evt: Event) -> Self {
        Self(evt)
    }
}

impl From<TimelineEvent> for Event {
    fn from(tevt: TimelineEvent) -> Self {
        tevt.0
    }
}

impl<'a> From<&'a TimelineEvent> for &'a Event {
    fn from(tevt: &'a TimelineEvent) -> Self {
        &tevt.0
    }
}

#[derive(Default)]
pub struct Timeline {
    queue: BinaryHeap<TimelineEvent>,
}

impl Timeline {
    pub fn next(&self) -> Option<&Tick> {
        self.peek().map(|e| &e.tick)
    }
    pub fn peek(&self) -> Option<&Event> {
        self.queue.peek().map(Into::into)
    }

    pub fn pop(&mut self) -> Option<Event> {
        self.queue.pop().map(Into::into)
    }

    pub fn push(&mut self, evt: Event) {
        self.queue.push(evt.into());
    }
}
