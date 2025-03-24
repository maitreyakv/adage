pub trait Layer<C> {
    type Resource;
    type Key;

    fn provide(&self, key: &Self::Key, ctx: C) -> Self::Resource;
}

pub trait Context<R, K> {
    fn get(&self, key: K) -> R;
}
