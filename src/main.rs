#[cfg(feature = "test-freertos")]
mod tests;

use ustd::*;

tests_main![
    tests::task::spawn,
    tests::task::priority,
    tests::task::ping_pong,
];
