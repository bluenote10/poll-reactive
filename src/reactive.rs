use std::cell::{Cell, RefCell};
use std::rc::Rc;

// Perhaps use `trait-set`?
// https://stackoverflow.com/a/70297552/1804173
pub trait CommonBound: PartialEq {}
impl<T: PartialEq> CommonBound for T {}

// Deriving clone doesn't seem to work in this case, see: https://github.com/rust-lang/rust/issues/122750
// #[derive(Clone)]
pub struct Dynamic<T>
where
    T: CommonBound,
{
    value: Rc<RefCell<T>>,
    value_id: Rc<Cell<u64>>,
}

impl<T> Clone for Dynamic<T>
where
    T: CommonBound,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            value_id: self.value_id.clone(),
        }
    }
}

impl<T> Dynamic<T>
where
    T: CommonBound,
{
    pub fn new(x: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(x)),
            value_id: Rc::new(Cell::new(0)),
        }
    }

    pub fn set(&self, y: T) {
        let has_changed = { *self.value.borrow() != y };
        if has_changed {
            self.value.replace(y);
            self.value_id.set(self.value_id.get() + 1);
        }
    }

    pub fn update(&self, f: impl FnOnce(&T) -> T) {
        let (y, has_changed) = {
            let x = self.value.borrow();
            let y = f(&x);
            let has_changed = *x != y;
            (y, has_changed)
        };
        if has_changed {
            self.value.replace(y);
            self.value_id.set(self.value_id.get() + 1);
        }
    }

    pub fn update_inplace(&self, f: impl FnOnce(&mut T) -> bool) {
        let has_changed = {
            let mut x = self.value.borrow_mut();
            f(&mut x)
        };
        if has_changed {
            self.value_id.set(self.value_id.get() + 1);
        }
    }
}

// Direct getters

pub trait DirectGet<T> {
    fn get(&self) -> T;
}

impl<T> DirectGet<T> for Dynamic<T>
where
    T: Clone + CommonBound,
{
    fn get(&self) -> T {
        self.value.borrow().clone()
    }
}

impl<T> DirectGet<T> for Consumer<T>
where
    T: Clone + CommonBound,
{
    fn get(&self) -> T {
        self.dynamic.value.borrow().clone()
    }
}

// Consumer<T>
// Monitor<T>
// React<T>
// Signal<T>

pub struct Consumer<T>
where
    T: CommonBound,
{
    dynamic: Dynamic<T>,
    consumed_id: Cell<u64>,
}

impl<T> Consumer<T>
where
    T: CommonBound,
{
    pub fn new(dynamic: Dynamic<T>) -> Self {
        Self {
            dynamic,
            consumed_id: Cell::new(u64::MAX),
        }
    }
}

// OnChange trait

pub trait OnChange<F> {
    fn on_change(&self, f: F);
}

impl<A, F> OnChange<F> for Consumer<A>
where
    A: CommonBound,
    F: FnOnce(&A),
{
    fn on_change(&self, f: F) {
        let value_id = self.dynamic.value_id.get();
        // println!("{} {}", value_id, self.consumed_id.get());
        if value_id != self.consumed_id.get() {
            f(&self.dynamic.value.borrow());
            self.consumed_id.set(value_id);
        }
    }
}

pub struct Consumer2<A, B>(Consumer<A>, Consumer<B>)
where
    A: CommonBound,
    B: CommonBound;

impl<A, B> Consumer2<A, B>
where
    A: CommonBound,
    B: CommonBound,
{
    pub fn new(dynamics: (Dynamic<A>, Dynamic<B>)) -> Self {
        Self(Consumer::new(dynamics.0), Consumer::new(dynamics.1))
    }
}

impl<A, B, F> OnChange<F> for Consumer2<A, B>
where
    A: CommonBound,
    B: CommonBound,
    F: FnOnce((&A, &B)),
{
    fn on_change(&self, f: F) {
        let value_id0 = self.0.dynamic.value_id.get();
        let value_id1 = self.1.dynamic.value_id.get();
        if value_id0 != self.0.consumed_id.get() || value_id1 != self.1.consumed_id.get() {
            f((
                &self.0.dynamic.value.borrow(),
                &self.1.dynamic.value.borrow(),
            ));
            self.0.consumed_id.set(value_id0);
            self.1.consumed_id.set(value_id1);
        }
    }
}

pub struct Consumer3<A, B, C>(Consumer<A>, Consumer<B>, Consumer<C>)
where
    A: CommonBound,
    B: CommonBound,
    C: CommonBound;

impl<A, B, C> Consumer3<A, B, C>
where
    A: CommonBound,
    B: CommonBound,
    C: CommonBound,
{
    pub fn new(dynamics: (Dynamic<A>, Dynamic<B>, Dynamic<C>)) -> Self {
        Self(
            Consumer::new(dynamics.0),
            Consumer::new(dynamics.1),
            Consumer::new(dynamics.2),
        )
    }
}

impl<A, B, C, F> OnChange<F> for Consumer3<A, B, C>
where
    A: CommonBound,
    B: CommonBound,
    C: CommonBound,
    F: FnOnce((&A, &B, &C)),
{
    fn on_change(&self, f: F) {
        let value_id0 = self.0.dynamic.value_id.get();
        let value_id1 = self.1.dynamic.value_id.get();
        let value_id2 = self.2.dynamic.value_id.get();
        if value_id0 != self.0.consumed_id.get()
            || value_id1 != self.1.consumed_id.get()
            || value_id2 != self.2.consumed_id.get()
        {
            f((
                &self.0.dynamic.value.borrow(),
                &self.1.dynamic.value.borrow(),
                &self.2.dynamic.value.borrow(),
            ));
            self.0.consumed_id.set(value_id0);
            self.1.consumed_id.set(value_id1);
            self.2.consumed_id.set(value_id2);
        }
    }
}

// TODO: Extend for higher orders...

// ----------------------------------------------------------------------------
// IntoConsumer
// ----------------------------------------------------------------------------

pub trait IntoConsumer {
    type Output;
    fn into_consumer(&self) -> Self::Output;
}

impl<A> IntoConsumer for Dynamic<A>
where
    A: CommonBound,
{
    type Output = Consumer<A>;
    fn into_consumer(&self) -> Self::Output {
        Consumer::new(self.clone())
    }
}

impl<A, B> IntoConsumer for (Dynamic<A>, Dynamic<B>)
where
    A: CommonBound,
    B: CommonBound,
{
    type Output = Consumer2<A, B>;
    fn into_consumer(&self) -> Self::Output {
        Consumer2::new(self.clone())
    }
}

impl<A, B, C> IntoConsumer for (Dynamic<A>, Dynamic<B>, Dynamic<C>)
where
    A: CommonBound,
    B: CommonBound,
    C: CommonBound,
{
    type Output = Consumer3<A, B, C>;
    fn into_consumer(&self) -> Self::Output {
        Consumer3::new(self.clone())
    }
}

// TODO: Extend for higher orders...

// ----------------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        let dynamic_a = Dynamic::new(0);
        let dynamic_b = dynamic_a.clone();
        {
            assert_eq!(*dynamic_a.value.borrow(), 0);
            assert_eq!(*dynamic_b.value.borrow(), 0);
            assert_eq!(dynamic_a.value_id.get(), 0);
            assert_eq!(dynamic_b.value_id.get(), 0);
        }

        dynamic_a.set(10);
        {
            assert_eq!(*dynamic_a.value.borrow(), 10);
            assert_eq!(*dynamic_b.value.borrow(), 10);
            assert_eq!(dynamic_a.value_id.get(), 1);
            assert_eq!(dynamic_b.value_id.get(), 1);
        }
        dynamic_a.set(10);
        {
            assert_eq!(*dynamic_a.value.borrow(), 10);
            assert_eq!(*dynamic_b.value.borrow(), 10);
            assert_eq!(dynamic_a.value_id.get(), 1);
            assert_eq!(dynamic_b.value_id.get(), 1);
        }

        dynamic_a.update(|x| x * 2);
        {
            assert_eq!(*dynamic_a.value.borrow(), 20);
            assert_eq!(*dynamic_b.value.borrow(), 20);
            assert_eq!(dynamic_a.value_id.get(), 2);
            assert_eq!(dynamic_b.value_id.get(), 2);
        }
        dynamic_a.update(|x| *x);
        {
            assert_eq!(*dynamic_a.value.borrow(), 20);
            assert_eq!(*dynamic_b.value.borrow(), 20);
            assert_eq!(dynamic_a.value_id.get(), 2);
            assert_eq!(dynamic_b.value_id.get(), 2);
        }

        dynamic_a.update_inplace(|x| {
            *x = 30;
            true
        });
        {
            assert_eq!(*dynamic_a.value.borrow(), 30);
            assert_eq!(*dynamic_b.value.borrow(), 30);
            assert_eq!(dynamic_a.value_id.get(), 3);
            assert_eq!(dynamic_b.value_id.get(), 3);
        }
        dynamic_a.update_inplace(|_x| false);
        {
            assert_eq!(*dynamic_a.value.borrow(), 30);
            assert_eq!(*dynamic_b.value.borrow(), 30);
            assert_eq!(dynamic_a.value_id.get(), 3);
            assert_eq!(dynamic_b.value_id.get(), 3);
        }
    }

    #[test]
    fn direct_get() {
        let dynamic = Dynamic::new(42);
        assert_eq!(dynamic.get(), 42);
        let consumer = dynamic.into_consumer();
        assert_eq!(consumer.get(), 42);

        // Non-copy
        #[derive(Clone, PartialEq, Debug)]
        struct Foo(i32);

        let dynamic = Dynamic::new(Foo(42));
        assert_eq!(dynamic.get(), Foo(42));
        let consumer = dynamic.into_consumer();
        assert_eq!(consumer.get(), Foo(42));
    }

    #[test]
    fn support_for_no_clone_no_copy() {
        // Non-copy/clone
        #[derive(PartialEq, Debug)]
        struct Foo(i32);

        let dynamic = Dynamic::new(Foo(42));
        let consumer = dynamic.into_consumer();

        let mut inner = 0;

        consumer.on_change(|x| inner = x.0);
        assert_eq!(inner, 42);

        dynamic.update(|foo| Foo(foo.0 + 1));

        consumer.on_change(|x| inner = x.0);
        assert_eq!(inner, 43);
    }

    #[test]
    fn two_dynamics() {
        let dynamic_a = Dynamic::new(10);
        let dynamic_b = Dynamic::new(20);

        let consumer_a = Consumer::new(dynamic_a.clone());
        let consumer_b = Consumer::new(dynamic_b.clone());
        let consumer_ab = (dynamic_a.clone(), dynamic_b.clone()).into_consumer();

        let poll = || {
            let mut res_a = None;
            let mut res_b = None;
            let mut res_ab = None;
            consumer_a.on_change(|a| res_a = Some(*a));
            consumer_b.on_change(|b| res_b = Some(*b));
            consumer_ab.on_change(|(a, b)| res_ab = Some((*a, *b)));
            (res_a, res_b, res_ab)
        };

        // Initial poll
        let (res_a, res_b, res_ab) = poll();
        assert_eq!(res_a, Some(10));
        assert_eq!(res_b, Some(20));
        assert_eq!(res_ab, Some((10, 20)));

        let (res_a, res_b, res_ab) = poll();
        assert_eq!(res_a, None);
        assert_eq!(res_b, None);
        assert_eq!(res_ab, None);

        // Update a
        dynamic_a.update(|a| a + 1);
        let (res_a, res_b, res_ab) = poll();
        assert_eq!(res_a, Some(11));
        assert_eq!(res_b, None);
        assert_eq!(res_ab, Some((11, 20)));

        // Update b
        dynamic_b.update(|b| b + 2);
        let (res_a, res_b, res_ab) = poll();
        assert_eq!(res_a, None);
        assert_eq!(res_b, Some(22));
        assert_eq!(res_ab, Some((11, 22)));

        // Update both
        dynamic_a.update(|a| a + 10);
        dynamic_b.update(|b| b + 11);
        let (res_a, res_b, res_ab) = poll();
        assert_eq!(res_a, Some(21));
        assert_eq!(res_b, Some(33));
        assert_eq!(res_ab, Some((21, 33)));

        // Reset
        dynamic_a.set(10);
        dynamic_b.set(20);
        let (res_a, res_b, res_ab) = poll();
        assert_eq!(res_a, Some(10));
        assert_eq!(res_b, Some(20));
        assert_eq!(res_ab, Some((10, 20)));

        // No-op mutations
        dynamic_a.set(10);
        dynamic_b.set(20);
        let (res_a, res_b, res_ab) = poll();
        assert_eq!(res_a, None);
        assert_eq!(res_b, None);
        assert_eq!(res_ab, None);

        dynamic_a.update(|x| *x);
        dynamic_b.update(|x| *x);
        let (res_a, res_b, res_ab) = poll();
        assert_eq!(res_a, None);
        assert_eq!(res_b, None);
        assert_eq!(res_ab, None);

        dynamic_a.update_inplace(|_x| false);
        dynamic_b.update_inplace(|_x| false);
        let (res_a, res_b, res_ab) = poll();
        assert_eq!(res_a, None);
        assert_eq!(res_b, None);
        assert_eq!(res_ab, None);
    }

    #[test]
    fn consumer_3() {
        let dynamic_a = Dynamic::new(10);
        let dynamic_b = Dynamic::new(20);
        let dynamic_c = Dynamic::new(30);

        let consumer = (dynamic_a.clone(), dynamic_b.clone(), dynamic_c.clone()).into_consumer();

        let poll = || {
            let mut res = None;
            consumer.on_change(|(a, b, c)| res = Some((*a, *b, *c)));
            res
        };

        // Initial poll
        let res = poll();
        assert_eq!(res, Some((10, 20, 30)));

        let res = poll();
        assert_eq!(res, None);

        dynamic_a.update(|a| a + 1);
        let res = poll();
        assert_eq!(res, Some((11, 20, 30)));

        dynamic_b.update(|b| b + 2);
        let res = poll();
        assert_eq!(res, Some((11, 22, 30)));

        dynamic_c.update(|c| c + 3);
        let res = poll();
        assert_eq!(res, Some((11, 22, 33)));
    }
}
