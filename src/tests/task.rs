use crate::{io::println, sync::Semaphore, task, test};
use core::{
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Duration,
};
use lazy_static::lazy_static;
use macro_rules_attribute::apply;

#[apply(test)]
fn spawn() {
    lazy_static! {
        static ref SEM: Semaphore = Semaphore::new().unwrap();
        static ref VAL: AtomicBool = AtomicBool::new(false);
    }

    println!("spawn");
    let task = task::spawn(|| {
        println!("task sleep");
        task::sleep(Some(Duration::from_millis(100)));
        VAL.store(true, Ordering::SeqCst);
        println!("task give");
        assert!(SEM.give());
        println!("task donw");
    })
    .unwrap();

    println!("take");
    SEM.take(None);
    assert!(VAL.load(Ordering::SeqCst));

    println!("join");
    task.join(None);

    println!("done");
}

#[apply(test)]
fn priority() {
    use crate::task::Priority;

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

#[apply(test)]
fn ping_pong() {
    lazy_static! {
        static ref ISEM: Semaphore = Semaphore::new().unwrap();
        static ref OSEM: Semaphore = Semaphore::new().unwrap();
        static ref VAL: AtomicUsize = AtomicUsize::new(0);
    }
    const N: usize = 1024;

    let prod = task::Builder::new()
        .spawn(|| {
            for i in 0..N {
                assert!(ISEM.take(None));
                assert_eq!(VAL.fetch_add(1, Ordering::SeqCst), 2 * i);
                assert!(OSEM.give());
            }
        })
        .unwrap();

    let cons = task::Builder::new()
        .spawn(move || {
            assert!(ISEM.give());
            for i in 0..N {
                assert!(OSEM.take(None));
                assert_eq!(VAL.fetch_add(1, Ordering::SeqCst), 2 * i + 1);
                assert!(ISEM.give());
            }
        })
        .unwrap();

    prod.join(None);
    cons.join(None);
}
