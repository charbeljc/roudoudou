use std::sync::Once;
use dotenv::dotenv;
use ngrok2;
use std::env;
use::log::debug;

static START: Once = Once::new();

pub fn setup() {
    START.call_once(|| {
        dotenv().ok();
        debug!("Test Session Setup!!");
        // let tunnel = ngrok::builder()
        //     .http()
        //     .port(1999)
        //     .run().unwrap();
        // let public_url = tunnel.http().unwrap();
        // env::set_var("ODOO_URL", public_url.as_str());
    });
}
