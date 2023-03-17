use core::time::Duration;
use macro_rules_attribute::apply;
use ustd::task::BlockingContext;

#[apply(ustd::main)]
fn main(cx: &mut ustd::task::TaskContext) {
    println!("Main task: {:?}", cx.task().id(),);
    cx.sleep(Some(Duration::from_millis(100)));
    let handle = ustd::task::Builder::new()
        .priority(2)
        .spawn(|cx| {
            println!("Spawned task (inside): {:?}", cx.task().id(),);
        })
        .unwrap();
    println!("Spawned task (outside): {:?}", handle.task().id(),);

    handle.join(cx, None);
}
