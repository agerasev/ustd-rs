/// Entry point of ustd program.
#[macro_export]
macro_rules! main {
    (fn main() $body:block) => {
        fn main() {
            let enter = $crate::backend::task::enter();
            {
                $body
            }
            drop(enter);
        }
    };
}

/// Run test in ustd environment.
#[macro_export]
macro_rules! test {
    (fn $name:ident() $body:block) => {
        #[test]
        fn $name() {
            let enter = $crate::backend::task::enter();
            {
                $body
            }
            drop(enter);
        }
    };
}
