use ustd::{run_tests, tests};

fn main() {
    run_tests![
        tests::task::spawn,
        tests::task::priority,
        tests::task::ping_pong,
    ];
}
