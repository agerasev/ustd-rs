/// Entry point of ustd program.
#[macro_export]
macro_rules! main {
    ($( #[$attr:meta] )* $vis:vis fn $name:ident($cx:ident: $cx_ty:ty) $body:block) => {
        $( #[$attr] )*
        $vis fn $name() {
            let mut __cx = $crate::task::TaskContext::enter();
            let $cx: $cx_ty = &mut __cx;
            {
                $body
            }
            drop(__cx);
        }
    };
}

/// Run test in ustd environment.
#[macro_export]
macro_rules! test {
    ($( #[$attr:meta] )* $vis:vis fn $name:ident($cx:ident: $cx_ty:ty) $body:block) => {
        $( #[$attr] )*
        #[test]
        $vis fn $name() {
            let mut __cx = $crate::task::TaskContext::enter();
            let $cx: $cx_ty = &mut __cx;
            {
                $body
            }
            drop(__cx);
        }
    };
}

#[macro_export]
macro_rules! tests_main {
    ($( $test:path ),* $(,)?) => {
        fn main() {
            panic!("Use `cargo test`");
        }
    };
}
