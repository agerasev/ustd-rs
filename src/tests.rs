use crate::{
    sync::Semaphore,
    task::{self, Priority},
};
use core::{
    sync::atomic::{AtomicBool, Ordering},
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
