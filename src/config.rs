use std::env;
use std::collections::HashSet;
use url::Url;


use std::sync::Once;
static BLAH: String = String::from("");
static mut CONFIG: Config = Config {
    ignored: HashSet::new(),
    routes: String::from(""),
    screenshots: String::from(""),
    threshold: 0.0,
    testing: String::from(""),
    trusted: String::from(""),
};
static INIT: Once = Once::new();

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct CliConfig {
    pub config: Option<String>,
    pub screenshots: Option<String>,
    pub testing: Option<String>,
    pub trusted: Option<String>,
    pub threshold: Option<String>,
    pub ignored: Option<String>,
    pub routes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Some pages may have problematic HTML that won't parse
    /// or be so large the browser times out.
    /// This provides a way to ignore those routes.
    /// If a file is in `routes` and in `ignored`, `ignored` wins.
    pub ignored: HashSet<String>,
    /// Strategy for finding routes to test.
    /// `"sitemap"` is a special value meaning use the trusted domain's sitemap.xml.
    /// All other values are parsed as a comma separated list of slugs.
    pub routes: String,
    /// Absolute path to screenshot storage directoroy.
    /// Can use repo root's screenshots dir, which is included for that purpose.
    ///
    /// E.g. `/home/user/github/nitpx/screenshots`
    pub screenshots: String,
    /// Float from 0 to 100 for how much percentage difference to allow between versions before
    /// test is considered to have failed.
    /// Defaults to 0 (no differences allowed).
    pub threshold: f64,
    /// Test version of root URL of the website.
    pub testing: String,
    /// Trusted/production version of root URL of the website.
    pub trusted: String,
}

/// Gets or makes cached config. It only read's the environment once.
/// cli is ignored on subsequent reads.
pub fn make_config(cli: Option<CliConfig>) -> Config {
    cached! {
        CONFIG;
        fn inner_make_config(cli_config: Option<CliConfig>) -> Config = {
            let trusted = cli_config
                .and_then(|cli| cli.trusted)
                .unwrap_or_else(|| {
                    env::var("NIT_PX_TRUSTED").unwrap_or_else(|_| {
                        let default_trusted = String::from("https://crates.io/crates/headless_chrome/0.8.0/");
                        println!("No trusted domain URL found in environment, command line, or config file. Defaulting to garbage value {}", default_trusted);
                        default_trusted
                    })
                });
            match Url::parse(&trusted) {
                Ok(x) => {
                    println!("parsed url fine: {}", &trusted)
                },
                Err(_) => panic!("bad url!"),
            }

            let testing = env::var("NIT_PX_TESTING").unwrap_or(
                String::from("https://crates.io/crates/headless_chrome/0.9.0/")
            );

            let screenshots = env::var("NIT_PX_SCREENSHOTS").unwrap_or(
                String::from("/home/username/github/nitpx/screenshots")
            );

            let ignored: HashSet<String> = if let Ok(x) = env::var("NIT_PX_IGNORED") {
                x.split(',').map(|x| x.into()).collect()
            } else {
                HashSet::new()
            };

            let threshold: f64 = if let Ok(x) = env::var("NIT_PX_THRESHOLD") {
                x.parse::<f64>().unwrap_or_else(|_| {
                    println!("Bad value received for NIT_PX_THRESHOLD. Expected a stringified float, but received {}", x);
                    0.0
                })
            } else {
                0.0
            };

            Config {
                routes: env::var("NIT_PX_ROUTES").unwrap_or(String::from("sitemap")),
                ignored,
                screenshots,
                testing,
                threshold,
                trusted,
            }
        }
    }

    inner_make_config(cli)
}
