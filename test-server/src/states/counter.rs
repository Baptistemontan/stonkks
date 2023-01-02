use std::{ops::Deref, sync::atomic::AtomicUsize};

#[derive(Default)]
pub struct CounterState(pub AtomicUsize);

impl Deref for CounterState {
    type Target = AtomicUsize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
