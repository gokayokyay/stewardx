use futures::Stream;
pub type BoxedStream = Box<dyn Stream<Item = u8> + Unpin>;

