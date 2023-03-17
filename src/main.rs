#[cfg(feature = "test-freertos")]
use ustd::tests;
use ustd::tests_main;

tests_main![
    tests::task::spawn,
    tests::task::priority,
    tests::task::ping_pong,
];
