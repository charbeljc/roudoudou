use std::env;
pub use url::{ParseError, Url};

pub fn odoo_url_from_env() -> Result<Url, ParseError> {
    match env::var("ODOO_URL") {
        Ok(url) => Url::parse(&url),
        Err(_) => {
            let odoo_host = match env::var("ODOO_HOST") {
                Ok(val) => val,
                Err(_) => "localhost".to_owned(),
            };
            let odoo_port = match env::var("ODOO_PORT") {
                Ok(val) => val.parse().map_err(|_e| ParseError::InvalidPort),
                Err(_) => Ok(8069),
            }?;
            let scheme = if odoo_port == 443 { "https" } else { "http" };
            Url::parse(
                &format!(
                    "{}:{}{}",
                    scheme,
                    odoo_host,
                    match odoo_port {
                        443 | 80 => "".to_owned(),
                        port => format!(":{}", port),
                    }
                )
            )
        }
    }
}

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod tests {
    use crate::odoo_url_from_env;
    use url::Url;
    use std::env;
    use std::sync::{Arc, Mutex};
    
    lazy_static! {
        static ref LOCK: Arc<Mutex<u32>> = {
            Arc::new(Mutex::new(0))
        };
    }

    #[test]
    fn test_odoo_default_url() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::remove_var("ODOO_URL");
        env::remove_var("ODOO_HOST");
        env::remove_var("ODOO_PORT");

        assert_eq!(odoo_url_from_env(), Url::parse("http://localhost:8069"));
    }
    #[test]
    fn test_odoo_url_precedence() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();
        
        env::set_var("ODOO_URL", "http://example.com");
        env::set_var("ODOO_HOST", "localhost");
        env::set_var("ODOO_PORT", "8069");

        assert_eq!(odoo_url_from_env(), Url::parse("http://example.com"));
    }

    #[test]
    fn test_odoo_host_port() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::remove_var("ODOO_URL");
        env::set_var("ODOO_HOST", "example.com");
        env::set_var("ODOO_PORT", "8068");

        assert_eq!(odoo_url_from_env(), Url::parse("http://example.com:8068"));
    }

    #[test]
    fn test_port_80() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::remove_var("ODOO_URL");
        env::set_var("ODOO_HOST", "example.com");
        env::set_var("ODOO_PORT", "80");

        assert_eq!(odoo_url_from_env(), Url::parse("http://example.com"));        
    }

    #[test]
    fn test_port_443() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::remove_var("ODOO_URL");
        env::set_var("ODOO_HOST", "example.com");
        env::set_var("ODOO_PORT", "443");

        assert_eq!(odoo_url_from_env(), Url::parse("https://example.com"));        
    }

    #[test]
    fn test_invalid_url() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::set_var("ODOO_URL", "http://foooobar:zorgl");

        assert!(odoo_url_from_env().is_err());
    }

    #[test]
    fn test_invalid_port() {
        let lock = Arc::clone(&LOCK);
        let mut _data = lock.lock().unwrap();

        env::remove_var("ODOO_URL");
        env::set_var("ODOO_HOST", "foobar");
        env::set_var("ODOO_PORT", "foobar");

        assert!(odoo_url_from_env().is_err());
    }
}
