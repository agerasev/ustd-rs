extern crate std;

use crate::{error::Error, task::Priority};
use core::time::Duration;
use std::{
    cell::RefCell,
    thread::{self, Thread, ThreadId},
    thread_local,
};

pub(crate) type TaskId = ThreadId;

#[derive(Clone, Copy, Default, Debug)]
pub(crate) struct Info {
    priority: Priority,
}

#[derive(Clone, Debug)]
pub(crate) struct Task {
    thread: Thread,
    info: Info,
}

pub(crate) struct Local {
    info: Info,
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

pub(crate) struct Builder {
    inner: thread::Builder,
    info: Info,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            inner: thread::Builder::new(),
            info: Info::default(),
        }
    }

    fn map<F: FnOnce(thread::Builder) -> thread::Builder>(self, f: F) -> Self {
        Self {
            inner: f(self.inner),
            info: self.info,
        }
    }

    pub fn name(self, name: &str) -> Self {
        self.map(|b| b.name(name.into()))
    }
    pub fn stack_size(self, size: usize) -> Self {
        self.map(|b| b.stack_size(size))
    }
    pub fn priority(mut self, priority: Priority) -> Self {
        self.info.priority = priority;
        self
    }
    pub fn spawn<F: FnOnce() + Send + 'static>(self, func: F) -> Result<Task, Error> {
        Ok(Task {
            thread: self
                .inner
                .spawn(move || {
                    LOCAL.with(|this| {
                        this.borrow_mut().replace(Local {
                            info: self.info,
                            interrupt: false,
                        })
                    });
                    func();
                })?
                .thread()
                .clone(),
            info: self.info,
        })
    }
}

pub(crate) fn current() -> Task {
    Task {
        thread: thread::current(),
        info: LOCAL.with(|this| {
            let ref_ = this.borrow();
            let local = ref_.as_ref().unwrap();
            assert!(!local.interrupt);
            local.info
        }),
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
