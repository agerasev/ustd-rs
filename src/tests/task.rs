use crate::{
    io::println,
    sync::Semaphore,
    task::{self, BlockingContext, TaskContext},
    test,
};
use core::{
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Duration,
};
use lazy_static::lazy_static;
use macro_rules_attribute::apply;

#[apply(test)]
fn spawn(cx: &mut TaskContext) {
    lazy_static! {
        static ref SEM: Semaphore = Semaphore::new().unwrap();
        static ref VAL: AtomicBool = AtomicBool::new(false);
    }

    println!("spawn");
    let task = task::spawn(|cx| {
        println!("task sleep");
        cx.sleep(Some(Duration::from_millis(100)));
        VAL.store(true, Ordering::SeqCst);
        println!("task give");
        assert!(SEM.try_give(cx));
        println!("task done");
    })
    .unwrap();

    println!("take");
    SEM.take(cx, None);
    assert!(VAL.load(Ordering::SeqCst));

    println!("join");
    task.join(cx, None);

    println!("done");
}

#[apply(test)]
fn priority(cx: &mut TaskContext) {
    use crate::task::Priority;

    lazy_static! {
        static ref SEM: Semaphore = Semaphore::new().unwrap();
        static ref VAL: AtomicUsize = AtomicUsize::new(0);
    }
    const COUNT: usize = 5;

    for i in (0..COUNT).rev() {
        task::Builder::new()
            .priority(i as Priority)
            .spawn(move |cx| {
                SEM.take(cx, None);
                assert_eq!(VAL.load(Ordering::SeqCst), i);
            })
            .unwrap();
    }

    task::Builder::new()
        .priority(COUNT as Priority)
        .spawn(move |cx| {
            cx.sleep(Some(Duration::from_millis(100)));
            for _ in 0..COUNT {
                while !SEM.try_give(cx) {
                    cx.sleep(Some(Duration::from_micros(100)));
                }
                VAL.fetch_add(1, Ordering::SeqCst);
            }
        })
        .unwrap()
        .join(cx, None);
}

#[apply(test)]
fn ping_pong(cx: &mut TaskContext) {
    lazy_static! {
        static ref ISEM: Semaphore = Semaphore::new().unwrap();
        static ref OSEM: Semaphore = Semaphore::new().unwrap();
        static ref VAL: AtomicUsize = AtomicUsize::new(0);
    }
    const N: usize = 1024;

    let prod = task::Builder::new()
        .spawn(|cx| {
            for i in 0..N {
                assert!(ISEM.take(cx, None));
                assert_eq!(VAL.fetch_add(1, Ordering::SeqCst), 2 * i);
                assert!(OSEM.try_give(cx));
            }
        })
        .unwrap();

    let cons = task::Builder::new()
        .spawn(move |cx| {
            assert!(ISEM.try_give(cx));
            for i in 0..N {
                assert!(OSEM.take(cx, None));
                assert_eq!(VAL.fetch_add(1, Ordering::SeqCst), 2 * i + 1);
                assert!(ISEM.try_give(cx));
            }
        })
        .unwrap();

    prod.join(cx, None);
    cons.join(cx, None);
}
