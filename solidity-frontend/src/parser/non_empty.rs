use std::convert::TryFrom;

#[derive(Debug)]
pub struct EmptyVectorError;

pub struct NonEmpty<T>(pub Vec<T>);

impl<T> NonEmpty<T> {
    pub(crate) fn new(first: T) -> NonEmpty<T> {
        NonEmpty(vec![first])
    }
}

impl<T> TryFrom<Vec<T>> for NonEmpty<T> {
    type Error = EmptyVectorError;
    fn try_from(source: Vec<T>) -> Result<NonEmpty<T>, Self::Error> {
        if source.is_empty() {
            Err(EmptyVectorError)
        } else {
            Ok(NonEmpty(source))
        }
    }
}

impl<T: Clone> TryFrom<&[T]> for NonEmpty<T> {
    type Error = EmptyVectorError;
    fn try_from(source: &[T]) -> Result<NonEmpty<T>, Self::Error> {
        NonEmpty::try_from(source.to_vec())
    }
}

impl<T> Into<Vec<T>> for NonEmpty<T> {
    fn into(self) -> Vec<T> {
        self.0
    }
}

impl<T> AsRef<[T]> for NonEmpty<T> {
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<T: Clone> Clone for NonEmpty<T> {
    fn clone(&self) -> NonEmpty<T> {
        NonEmpty(self.0.clone())
    }
}
