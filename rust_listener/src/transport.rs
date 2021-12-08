
pub trait Transport {
    fn put(&mut self, c: u8) -> Option<Box<Vec<u8>>>;
}
