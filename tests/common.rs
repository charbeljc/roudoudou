use std::sync::Once;
use dotenv::dotenv;
static START: Once = Once::new();

pub fn setup() {
    START.call_once(|| {
        dotenv().ok();
        eprintln!("Test Session Setup!!");
    });
}
