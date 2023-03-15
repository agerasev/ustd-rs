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

    task::spawn(|| {
        task::sleep(Some(Duration::from_millis(100)));
        VAL.store(true, Ordering::SeqCst);
        assert!(SEM.give());
    })
    .unwrap();

    task::spawn(|| {
        SEM.take(None);
        assert!(VAL.load(Ordering::SeqCst));
    })
    .unwrap();
}

#[test]
//#[cfg(feature = "backend-freertos")]
fn priority() {
    lazy_static! {
        static ref SEM: Semaphore = Semaphore::new().unwrap();
        static ref VAL: AtomicUsize = AtomicUsize::new(0);
    }
    const COUNT: usize = 5;

    for i in (0..COUNT).rev() {
        task::Builder::new()
            .priority(i as Priority)
            .spawn(move || {
                SEM.take(None);
                assert_eq!(VAL.load(Ordering::SeqCst), i);
            })
            .unwrap();
    }

    task::Builder::new()
        .priority(COUNT as Priority)
        .spawn(move || {
            task::sleep(Some(Duration::from_millis(100)));
            for _ in 0..COUNT {
                while !SEM.give() {
                    task::sleep(Some(Duration::from_micros(100)));
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

    task::spawn(|| {
        for i in 0..N {
            ISEM.take(None);
            assert_eq!(VAL.fetch_add(1, Ordering::SeqCst), 2 * i);
            assert!(OSEM.give());
        }
    })
    .unwrap();

    task::spawn(move || {
        assert!(ISEM.give());
        for i in 0..N {
            OSEM.take(None);
            assert!(ISEM.give());
            assert_eq!(VAL.fetch_add(1, Ordering::SeqCst), 2 * i + 1);
        }
    })
    .unwrap();
}
