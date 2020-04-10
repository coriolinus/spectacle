//! ## ![Opt-in Runtime Introspection](../../../media/oiri.png)
//!
//! This crate provides the `Introspect` trait. Types implementing `Introspect`
//! can recursively walk into their structure, calling their visitor function
//! for both themselves and all children. It operates via the [`Any`
//! trait](https://doc.rust-lang.org/std/any/trait.Any.html).
//!
//! Note that because `Any` is only implemented for `'static` items,
//! this trait can only be implemented for owned or `'static` objects which
//! themselves contain no non-`'static` references.
//!
//! It also includes the trail of accessors and selectors describing how to get
//! to the current location from the root object. Given those two things, it is
//! straightforward to find and access the portion of data of interest.

#[cfg(feature = "derive")]
pub use spectacle_derive::Spectacle;

use impl_tuples::impl_tuples;
use std::any::Any;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Breadcrumb {
    Variant(&'static str),
    Field(&'static str),
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
pub trait Introspect {
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
        impl Introspect for $t {
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
        impl<T> Introspect for [T; $n]
        where
            T: 'static + Introspect,
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

impl_tuples!(32);

impl<T> Introspect for Option<T>
where
    T: 'static + Introspect,
{
    fn introspect_from<F>(&self, breadcrumbs: Breadcrumbs, visit: F)
    where
        F: Fn(&Breadcrumbs, &dyn Any),
    {
        visit(&breadcrumbs, self);
        if let Some(t) = self {
            let mut breadcrumbs = breadcrumbs.clone();
            breadcrumbs.push_back(Breadcrumb::Variant("Some"));
            t.introspect_from(breadcrumbs, &visit);
        }
    }
}

impl<T, E> Introspect for Result<T, E>
where
    T: 'static + Introspect,
    E: 'static + Introspect,
{
    fn introspect_from<F>(&self, breadcrumbs: Breadcrumbs, visit: F)
    where
        F: Fn(&Breadcrumbs, &dyn Any),
    {
        visit(&breadcrumbs, self);
        match self {
            Ok(t) => {
                let mut breadcrumbs = breadcrumbs.clone();
                breadcrumbs.push_back(Breadcrumb::Variant("Ok"));
                t.introspect_from(breadcrumbs, &visit);
            }
            Err(e) => {
                let mut breadcrumbs = breadcrumbs.clone();
                breadcrumbs.push_back(Breadcrumb::Variant("Err"));
                e.introspect_from(breadcrumbs, &visit);
            }
        }
    }
}

macro_rules! impl_list {
    ($($t:ident)::+) => {
        #[cfg(feature = "collections")]
        impl<T> Introspect for $($t)::+<T>
        where
            T: 'static + Introspect,
        {
            fn introspect_from<F>(&self, breadcrumbs: Breadcrumbs, visit: F)
            where
                F: Fn(&Breadcrumbs, &dyn Any),
            {
                visit(&breadcrumbs, self);
                for (idx, item) in self.iter().enumerate() {
                    let mut breadcrumbs = breadcrumbs.clone();
                    breadcrumbs.push_back(Breadcrumb::Index(format!("{}", idx)));
                    item.introspect_from(breadcrumbs, &visit);
                }
            }
        }
    };
}

// it's tedious to make macro rules tail recursion work given the inner list,
// so we just call the macro once for each here
impl_list!(Vec);
impl_list!(std::collections::VecDeque);
impl_list!(std::collections::LinkedList);

macro_rules! impl_set {
    ($($t:ident)::+) => {
        #[cfg(feature = "collections")]
        impl<T> Introspect for $($t)::+<T>
        where
            T: 'static + Introspect,
        {
            fn introspect_from<F>(&self, breadcrumbs: Breadcrumbs, visit: F)
            where
                F: Fn(&Breadcrumbs, &dyn Any),
            {
                visit(&breadcrumbs, self);
                for item in self.iter() {
                    let mut breadcrumbs = breadcrumbs.clone();
                    breadcrumbs.push_back(Breadcrumb::SetMember);
                    item.introspect_from(breadcrumbs, &visit);
                }
            }
        }
    };
}

impl_set!(std::collections::HashSet);
impl_set!(std::collections::BTreeSet);
impl_set!(std::collections::BinaryHeap);

macro_rules! impl_map {
    ($($t:ident)::+) => {
        #[cfg(feature = "collections")]
        impl<K, V> Introspect for $($t)::+<K, V>
        where
            K: 'static + std::fmt::Debug,
            V: 'static + Introspect,
        {
            fn introspect_from<F>(&self, breadcrumbs: Breadcrumbs, visit: F)
            where
                F: Fn(&Breadcrumbs, &dyn Any),
            {
                visit(&breadcrumbs, self);
                for (k, v) in self.iter() {
                    let mut breadcrumbs = breadcrumbs.clone();
                    breadcrumbs.push_back(Breadcrumb::Index(format!("{:?}", k)));
                    v.introspect_from(breadcrumbs, &visit);
                }
            }
        }
    };
}

impl_map!(std::collections::HashMap);
impl_map!(std::collections::BTreeMap);

macro_rules! impl_serde_json {
    ($($t:ident)::+) => {
        #[cfg(feature = "serde-json")]
        impl Introspect for $($t)::+ {
            fn introspect_from<F>(&self, breadcrumbs: Breadcrumbs, visit: F)
            where
                F: Fn(&Breadcrumbs, &dyn Any),
            {
                visit(&breadcrumbs, self);
            }
        }
    };

    ($t:ident, $($ts:ident),+ $(,)?) => {
        impl_primitive!($t);
        impl_primitive!($($ts),*);
    };
}

impl_serde_json!(serde_json::Error);
impl_serde_json!(serde_json::Number);

#[cfg(feature = "serde-json")]
impl Introspect for serde_json::Map<String, serde_json::Value> {
    fn introspect_from<F>(&self, breadcrumbs: Breadcrumbs, visit: F)
    where
        F: Fn(&Breadcrumbs, &dyn Any),
    {
        visit(&breadcrumbs, self);
        for (k, v) in self.iter() {
            let mut breadcrumbs = breadcrumbs.clone();
            breadcrumbs.push_back(Breadcrumb::Index(format!("{}", k)));
            v.introspect_from(breadcrumbs, &visit);
        }
    }
}

#[cfg(feature = "serde-json")]
impl Introspect for serde_json::Value {
    fn introspect_from<F>(&self, breadcrumbs: Breadcrumbs, visit: F)
    where
        F: Fn(&Breadcrumbs, &dyn Any),
    {
        visit(&breadcrumbs, self);

        match self {
            serde_json::Value::Bool(x) => {
                let mut breadcrumbs = breadcrumbs.clone();
                breadcrumbs.push_back(Breadcrumb::Variant("Bool"));
                x.introspect_from(breadcrumbs, &visit);
            }
            serde_json::Value::Number(x) => {
                let mut breadcrumbs = breadcrumbs.clone();
                breadcrumbs.push_back(Breadcrumb::Variant("Number"));
                x.introspect_from(breadcrumbs, &visit);
            }
            serde_json::Value::String(x) => {
                let mut breadcrumbs = breadcrumbs.clone();
                breadcrumbs.push_back(Breadcrumb::Variant("String"));
                x.introspect_from(breadcrumbs, &visit);
            }
            serde_json::Value::Array(x) => {
                let mut breadcrumbs = breadcrumbs.clone();
                breadcrumbs.push_back(Breadcrumb::Variant("Array"));
                x.introspect_from(breadcrumbs, &visit);
            }
            serde_json::Value::Object(x) => {
                let mut breadcrumbs = breadcrumbs.clone();
                breadcrumbs.push_back(Breadcrumb::Variant("Object"));
                x.introspect_from(breadcrumbs, &visit);
            }
            _ => {}
        }
    }
}
