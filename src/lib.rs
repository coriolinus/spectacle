use impl_tuples::impl_tuples;
use std::any::Any;


#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Breadcrumb {
    EnumVariant(&'static str),
    StructField(&'static str),
    Index(String),
    TupleIndex(usize),
    SetMember,
}

pub type Breadcrumbs = im::vector::Vector<Breadcrumb>;

/// Recursively introspect through `Self`.
///
/// Visit each struct field, enum variant, etc. It operates via the
/// [`Any` trait](https://doc.rust-lang.org/std/any/trait.Any.html).
/// Note that because `Any` is only implemented for `'static` items,
/// this trait can only be implemented for owned or `'static` objects which
/// themselves contain no non-`'static` references.
pub trait Spectacle {
    /// Recursively descend through `Self`, visiting it, and then all child items.
    ///
    /// This is a helper function which just calls `introspect_from` with an empty
    /// `Breadcrumbs` trail.
    ///
    /// The visitor receives two type parameters: a trail of breadcrumbs
    /// leading to the current location, and the current item. The breadcrumbs
    /// list is empty for the external call. Parent items are visited before
    /// child items. Child items should be visited in natural order.
    fn introspect<F>(&self, visit: F)
    where
        F: Fn(&Breadcrumbs, &dyn Any),
    {
        self.introspect_from(Breadcrumbs::new(), visit);
    }

    /// Recursively descend through `Self`, visiting it, and then all child items.
    ///
    /// The visitor receives two type parameters: a trail of breadcrumbs
    /// leading to the current location, and the current item. The breadcrumbs
    /// list is empty for the external call. Parent items are visited before
    /// child items. Child items should be visited in natural order.
    ///
    /// When manually implementing this trait, note that it is cheap to clone
    /// the `Breadcrumbs`, so it is idiomatic to clone and push for each call into
    /// the child.
    fn introspect_from<F>(&self, breadcrumbs: Breadcrumbs, visit: F)
    where
        F: Fn(&Breadcrumbs, &dyn Any);
}

macro_rules! impl_primitive {
    ($t:ty) => {
        impl Spectacle for $t {
            fn introspect_from<F>(&self, breadcrumbs: Breadcrumbs, visit: F)
            where
                F: Fn(&Breadcrumbs, &dyn Any),
            {
                visit(&breadcrumbs, self);
            }
        }
    };

    ($t:ty, $($ts:ty),+ $(,)?) => {
        impl_primitive!($t);
        impl_primitive!($($ts),*);
    };
}

impl_primitive!(
    (),
    bool,
    char,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    String,
    &'static str
);

macro_rules! impl_array {
    ($n:expr) => {
        impl<T> Spectacle for [T; $n]
        where
            T: 'static + Spectacle,
        {
            fn introspect_from<F>(&self, breadcrumbs: Breadcrumbs, visit: F)
            where
                F: Fn(&Breadcrumbs, &dyn Any),
            {
                visit(&breadcrumbs, self);
                for (idx, child) in self.iter().enumerate() {
                    let mut breadcrumbs = breadcrumbs.clone();
                    breadcrumbs.push_back(Breadcrumb::Index(format!("{}", idx)));
                    child.introspect_from(breadcrumbs, &visit);
                }
            }
        }
    };

    ($n:expr, $($ns:expr),+ $(,)?) => {
        impl_array!($n);
        impl_array!($($ns),*);
    };
}

impl_array!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32,
);

impl<T0, T1> Spectacle for (T0, T1)
where
    T0: 'static + Spectacle,
    T1: 'static + Spectacle,
{
    fn introspect_from<F>(&self, breadcrumbs: Breadcrumbs, visit: F)
    where
        F: Fn(&Breadcrumbs, &dyn Any),
    {
        visit(&breadcrumbs, self);

        {
            let mut breadcrumbs = breadcrumbs.clone();
            breadcrumbs.push_back(Breadcrumb::TupleIndex(0));
            self.0.introspect_from(breadcrumbs, &visit);
        }

        {
            let mut breadcrumbs = breadcrumbs.clone();
            breadcrumbs.push_back(Breadcrumb::TupleIndex(0));
            self.0.introspect_from(breadcrumbs, &visit);
        }
    }
}

impl_tuples!(32);
