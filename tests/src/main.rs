mod tasks;

use ustd::*;

tests_main![tasks::spawn, tasks::priority, tasks::ping_pong];
