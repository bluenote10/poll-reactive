# Implementation variations

Variations on implementing `OnChange`:

```rust
// This compiles, but the solution isn't so great, because the resulting multi-consumer
// type names would become very long, because they have to repeat the `Consumer` type
// for each element of the tuple...

impl<A, B, F> OnChange<F> for (Consumer<A>, Consumer<B>)
where
    A: CommonBound,
    B: CommonBound,
    F: FnOnce((&A, &B)),
{
    fn on_change(&self, f: F) {
        let value_id0 = self.0.dynamic.value_id.get();
        let value_id1 = self.1.dynamic.value_id.get();
        // println!("{} {}", value_id, self.consumed_id.get());
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

// Note that the issue in general is: Because we have implemented OnChange already for
// Dynamic<A>, we are not allowed to specialize it for `Dynamic<(A, B)>` because A is
// more generic than `(A, B)` (and already covers it).

// Note that "overloading" a type alias with variadic generic is not allowed:
// type Consumer<A, B> = (Consumer<A>, Consumer<B>);
// type Consumer<A, B, C> = (Consumer<A>, Consumer<B>, Consumer<C>);
// This would only work by disambiguating them with `Consumer2`, `Consumer3`, ...

// The next idea was to get around the "`A` is more generic than `(A, B)`" by introducing
// a special `MultiConsumer` type, which doesn't implement OnChange for a plain `A`.
// However, when only using a single generic argument for the MultiConsumer, it would result
// in very long type names like MultiConsumer<(Consumer<i32>, Consumer<i32>), because the
// `Consumer` now have to be repeated inside the generic argument. Just writing MultiConsumer<(i32, i32)>
// doesn't seem possible.

struct MultiConsumer<T>(T);

impl<A, F> OnChange<F> for MultiConsumer<(Consumer<A>,)>
where
    A: CommonBound,
    F: FnOnce(&A),
{
    fn on_change(&self, f: F) {
        let value_id = self.0 .0.dynamic.value_id.get();
        // println!("{} {}", value_id, self.consumed_id.get());
        if value_id != self.0 .0.consumed_id.get() {
            f(&self.0 .0.dynamic.value.borrow());
            self.0 .0.consumed_id.set(value_id);
        }
    }
}

impl<A, B, F> OnChange<F> for MultiConsumer<(Consumer<A>, Consumer<B>)>
where
    A: CommonBound,
    B: CommonBound,
    F: FnOnce((&A, &B)),
{
    fn on_change(&self, f: F) {
        let value_id0 = self.0 .0.dynamic.value_id.get();
        let value_id1 = self.0 .1.dynamic.value_id.get();
        // println!("{} {}", value_id, self.consumed_id.get());
        if value_id0 != self.0 .0.consumed_id.get() || value_id1 != self.0 .1.consumed_id.get() {
            f((
                &self.0 .0.dynamic.value.borrow(),
                &self.0 .1.dynamic.value.borrow(),
            ));
            self.0 .0.consumed_id.set(value_id0);
            self.0 .1.consumed_id.set(value_id1);
        }
    }
}

// This was another try, giving the MultiConsumer multiple generic argument. This "works", but
// of course this made the entire type name just even longer: MultiConsumer<(i32, i32), (Consumer<i32>, Consumer<i32>)>
// and repetitive.

struct MultiConsumer<Arg, Storage>(Storage, PhantomData<Arg>);

impl<A, F> OnChange<F> for MultiConsumer<(A,), (Consumer<A>,)>
where
    A: CommonBound,
    F: FnOnce(&A),
{
    fn on_change(&self, f: F) {
        let value_id = self.0 .0.dynamic.value_id.get();
        // println!("{} {}", value_id, self.consumed_id.get());
        if value_id != self.0 .0.consumed_id.get() {
            f(&self.0 .0.dynamic.value.borrow());
            self.0 .0.consumed_id.set(value_id);
        }
    }
}

impl<A, B, F> OnChange<F> for MultiConsumer<(A, B), (Consumer<A>, Consumer<B>)>
where
    A: CommonBound,
    B: CommonBound,
    F: FnOnce((&A, &B)),
{
    fn on_change(&self, f: F) {
        let value_id0 = self.0 .0.dynamic.value_id.get();
        let value_id1 = self.0 .1.dynamic.value_id.get();
        // println!("{} {}", value_id, self.consumed_id.get());
        if value_id0 != self.0 .0.consumed_id.get() || value_id1 != self.0 .1.consumed_id.get() {
            f((
                &self.0 .0.dynamic.value.borrow(),
                &self.0 .1.dynamic.value.borrow(),
            ));
            self.0 .0.consumed_id.set(value_id0);
            self.0 .1.consumed_id.set(value_id1);
        }
    }
}

// This was the first implementation of a Consumer2, without a trait, and using
// a "transposed" representation of the dynamics/ids.
pub struct Consumer2<A, B>
where
    A: CommonBound,
    B: CommonBound,
{
    dynamics: (Dynamic<A>, Dynamic<B>),
    consumed_ids: Cell<(u64, u64)>,
}

impl<A, B> Consumer2<A, B>
where
    A: CommonBound,
    B: CommonBound,
{
    pub fn new(dynamics: (Dynamic<A>, Dynamic<B>)) -> Self {
        Self {
            dynamics,
            consumed_ids: Cell::new((u64::MAX, u64::MAX)),
        }
    }

    pub fn on_change(&self, f: impl FnOnce((&A, &B))) {
        let value_id0 = self.dynamics.0.value_id.get();
        let value_id1 = self.dynamics.1.value_id.get();
        // println!("{} {}", value_id, self.consumed_id.get());
        if value_id0 != self.consumed_ids.get().0 || value_id1 != self.consumed_ids.get().1 {
            f((
                &self.dynamics.0.value.borrow(),
                &self.dynamics.1.value.borrow(),
            ));
            self.consumed_ids.set((value_id0, value_id1));
        }
    }
}
```

Variations on implementing `IntoConsumer`:


```rust
trait IntoConsumer<Output> {
    //type Output;
    //fn into_consumer(&self) -> Self::Output;
    fn into_consumer(&self) -> Output;
}

impl<A, B, DA, DB> IntoConsumer<Consumer2<A, B>> for (DA, DB)
where
    A: CommonBound,
    B: CommonBound,
    DA: AsRef<Dynamic<A>>,
    DB: AsRef<Dynamic<B>>,
{
    //type Output = Consumer2<A, B>;
    //fn into_consumer(&self) -> Self::Output {
    fn into_consumer(&self) -> Consumer2<A, B> {
        Consumer2::new((self.0.as_ref().clone(), self.1.as_ref().clone()))
    }
}
```