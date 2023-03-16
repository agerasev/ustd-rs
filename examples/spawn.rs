use core::time::Duration;
use macro_rules_attribute::apply;
use ustd::task;

#[apply(ustd::main)]
fn main() {
    println!("Main task: {:?}", task::current().id(),);
    task::sleep(Some(Duration::from_millis(100)));
    let handle = task::Builder::new()
        .priority(2)
        .spawn(|| {
            println!("Spawned task (inside): {:?}", task::current().id(),);
        })
        .unwrap();
    println!("Spawned task (outside): {:?}", handle.task().id(),);

    handle.join(None);
}
