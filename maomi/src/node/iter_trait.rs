pub trait MutIterator<'a> {
    type Item;

    fn next(&'a mut self) -> Option<Self::Item> where Self::Item: 'a;
}
