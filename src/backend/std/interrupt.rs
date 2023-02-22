pub struct InterruptContext {
    _unused: [u8; 0],
}

impl InterruptContext {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { _unused: [] }
    }
}
impl Drop for InterruptContext {
    fn drop(&mut self) {}
}
