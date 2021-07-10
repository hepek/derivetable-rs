pub use derivetable_derive::Table;

pub struct IndexIterator<'a, I: Iterator<Item=usize>, R> {
    pub data: &'a Vec<R>,
    pub idxs: I,
}

impl<'a, I, R> Iterator for IndexIterator<'a, I, R>
where
    I: Iterator<Item=usize>
{
    type Item = (usize, &'a R);
    fn next(&mut self) -> Option<Self::Item> {
        self.idxs.next()
           .map(|idx| { 
               let row = &self.data[idx];
               (idx, row)
           })
    }
}

pub struct IndexDoubleEndedIterator<'a, I: DoubleEndedIterator<Item=usize>, R> {
    pub data: &'a Vec<R>,
    pub idxs: I,
}

impl<'a, I, R> Iterator for IndexDoubleEndedIterator<'a, I, R>
where
    I: DoubleEndedIterator<Item=usize>
{
    type Item = (usize, &'a R);
    fn next(&mut self) -> Option<Self::Item> {
        self.idxs.next()
           .map(|idx| { 
               let row = &self.data[idx]; 
               (idx, row)
           })
    }
}

impl<'a, I, R> DoubleEndedIterator for IndexDoubleEndedIterator<'a, I, R>
where
    I: DoubleEndedIterator<Item=usize>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.idxs.next_back()
           .map(|idx| { 
               let row = &self.data[idx];
               (idx, row)
           })
    }
}

