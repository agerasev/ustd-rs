use crate::{println, task};
use freertos::FreeRtosUtils;

fn run_test<F: FnOnce()>(name: &'static str, f: F) {
    f();
    println!("test {} ... ok", name);
}

pub fn run_tests<I: Iterator<Item = (&'static str, fn())>>(tests: I) {
    let mut count = 0;
    for (name, test) in tests {
        task::spawn(move || run_test(name, test)).unwrap();
        count += 1;
    }
    println!("running {} tests", count);
    FreeRtosUtils::start_scheduler();
}
