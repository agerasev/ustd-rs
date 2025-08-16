extern crate std;

use crate::error::Error;
use core::{marker::PhantomData, time::Duration};
use std::{
    cell::RefCell,
    sync::{Arc, Condvar, Mutex, Weak},
    thread::{self, Thread, ThreadId},
    thread_local,
    time::Instant,
};

/// Basic execution context.
pub trait Context {}

/// Context that allows to make blocking calls.
pub trait BlockingContext: Context {
    fn sleep(&mut self, duration: Option<Duration>);
}

/// Unique task identifier.
pub type TaskId = ThreadId;

/// Task priority.
pub type Priority = usize;

#[derive(Default)]
struct State {
    condvar: Condvar,
    finished: Mutex<bool>,
}

impl State {
    fn finish(&self) {
        let mut guard = self.finished.lock().unwrap();
        assert!(!*guard);
        *guard = true;
        self.condvar.notify_all();
    }
    fn wait_finished(&self, timeout: Option<Duration>) -> bool {
        let mut guard_slot = Some(self.finished.lock().unwrap());
        let instant = Instant::now();
        loop {
            let guard = guard_slot.take().unwrap();
            if *guard {
                break true;
            }
            guard_slot.replace(match timeout {
                Some(total) => {
                    let current = instant.elapsed();
                    if current >= total {
                        break false;
                    }
                    self.condvar.wait_timeout(guard, total - current).unwrap().0
                }
                None => self.condvar.wait(guard).unwrap(),
            });
        }
    }
}

/// Unit of execution.
///
/// Internally the same as [`std::thread::Thread`].
#[derive(Clone)]
pub struct Task {
    thread: Thread,
}

impl From<Thread> for Task {
    fn from(thread: Thread) -> Self {
        Self { thread }
    }
}

/// Task handle.
pub struct Handle {
    task: Task,
    state: Weak<State>,
}

/// Context inside task.
pub struct TaskContext {
    task: Task,
    state: Arc<State>,
    /// To ensure `!Sync + !Send`
    _p: PhantomData<*const ()>,
}

impl Task {
    /// Task unique identifier.
    pub fn id(&self) -> TaskId {
        self.thread.id()
    }
    pub fn thread(&self) -> Thread {
        self.thread.clone()
    }
}

impl Handle {
    pub fn task(&self) -> Task {
        self.task.clone()
    }
    /// Wait for task to finish.
    pub fn join<C: BlockingContext>(&self, _cx: &mut C, timeout: Option<Duration>) -> bool {
        if let Some(state) = self.state.upgrade() {
            state.wait_finished(timeout)
        } else {
            true
        }
    }
}

thread_local! {
    static STATE: RefCell<Weak<State>> = const { RefCell::new(Weak::new()) };
}

fn init_current_state(state: Arc<State>) {
    STATE.with_borrow_mut(move |weak| {
        assert_eq!(weak.strong_count(), 0);
        *weak = Arc::downgrade(&state);
    });
}

impl TaskContext {
    fn new(task: Task, state: Arc<State>) -> Self {
        Self {
            task,
            state,
            _p: PhantomData,
        }
    }
    /// Create a new context for current task.
    ///
    /// Panics if context for the task already exists.
    pub fn enter() -> Self {
        let state = Arc::new(State::default());
        init_current_state(state.clone());
        Self::new(thread::current().into(), state)
    }
    /// Get already created context for current task.
    ///
    /// Panics if context hasn't created or already dropped.
    pub fn current() -> Self {
        let state = STATE.with_borrow(|weak| weak.upgrade().expect("Task has no active context"));
        Self::new(thread::current().into(), state)
    }

    pub fn task(&self) -> Task {
        self.task.clone()
    }
}

impl Context for TaskContext {}

impl BlockingContext for TaskContext {
    /// Sleep for specified `duration`.
    ///
    /// If `None` then sleep infinetely.
    fn sleep(&mut self, duration: Option<Duration>) {
        match duration {
            Some(t) => thread::sleep(t),
            None => loop {
                thread::park();
            },
        }
    }
}

/// Context inside interrupt.
#[derive(Default)]
pub struct InterruptContext {
    /// To ensure `!Sync + !Send`
    _p: PhantomData<*const ()>,
}

impl InterruptContext {
    /// # Safety
    /// It is always safe to construct [`InterruptContext`] using `std` backend.
    /// The method marked as `unsafe` for compatibility with `freertos` backend.
    pub unsafe fn new() -> Self {
        Self { _p: PhantomData }
    }
    pub fn should_yield(&self) -> bool {
        false
    }
}

impl Context for InterruptContext {}

pub struct Builder {
    inner: thread::Builder,
}

impl Builder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            inner: thread::Builder::new(),
        }
    }

    fn map<F: FnOnce(thread::Builder) -> thread::Builder>(self, f: F) -> Self {
        Self {
            inner: f(self.inner),
        }
    }

    pub fn name(self, name: &str) -> Self {
        self.map(|b| b.name(name.into()))
    }
    pub fn stack_size(self, size: usize) -> Self {
        self.map(|b| b.stack_size(size))
    }
    pub fn priority(self, _: Priority) -> Self {
        // nothing to do
        self
    }
    pub fn spawn<F: FnOnce(&mut TaskContext) + Send + 'static>(
        self,
        func: F,
    ) -> Result<Handle, Error> {
        let state = Arc::new(State::default());
        let thread = {
            let state = state.clone();
            self.inner
                .spawn(move || {
                    init_current_state(state.clone());
                    let mut cx = TaskContext::new(thread::current().into(), state);
                    func(&mut cx);
                    cx.state.finish();
                })?
                .thread()
                .clone()
        };
        Ok(Handle {
            task: Task { thread },
            state: Arc::downgrade(&state),
        })
    }
}

/// Spawn a new task.
pub fn spawn<F: FnOnce(&mut TaskContext) + Send + 'static>(func: F) -> Result<Handle, Error> {
    Builder::new().spawn(func)
}
