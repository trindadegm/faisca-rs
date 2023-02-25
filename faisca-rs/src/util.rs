pub struct OnDropDefer<T, F: FnOnce(T)>(Option<(T, F)>);

impl<T, F> OnDropDefer<T, F>
where
    F: FnOnce(T),
{
    #[inline(always)]
    pub fn new(v: T, on_drop: F) -> Self {
        Self(Some((v, on_drop)))
    }

    #[inline(always)]
    pub fn take(mut self) -> T {
        let mut mule = None;
        std::mem::swap(&mut self.0, &mut mule);
        mule.unwrap().0
    }
}
impl<T, F> AsRef<T> for OnDropDefer<T, F>
where
    F: FnOnce(T),
{
    #[inline(always)]
    fn as_ref(&self) -> &T {
        &self.0.as_ref().unwrap().0
    }
}
impl<T, F> AsMut<T> for OnDropDefer<T, F>
where
    F: FnOnce(T),
{
    #[inline(always)]
    fn as_mut(&mut self) -> &mut T {
        &mut self.0.as_mut().unwrap().0
    }
}
impl<T, F> Drop for OnDropDefer<T, F>
where
    F: FnOnce(T),
{
    #[inline]
    fn drop(&mut self) {
        let mut mule = None;
        std::mem::swap(&mut self.0, &mut mule);
        if let Some((v, on_drop)) = mule {
            on_drop(v);
        }
    }
}
