use std::sync::Once;
use dotenv::dotenv;
use::log::debug;

static START: Once = Once::new();

pub fn setup() {
    START.call_once(|| {
        dotenv().ok();
        debug!("Test Session Setup!!");
    });
}
