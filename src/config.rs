use std::env;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Config {
    // absolute path
    pub screenshot_dir: String,
    pub testing_url: String, 
    pub trusted_url: String, 
    pub threshold: f64,
    // Some pages may have problematic HTML that won't parse
    // or be so large the browser times out.
    // This provides a way to ignore those routes
    pub ignored_routes: HashSet<String>,
    pub routes: String,
}

pub fn make_config() -> Config {
    cached! {
        CONFIG;
        fn inner_make_config() -> Config = {
            let trusted_url = env::var("NIT_PX_TRUSTED").unwrap_or(
                String::from("https://crates.io/crates/headless_chrome/0.8.0/")
            );

            let testing_url = env::var("NIT_PX_TESTING").unwrap_or(
                String::from("https://crates.io/crates/headless_chrome/0.9.0/")
            );

            let screenshot_dir = env::var("NIT_PX_SCREENSHOT_DIR").unwrap_or(
                String::from("/home/username/github/nitpx/screenshots")
            );

            let ignored_routes: HashSet<String> = if let Ok(x) = env::var("NIT_PX_IGNORED_ROUTES") {
                x.split(',').map(|x| x.into()).collect()
            } else {
                HashSet::new()
            };

            Config {
                routes: env::var("NIT_PX_ROUTES").unwrap_or(String::from("sitemap")),
                ignored_routes,
                screenshot_dir,
                testing_url,
                threshold: 0.0,
                trusted_url,
            }
        }
    }

    inner_make_config()
}
