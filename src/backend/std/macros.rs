/// Entry point of ustd program.
#[macro_export]
macro_rules! main {
    (fn main($cx:ident: $cx_ty:ty) $body:block) => {
        fn main() {
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
    (fn $name:ident($cx:ident: $cx_ty:ty) $body:block) => {
        #[test]
        fn $name() {
            let mut __cx = $crate::task::TaskContext::enter();
            let $cx: $cx_ty = &mut __cx;
            {
                $body
            }
            drop(__cx);
        }
    };
}
