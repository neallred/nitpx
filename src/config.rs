use std::env;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Config {
    pub screenshot_dir: String, // absolute path
    pub testing_url: String, 
    pub trusted_url: String, 
    pub threshold: f64,
    pub ignored_routes: HashSet<String>,
    pub from_sitemap: bool,
}

pub fn make_config() -> Config {
    cached! {
        CONFIG;
        fn inner_make_config() -> Config = {
            let trusted_url = if let Ok(x) = env::var("NIT_PX_TRUSTED") {
                x
            } else {
                String::from("https://crates.io/crates/headless_chrome/0.8.0/")
            };

            let testing_url = if let Ok(x) = env::var("NIT_PX_TESTING") {
                x
            } else {
                String::from("https://crates.io/crates/headless_chrome/0.9.0/")
            };

            let screenshot_dir = if let Ok(x) = env::var("NIT_PX_SCREENSHOT_DIR") {
                x
            } else {
                String::from("/home/username/github/nitpx/screenshots")
            };

            let ignored_routes = if let Ok(x) = env::var("NIT_PX_IGNORED_ROUTES") {
                let mut ignored: HashSet<String> = HashSet::new();
                for x in x.split(',') {
                    ignored.insert(x.into());
                }
                ignored
            } else {
                HashSet::new()
            };

            let from_sitemap = if let Ok(x) = env::var("NIT_PX_FROM_SITEMAP") {
                x == "true"
            } else {
                false
            };

            Config {
                from_sitemap,
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
