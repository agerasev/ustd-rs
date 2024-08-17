use crate::{
    println,
    task::{self, TaskContext},
};
use core::sync::atomic::{AtomicUsize, Ordering};

extern "C" {
    fn __ustd_exit() -> !;
}
fn exit() -> ! {
    unsafe { __ustd_exit() }
}

pub fn run_tests<I: Iterator<Item = (&'static str, fn(&mut TaskContext))>>(iter: I) {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    for (name, test) in iter {
        task::spawn(move |cx| {
            test(cx);
            println!("test {} ... ok", name);
            if COUNTER.fetch_sub(1, Ordering::AcqRel) == 1 {
                println!();
                println!("test result: ok.");
                exit();
            }
        })
        .unwrap();
        COUNTER.fetch_add(1, Ordering::AcqRel);
    }
    println!("running {} tests", COUNTER.load(Ordering::Acquire));
    freertos::FreeRtosUtils::start_scheduler();
}
