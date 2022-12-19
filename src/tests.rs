use crate::{
    sync::Semaphore,
    task::{self, Priority},
};
use core::{
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Duration,
};
use lazy_static::lazy_static;

#[test]
fn spawn() {
    lazy_static! {
        static ref SEM: Semaphore = Semaphore::new();
        static ref VAL: AtomicBool = AtomicBool::new(false);
    }
    task::spawn(Priority::default(), || {
        task::sleep(Duration::from_millis(100));
        VAL.store(true, Ordering::SeqCst);
        assert!(SEM.try_give());
    });
    SEM.take();
    assert!(VAL.load(Ordering::SeqCst));
}

#[test]
fn priority() {
    lazy_static! {
        static ref SEM: Semaphore = Semaphore::new();
        static ref VAL: AtomicUsize = AtomicUsize::new(0);
    }
    const COUNT: usize = 16;
    for i in (0..COUNT).rev() {
        task::spawn(Priority::default() + i, move || {
            SEM.take();
            assert_eq!(VAL.load(Ordering::SeqCst), i);
        });
    }
    task::sleep(Duration::from_millis(100));
    for _ in 0..COUNT {
        while !SEM.try_give() {
            task::sleep(Duration::from_micros(100));
        }
        VAL.fetch_add(1, Ordering::SeqCst);
    }
}
