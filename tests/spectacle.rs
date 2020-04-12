use spectacle::{Introspect, Spectacle};

/// construct a state machine which verifies that we get the expected visits, of the
/// expected types, in the expected order, and no others.
macro_rules! expect_visits {
    () => {};
    ($( $expect:expr => $ty:ty ),+ $(,)?) => {
        expect_visits!(@impl $($expect => $ty,)*)
    };
    (@impl $expect:expr => $ty:ty, $( $tail:expr => $tty:ty, )*) => {
        expect_visits!(@enumerate $($tail=>$tty,)*; (r 0_usize , $expect , $ty),);
    };

    (@enumerate
           $expect:expr => $ty:ty,
        $( $tail:expr   => $tty:ty, )* ;
           (r $e_hd_idx:expr, $e_hd_expect:expr, $e_hd_ty:ty),
        $( (r $e_idx:expr, $e_expect:expr, $e_ty:ty), )*
    ) => {
        expect_visits!(@enumerate
            $($tail=>$tty,)*;
            (r 1_usize + $e_hd_idx, $expect, $ty),
            (r $e_hd_idx, $e_hd_expect, $e_hd_ty),
            $((r $e_idx, $e_expect, $e_ty),)*
        )
    };

    // enumeration complete; now reverse it
    (@enumerate ;
           (r $hidx:expr, $hexpect:expr, $hty:ty),
        $( (r $tidx:expr, $texpect:expr, $tty:ty), )*
        $( (f $ridx:expr, $rexpect:expr, $rty:ty ), )*
    ) => {
        expect_visits!(@enumerate ;
            $( (r $tidx, $texpect, $tty), )*
               (f $hidx, $hexpect, $hty),
            $( (f $ridx, $rexpect, $rty), )*
        )
    };

    // all elements in forward order
    (@enumerate ;
           (f $hidx:expr, $hexpect:expr, $hty:ty ),
        $( (f $tidx:expr, $texpect:expr, $tty:ty ), )*
    ) => {
        let mut idx = 0_usize;
        ($hexpect).introspect(|_, visit| {
            match idx {
                n if n == 0 => {
                    dbg!(n);
                    let got = dbg!(visit.downcast_ref::<$hty>()).unwrap();
                    assert_eq!(got, &$hexpect);
                }
                $(
                    n if n == $tidx => {
                        dbg!(n);
                        let got = dbg!(visit.downcast_ref::<$tty>()).unwrap();
                        assert_eq!(got, &$texpect);
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
    expect_visits!(SIMPLE_STRUCT => SimpleStruct, 123 => usize, "bar" => &'static str);
}

#[derive(Debug, PartialEq, Eq, Spectacle)]
pub enum UnitEnum {
    A,
    B,
    C,
}

const UNIT_ENUM: UnitEnum = UnitEnum::B;

#[test]
fn unit_enum() {
    expect_visits!(UNIT_ENUM => UnitEnum);
}

#[derive(Debug, PartialEq, Eq, Spectacle)]
pub struct GenericStruct<T> {
    t: T,
}

const GENERIC_SIMPLE: GenericStruct<SimpleStruct> = GenericStruct { t: SIMPLE_STRUCT };

#[test]
fn generic_struct() {
    expect_visits!(
        GENERIC_SIMPLE => GenericStruct<SimpleStruct>,
        SIMPLE_STRUCT => SimpleStruct,
        123 => usize,
        "bar" => &'static str,
    );
}

#[derive(Debug, PartialEq, Eq, Spectacle)]
pub struct Pair<T>(T, T);

const PAIR: Pair<u32> = Pair(123, 456);

#[test]
fn pair() {
    expect_visits!(PAIR => Pair<u32>, 123 => u32, 456 => u32);
}

#[derive(Debug, PartialEq, Eq, Spectacle)]
pub enum StructEnum {
    Variant {
        foo: &'static str,
        bar: &'static [u8],
    },
}

const STRUCT_ENUM: StructEnum = StructEnum::Variant {
    foo: "foo",
    bar: b"bar",
};

#[test]
fn struct_enum() {
    expect_visits!(
        STRUCT_ENUM => StructEnum,
        "foo" => &'static str,
        b"bar" => &'static [u8],
        b'b' => u8,
        b'a' => u8,
        b'r' => u8,
    );
}
