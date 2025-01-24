use std::iter::Peekable;

pub struct IterDone<T: Iterator> {
    iter: Peekable<T>,
}

impl<T: Iterator> Iterator for IterDone<T> {
    type Item = (bool, T::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next();
        let done = self.iter.peek().is_none();
        item.map(|v| (done, v))
    }
}

pub trait IterDoneExt<T: Iterator> {
    fn until_done(self) -> IterDone<T>;
}

impl<T: Iterator> IterDoneExt<T> for T {
    fn until_done(self) -> IterDone<T> {
        IterDone {
            iter: self.peekable(),
        }
    }
}
