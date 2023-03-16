#[macro_export]
macro_rules! main {
    (fn main() $body:block) => {
        fn main() {
            $crate::task::spawn(|| $body).unwrap();
            $crate::println!("Start scheduler");
            $crate::freertos::FreeRtosUtils::start_scheduler();
        }
    };
}

#[macro_export]
macro_rules! test {
    (fn $name:ident() $body:block) => {
        pub fn $name() $body
    };
}

#[macro_export]
macro_rules! run_tests {
    ($( $test:path ),* $(,)?) => {
        let mut count = 0;
        for (name, test) in [$( (stringify!($test), $test as fn()) ),*] {
            $crate::task::spawn(move || {
                test();
                println!("test {} ... ok", name);
            }).unwrap();
            count += 1;
        }
        println!("running {} tests", count);
        $crate::freertos::FreeRtosUtils::start_scheduler();
    };
}
