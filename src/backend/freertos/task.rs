use crate::Error;
use core::{
    ops::{Add, AddAssign},
    time::Duration,
};

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct TaskId(freertos::FreeRtosBaseType);

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Priority(u8);

impl Priority {
    fn idle() -> Self {
        Priority(0)
    }
    fn to_freertos(self) -> freertos::TaskPriority {
        freertos::TaskPriority(self.0)
    }
}

#[derive(Clone, Debug)]
pub struct Task {
    task: freertos::Task,
}

impl Task {
    pub fn id(&self) -> TaskId {
        TaskId(self.task.get_id().unwrap())
    }
    pub fn priority(&self) -> Priority {
        unimplemented!()
    }
}

pub fn spawn<F: FnOnce() + Send + 'static>(priority: Priority, func: F) -> Result<Task, Error> {
    freertos::Task::new()
        .priority(priority.to_freertos())
        .start(|_| func())
        .map(|task| Task { task })
}

pub fn current() -> Result<Task, Error> {
    freertos::Task::current().map(|task| Task { task })
}

pub fn sleep(dur: Duration) {
    freertos::CurrentTask::delay(freertos::Duration::ms(dur.as_millis() as u32))
}

impl Default for Priority {
    fn default() -> Self {
        Self::idle() + 1
    }
}

impl Add<usize> for Priority {
    type Output = Self;
    fn add(mut self, rhs: usize) -> Self::Output {
        self.0 += rhs as u8;
        self
    }
}

impl AddAssign<usize> for Priority {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs as u8;
    }
}
