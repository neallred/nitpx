use std::env;
use std::collections::HashSet;
use url::Url;
use directories::ProjectDirs;
use std::fs::File;
use std::io::{BufReader, Read};

use serde;
use serde_json;

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct CliConfig {
    pub config: Option<String>,
    pub noheadless: bool,
    pub screenshots: Option<String>,
    pub testing: Option<String>,
    pub trusted: Option<String>,
    pub threshold: Option<String>,
    pub ignored: Option<String>,
    pub routes: Option<String>,
}

pub fn config_to_env(config: &Config) -> String {
    format!("
export NITPX_HEADLESS=\"{}\"
export NITPX_IGNORED=\"{}\"
export NITPX_ROUTES=\"{}\"
export NITPX_SCREENSHOTS=\"{}\"
export NITPX_TESTING=\"{}\"
export NITPX_THRESHOLD=\"{}\"
export NITPX_TRUSTED=\"{}\"",
        config.headless,
        config.ignored.iter().map(|x| x.clone()).collect::<Vec<String>>().join(","),
        config.routes,
        config.screenshots,
        config.testing,
        config.threshold.to_string(),
        config.trusted,
    )
}

pub fn config_to_flags(config: &Config) -> String {
    format!("--headless {} --ignored {} --routes {} --screenshots {} --testing {} --threshold {} --trusted {}",
        config.headless,
        config.ignored.iter().map(|x| x.clone()).collect::<Vec<String>>().join(","),
        config.routes,
        config.screenshots,
        config.testing,
        config.threshold.to_string(),
        config.trusted,
    )
}

pub fn read_json_file<T: std::fmt::Debug + serde::de::DeserializeOwned + serde::ser::Serialize>(filename: &String) -> Option<T> {
    let file = File::open(filename);
    if file.is_err() {
        println!("Unable to read json config file \"{}\": {:?}\nAll required config values will need to be present in flags or the environment.\n", filename, file);
        return None;
    }
    let mut buffered_reader = BufReader::new(file.unwrap());
    let mut contents = String::new();
    let _number_of_bytes: usize = match buffered_reader.read_to_string(&mut contents) {
        Ok(number_of_bytes) => number_of_bytes,
        Err(_err) => 0,
    };

    let parsed: Result<T, serde_json::error::Error> = serde_json::from_str(&contents);

    if parsed.is_err() {
        println!("Error parsing json config file \"{}\": {:?}\nAll required config values will need to be present in flags or the environment.\n", filename, parsed);
    }

    parsed.ok()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Useful for debugging. Also, some browsers have issues setting large window heights and
    /// return a negative top that explodes rust headless chrome, which expects a non-negative
    /// integer. This provides a workaround.
    pub headless: bool,
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

fn assert_url(url: &String) -> () {
    if let Err(x) = Url::parse(url) {
        println!("Error parsing domain \"{}\": {:?}", url, x);
        std::process::exit(1);
    }
}

fn assert_threshold(threshold: f64) -> () {
    if threshold < 0.0 || threshold > 100.0 {
        println!("Threshold should be between 0 and 100. Received {}", threshold);
        std::process::exit(1);
    }
}

lazy_static! {
    static ref PROJECT_DIRS: ProjectDirs = ProjectDirs::from("red.allthings", "nitpx", "nitpx").unwrap_or_else(|| {
        println!("Unable to locate path to a project directory. Does the home directory exist? Exiting...\n");
        std::process::exit(1);
    });
    static ref CONFIG_FILE_PATH: String = get_config_file_path_internal();
}

fn get_config_file_path_internal() -> String {
    let config_folder_path = (&*PROJECT_DIRS).config_dir().to_str().unwrap_or_else(|| {
        println!("Unable to locate path to a project config directory. Exiting...\n");
        std::process::exit(1);
    });

    let mut config_file_path = String::from(config_folder_path.clone()); 
    config_file_path.push_str("/config.json");
    config_file_path
}

pub fn get_config_file_path() -> String {
    CONFIG_FILE_PATH.clone()
}

pub fn get_config(cli_config: &CliConfig) -> Config {
    let config_file_path = cli_config.config.clone().unwrap_or_else(|| get_config_file_path());
    let file_config: Option<Config> = read_json_file(&config_file_path);

    println!("{:?}", cli_config);

    let headless: bool = if cli_config.noheadless {
        false
    } else {
        env::var("NITPX_HEADLESS")
            .and_then(|x| Ok(x != "false"))
            .unwrap_or_else(|_| {
                match &file_config {
                    Some(file_config) => file_config.headless,
                    None => {
                        println!("Could not find headless flag. Defaulting to true");
                        true
                    }
                }
            })
    };

    let trusted = cli_config.trusted
        .clone()
        .unwrap_or_else(|| {
            env::var("NITPX_TRUSTED").unwrap_or_else(|_| {
                match &file_config {
                    Some(file_config) => file_config.trusted.clone(),
                    None => {
                        println!("Could not find trusted domain in environment, command line, or config file. Exiting.");
                        std::process::exit(1);
                    }
                }
            })
        });
    assert_url(&trusted);

    let testing = cli_config.testing
        .clone()
        .unwrap_or_else(|| {
            env::var("NITPX_TESTING").unwrap_or_else(|_| {
                match &file_config {
                    Some(file_config) => file_config.testing.clone(),
                    None => {
                        println!("Could not find testing domain in environment, command line, or config file. Exiting.");
                        std::process::exit(1);
                    }
                }
            })
        });
    assert_url(&testing);

    let screenshots = cli_config.screenshots
        .clone()
        .unwrap_or_else(|| {
            env::var("NITPX_SCREENSHOTS").unwrap_or_else(|_| {
                match &file_config {
                    Some(file_config) => file_config.screenshots.clone(),
                    None => {
                        println!("Could not find screenshots directory in environment, command line, or config file. Exiting.");
                        std::process::exit(1);
                    }
                }
            })
        });

    let ignored: HashSet<String> = cli_config.ignored
        .clone()
        .unwrap_or_else(|| {
            env::var("NITPX_IGNORED").unwrap_or_else(|_| {
                match &file_config {
                    Some(file_config) => file_config.ignored.iter().map(|x| x.clone()).collect::<Vec<String>>().join(","),
                    None => {
                        println!("No ignored routes: Testing all routes.");
                        String::from("")
                    }
                }
            })
        }).split(',').map(|x| x.into()).collect();

    let threshold: f64 = cli_config.threshold
        .clone()
        .unwrap_or_else(|| {
            env::var("NITPX_THRESHOLD").unwrap_or_else(|_| { String::from("") })
        })
        .parse::<f64>()
        .unwrap_or_else(|_| {
            match &file_config {
                Some(file_config) => file_config.threshold.clone(),
                None => {
                    println!("Bad threshold config value. Defaulting to 0.");
                    0.0
                }
            }
        });

    assert_threshold(threshold);

    let routes = cli_config.routes
        .clone()
        .unwrap_or_else(|| {
            env::var("NITPX_ROUTES").unwrap_or_else(|_| {
                match &file_config {
                    Some(file_config) => file_config.routes.clone(),
                    None => {
                        println!("No routes given in in environment, command line, or config file. Defaulting to collect routes to test from trusted domain sitemap.");
                        String::from("sitemap")
                    }
                }
            })
        });

    Config {
        headless,
        routes,
        ignored,
        screenshots,
        testing,
        threshold,
        trusted,
    }
}
