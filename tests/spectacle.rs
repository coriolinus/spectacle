use spectacle::{Introspect, Spectacle};

macro_rules! expect_visits {
    () => {};
    ($( $expect:expr => $ty:ty ),+ $(,)?) => {
        expect_visits!(@impl $($expect => $ty,)*)
    };
    (@impl $expect:expr => $ty:ty, $( $tail:expr => $tty:ty, )*) => {
        expect_visits!(@enumerate $($tail=>$tty,)*; 0_usize => $expect => $ty);
    };

    (@enumerate
        $expect:expr => $ty:ty, $( $tail:expr => $tty:ty, )*
        ; $e_hd_idx:expr => $e_hd_expect:expr => $e_hd_ty:ty
        $(; $e_idx:expr => $e_expect:expr => $e_ty:ty)*
    ) => {
        expect_visits!(@enumerate
            $($tail=>$tty,)*
            ; 1_usize + $e_hd_idx => $expect => $ty
            ; $e_hd_idx => $e_hd_expect => $e_hd_ty
            $(; $e_idx => $e_expect => $e_ty)*
        )
    };

    // enumeration complete; build match statement
    (@enumerate ; $hidx:expr => $hexpect:expr => $hty:ty $(; $tidx: expr => $texpect:expr => $tty:ty)*) => {
        // TODO: the items are backwards, here. We need one more layer of enumeration to re-reverse
        // this list, so that we visit the top-level item, not the final one.
        let mut idx = 0_usize;
        ($hexpect).introspect(|breadcrumbs, visit| {
            dbg!(breadcrumbs);
            match idx {
                n if n == 0 => {
                    dbg!(n);
                    let got = dbg!(visit.downcast_ref::<$hty>().unwrap());
                    assert_eq!(got, &$hexpect)
                }
                $(
                    n if n == $tidx => {
                        dbg!(n);
                        let got = dbg!(visit.downcast_ref::<$tty>().unwrap());
                        assert_eq!(got, &$texpect)
                    }
                )*
                _ => panic!("visited more items than expected"),
            }
            idx += 1;
        });
        let qty_items = 1_usize + expect_visits!(@count $($tidx,)*);
        assert_eq!(idx, qty_items, "visited fewer items than expected");
    };

    // count of items
    (@count $_head:expr, $($tail:expr,)*) => {
        1_usize + expect_visits!(@count $($tail,)*)
    };
    (@count) => {0_usize};
}
#[derive(Debug, PartialEq, Eq, Spectacle)]
struct SimpleStruct {
    a: usize,
    b: &'static str,
}

const SIMPLE_STRUCT: SimpleStruct = SimpleStruct { a: 123, b: "bar" };

#[test]
fn simple_struct() {
    expect_visits!(SIMPLE_STRUCT => SimpleStruct, 123 => i32, "bar" => &'static str);
}

#[derive(Debug, PartialEq, Eq, Spectacle)]
pub enum UnitEnum {
    A,
    B,
    C,
}

const UNIT_ENUM: UnitEnum = UnitEnum::B;

#[derive(Debug, PartialEq, Eq, Spectacle)]
pub enum MyResult<T, E> {
    Ok(T),
    Err(E),
}

#[derive(Debug, PartialEq, Eq, Spectacle)]
pub struct GenericStruct<T> {
    t: T,
}

#[test]
fn unit_enum() {
    expect_visits!(UNIT_ENUM => UnitEnum);
}
