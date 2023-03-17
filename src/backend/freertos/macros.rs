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
    ($( #[$attr:meta] )* fn $name:ident($cx:ident: $cx_ty:ty) $body:block) => {
        $( #[$attr] )*
        pub fn $name($cx: $cx_ty) $body
    };
}

#[macro_export]
macro_rules! tests_main {
    ($( $test:path ),* $(,)?) => {
        fn main() {
            $crate::test::run_tests([$( (stringify!($test), $test as fn(&mut $crate::task::TaskContext)) ),*].into_iter());
        }
    };
}
