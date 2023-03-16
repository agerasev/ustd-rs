extern crate std;

use crate::{error::Error, task::Priority};
use core::time::Duration;
use std::{
    cell::RefCell,
    sync::{Arc, Condvar, Mutex},
    thread::{self, Thread, ThreadId},
    thread_local,
    time::Instant,
};

pub(crate) type TaskId = ThreadId;

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
    fn wait_finish(&self, timeout: Option<Duration>) -> bool {
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

#[derive(Clone)]
pub(crate) struct Task {
    thread: Thread,
}

pub(crate) struct Handle {
    task: Task,
    state: Arc<State>,
}

pub(crate) struct Local {
    state: Arc<State>,
    interrupt: bool,
}

thread_local! {
    static LOCAL: RefCell<Option<Local>> = RefCell::new(None);
}

pub(crate) fn is_interrupt() -> bool {
    LOCAL.with(|this| this.borrow().as_ref().unwrap().interrupt)
}

impl Task {
    pub fn id(&self) -> TaskId {
        self.thread.id()
    }
}

impl Handle {
    pub fn task(&self) -> Task {
        self.task.clone()
    }
    pub fn join(&self, timeout: Option<Duration>) -> bool {
        self.state.wait_finish(timeout)
    }
}

pub(crate) struct Builder {
    inner: thread::Builder,
}

impl Builder {
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
    pub fn spawn<F: FnOnce() + Send + 'static>(self, func: F) -> Result<Handle, Error> {
        let state = Arc::new(State::default());
        let thread = {
            let state = state.clone();
            self.inner
                .spawn(move || {
                    let enter = Enter::new(state.clone());
                    func();
                    drop(enter);
                })?
                .thread()
                .clone()
        };
        Ok(Handle {
            task: Task { thread },
            state,
        })
    }
}

pub struct Enter {
    _unused: [u8; 0],
}

impl Enter {
    fn new(state: Arc<State>) -> Self {
        LOCAL.with(|this| {
            this.borrow_mut().replace(Local {
                interrupt: false,
                state,
            })
        });
        Self { _unused: [] }
    }
}

impl Drop for Enter {
    fn drop(&mut self) {
        LOCAL.with(|this| {
            this.borrow_mut().as_ref().unwrap().state.finish();
        });
    }
}

/// Make the current thread a task.
pub fn enter() -> Enter {
    Enter::new(Arc::new(State::default()))
}

pub(crate) fn current() -> Task {
    Task {
        thread: thread::current(),
    }
}

pub(crate) fn sleep(duration: Option<Duration>) {
    match duration {
        Some(t) => thread::sleep(t),
        None => loop {
            thread::park();
        },
    }
}

pub(crate) struct Interrupt {
    _unused: [u8; 0],
}

impl Interrupt {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        LOCAL.with(|this| {
            let mut ref_ = this.borrow_mut();
            let local = ref_.as_mut().unwrap();
            assert!(!local.interrupt);
            local.interrupt = true;
        });
        Self { _unused: [] }
    }
}

impl Drop for Interrupt {
    fn drop(&mut self) {
        LOCAL.with(|this| {
            let mut ref_ = this.borrow_mut();
            let local = ref_.as_mut().unwrap();
            assert!(local.interrupt);
            local.interrupt = false;
        });
    }
}
