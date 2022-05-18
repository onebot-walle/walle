pub trait MaybeVec<T> {
    fn list_or_t(self) -> Vec<T>;
}

impl<T> MaybeVec<T> for Vec<T> {
    fn list_or_t(self) -> Vec<T> {
        self
    }
}

impl<T> MaybeVec<T> for T {
    fn list_or_t(self) -> Vec<T> {
        vec![self]
    }
}
