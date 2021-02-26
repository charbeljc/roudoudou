use dotenv::dotenv;
use log::debug;
// use ngrok2;
use std::process::Command;
use std::sync::Once;

static START: Once = Once::new();

pub fn setup() {
    START.call_once(|| {
        dotenv().ok();
        env_logger::init();
        debug!("Test Session Setup!!");
        // let tunnel = ngrok::builder()
        //     .http()
        //     .port(1999)
        //     .run().unwrap();
        // let public_url = tunnel.http().unwrap();
        // env::set_var("ODOO_URL", public_url.as_str());
        stellar_restore();
    });
}

pub fn stellar_restore() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("stellar restore")
        .output()
        .expect("failed to restore db snapshot");
    debug!("output: {:?}", output.stdout);
}
