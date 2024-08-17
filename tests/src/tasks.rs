extern crate alloc;

use alloc::sync::Arc;
use core::{
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Duration,
};
use macro_rules_attribute::apply;
use ustd::{
    sync::Semaphore,
    task::{self, BlockingContext, TaskContext},
    test,
};

const SMALL_TIMEOUT: Option<Duration> = Some(Duration::from_millis(10));
const BIG_TIMEOUT: Option<Duration> = Some(Duration::from_secs(1));

#[apply(test)]
fn spawn(cx: &mut TaskContext) {
    struct Shared {
        sem: Semaphore,
        val: AtomicBool,
    }
    let sh = Arc::new(Shared {
        sem: Semaphore::new().unwrap(),
        val: AtomicBool::new(false),
    });

    let task = task::spawn({
        let sh = sh.clone();
        move |cx| {
            cx.sleep(SMALL_TIMEOUT);
            sh.val.store(true, Ordering::SeqCst);
            assert!(sh.sem.try_give(cx));
        }
    })
    .unwrap();

    assert!(sh.sem.take(cx, BIG_TIMEOUT));
    assert!(sh.val.load(Ordering::SeqCst));

    assert!(task.join(cx, BIG_TIMEOUT));
}

#[cfg(feature = "freertos")]
#[apply(test)]
fn priority(cx: &mut TaskContext) {
    use alloc::vec::Vec;
    use task::Priority;

    const COUNT: usize = 16;

    struct Shared {
        isem: Semaphore,
        osem: Semaphore,
        val: AtomicUsize,
    }
    let sh = Arc::new(Shared {
        isem: Semaphore::new().unwrap(),
        osem: Semaphore::new().unwrap(),
        val: AtomicUsize::new(0),
    });

    let mut tasks = Vec::new();
    for i in (0..COUNT).rev() {
        tasks.push(
            task::Builder::new()
                .priority(i as Priority)
                .spawn({
                    let sh = sh.clone();
                    move |cx| {
                        assert!(sh.isem.take(cx, BIG_TIMEOUT));
                        assert_eq!(sh.val.load(Ordering::SeqCst), COUNT - i - 1);
                        assert!(sh.osem.try_give(cx));
                    }
                })
                .unwrap(),
        );
    }

    cx.sleep(SMALL_TIMEOUT);
    for _ in 0..COUNT {
        assert!(sh.isem.try_give(cx));
        assert!(sh.osem.take(cx, BIG_TIMEOUT));
        sh.val.fetch_add(1, Ordering::SeqCst);
    }

    for h in tasks {
        assert!(h.join(cx, BIG_TIMEOUT));
    }
}

#[apply(test)]
fn ping_pong(cx: &mut TaskContext) {
    const N: usize = 1024;

    struct Shared {
        isem: Semaphore,
        osem: Semaphore,
        val: AtomicUsize,
    }
    let sh = Arc::new(Shared {
        isem: Semaphore::new().unwrap(),
        osem: Semaphore::new().unwrap(),
        val: AtomicUsize::new(0),
    });

    let prod = task::Builder::new()
        .spawn({
            let sh = sh.clone();
            move |cx| {
                for i in 0..N {
                    assert!(sh.isem.take(cx, BIG_TIMEOUT));
                    assert_eq!(sh.val.fetch_add(1, Ordering::SeqCst), 2 * i);
                    assert!(sh.osem.try_give(cx));
                }
            }
        })
        .unwrap();

    let cons = task::Builder::new()
        .spawn(move |cx| {
            assert!(sh.isem.try_give(cx));
            for i in 0..N {
                assert!(sh.osem.take(cx, BIG_TIMEOUT));
                assert_eq!(sh.val.fetch_add(1, Ordering::SeqCst), 2 * i + 1);
                assert!(sh.isem.try_give(cx));
            }
        })
        .unwrap();

    assert!(prod.join(cx, BIG_TIMEOUT));
    assert!(cons.join(cx, BIG_TIMEOUT));
}
