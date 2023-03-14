use core::time::Duration;
use freertos::FreeRtosUtils;
use ustd::task::{self, Priority};

fn main() {
    task::spawn(Priority::default(), || {
        task::sleep(Duration::from_millis(100));
        println!("Hello, FreeRTOS!");
    })
    .unwrap();

    FreeRtosUtils::start_scheduler();
}
