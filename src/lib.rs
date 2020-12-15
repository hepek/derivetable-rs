pub use derivetable_derive::Table;

pub struct IndexIterator<'a, I: DoubleEndedIterator<Item=&'a usize> + 'a, R> {
    pub data: &'a std::collections::BTreeMap<usize, R>,
    pub idxs: I,
}

impl<'a, I, R> Iterator for IndexIterator<'a, I, R>
where
    I: DoubleEndedIterator<Item=&'a usize> + 'a
{
    type Item = (&'a usize, &'a R);
    fn next(&mut self) -> Option<Self::Item> {
        self.idxs.next()
           .map(|idx| { 
               let row = self.data.get(idx).unwrap(); // filtermap instead of panic?
               (idx, row)
           })
    }
}

impl<'a, I, R> DoubleEndedIterator for IndexIterator<'a, I, R>
where
    I: DoubleEndedIterator<Item=&'a usize> + 'a
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.idxs.next_back()
           .map(|idx| { 
               let row = self.data.get(idx).unwrap(); // filtermap instead of panic?
               (idx, row)
           })
    }
}
