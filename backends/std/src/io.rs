extern crate std;

#[repr(transparent)]
pub struct Stdout(pub std::io::Stdout);

pub fn stdout() -> Stdout {
    Stdout(std::io::stdout())
}
