#[macro_export]
macro_rules! main {
    (fn main($cx:ident: $cx_ty:ty) $body:block) => {
        fn main() {
            $crate::task::spawn(|$cx| $body).unwrap();
            $crate::println!("Start scheduler");
            $crate::freertos::FreeRtosUtils::start_scheduler();
        }
    };
}

#[macro_export]
macro_rules! test {
    (fn $name:ident($cx:ident: $cx_ty:ty) $body:block) => {
        pub fn $name($cx: $cx_ty) $body
    };
}

#[macro_export]
macro_rules! run_tests {
    ($( $test:path ),* $(,)?) => {
        let mut count = 0;
        for (name, test) in [$( (stringify!($test), $test as fn(cx: &mut $crate::task::TaskContext)) ),*] {
            $crate::task::spawn(move |cx| {
                test(cx);
                println!("test {} ... ok", name);
            }).unwrap();
            count += 1;
        }
        println!("running {} tests", count);
        $crate::freertos::FreeRtosUtils::start_scheduler();
    };
}
