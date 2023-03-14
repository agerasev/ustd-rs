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
        static ref SEM: Semaphore = Semaphore::new().unwrap();
        static ref VAL: AtomicBool = AtomicBool::new(false);
    }

    task::spawn(Priority::default(), || {
        task::sleep(Duration::from_millis(100));
        VAL.store(true, Ordering::SeqCst);
        assert!(SEM.try_give());
    })
    .unwrap();

    task::spawn(Priority::default() + 1, || {
        SEM.take();
        assert!(VAL.load(Ordering::SeqCst));
    })
    .unwrap();
}

#[test]
fn priority() {
    lazy_static! {
        static ref SEM: Semaphore = Semaphore::new().unwrap();
        static ref VAL: AtomicUsize = AtomicUsize::new(0);
    }
    const COUNT: usize = 5;

    for i in (0..COUNT).rev() {
        task::spawn(Priority::default() + i, move || {
            SEM.take();
            assert_eq!(VAL.load(Ordering::SeqCst), i);
        })
        .unwrap();
    }

    task::spawn(Priority::default() + COUNT, move || {
        task::sleep(Duration::from_millis(100));
        for _ in 0..COUNT {
            while !SEM.try_give() {
                task::sleep(Duration::from_micros(100));
            }
            VAL.fetch_add(1, Ordering::SeqCst);
        }
    })
    .unwrap();
}

#[test]
fn ping_pong() {
    lazy_static! {
        static ref ISEM: Semaphore = Semaphore::new().unwrap();
        static ref OSEM: Semaphore = Semaphore::new().unwrap();
        static ref VAL: AtomicUsize = AtomicUsize::new(0);
    }
    const N: usize = 1024;

    task::spawn(Priority::default(), || {
        for i in 0..N {
            ISEM.take();
            assert_eq!(VAL.fetch_add(1, Ordering::SeqCst), 2 * i);
            assert!(OSEM.try_give());
        }
    })
    .unwrap();

    task::spawn(Priority::default() + 1, move || {
        assert!(ISEM.try_give());
        for i in 0..N {
            OSEM.take();
            assert!(ISEM.try_give());
            assert_eq!(VAL.fetch_add(1, Ordering::SeqCst), 2 * i + 1);
        }
    })
    .unwrap();
}
