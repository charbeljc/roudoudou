pub use serde_json::json;
pub use serde_json::{Map, Number, Value};
use pretty_assertions::assert_eq;


macro_rules! oo_clause {
    ( ($inner:tt $op:tt $right:tt) ) => {
        oo_clause!($inner $op $right)
    };
    ($left:ident == $right:expr) => {
        (stringify!($left), "=", $right)
    };
    ($left:ident != $right:expr) => {
        (stringify!($left), "!=", $right)
    };
    ($left:ident <= $right:expr) => {
        (stringify!($left), "<=", $right)
    };
    ($left:ident >= $right:expr) => {
        (stringify!($left), ">=", $right)
    };
    ($left:ident < $right:expr) => {
        (stringify!($left), "<", $right)
    };
    ($left:ident > $right:expr) => {
        (stringify!($left), ">", $right)
    };
}

macro_rules! oo_test {
    ( $($left:tt $op:tt $right:tt) && * ) => {
        json!([
            $(
                oo_clause!($left $op $right),
            )&&*
        ]);
    };
    ( $($left:tt $op:tt $right:tt) && * ) => {
        json!([
            $(
                oo_clause!($left $op $right),
            )&&*
        ]);
    };
}
// macro_rules! oo_test {
//     ( $($term:tt)&&* ) => {
//         json!([
//             $(
//                 oo_clause! $term,
//             )&&*
//         ])
//     };
//     ( $($left:tt $op:tt $right:tt) && * ) => {
//         json!([
//             $(
//                 oo_clause!($left $op $right),
//             )&&*
//         ]);
//     };

#[test]
fn test_macros() {
    assert_eq!(oo_test!(), json!([]));
    assert_eq!(oo_test!(foo == true), json!([("foo", "=", true)]));
    assert_eq!(oo_test!(foo != 1), json!([("foo", "!=", 1)]));
    assert_eq!(oo_test!(foo <= "hello"), json!([("foo", "<=", "hello")]));
    assert_eq!(oo_test!(foo >= 3.5), json!([("foo", ">=", 3.5)]));
    assert_eq!(oo_test!(foo < true), json!([("foo", "<", true)]));
    assert_eq!(oo_test!(foo > true), json!([("foo", ">", true)]));


    let a = oo_test!(foo == 1 && bar == 2);
    // let a = oo_test!(foo == 1 && (bar == 2));

    // assert_eq!(oo_test!((foo = bar)),
    // json!([("foo", "=", "bar")]);

    assert_eq!(oo_test!(foo < true && bar != "zorgl"),
        json!([("foo", "<", true), ("bar", "!=", "zorgl")]));
}