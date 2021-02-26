pub use serde_json::json;
pub use serde_json::{Map, Number, Value};

macro_rules! oo_test {
    () => {
        json!([])
    };
    ($left:ident == $right:expr) => {
        json!([stringify!($left), '=', $right])
    };
}

#[test]
fn test_macros() {
    let empty_dom = oo_test!();
    let basic_eq = oo_test!(foo == true);

    assert_eq!(empty_dom, json!([]));
    assert_eq!(basic_eq, json![("foo", "=", true)]);
}