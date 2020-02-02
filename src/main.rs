extern crate clap;

use std::error::Error;
use nitpx;
use colored::*;
use clap::{App, Arg, ArgMatches};
use serde_json;

fn test(url: &String, config: &nitpx::config::Config) -> Result<(), Box<dyn Error>> {
    let slug = url.replace(&config.trusted, "");
    println!("{}{}{}", "testing \"".underline(), &slug.underline(), "\"".underline());

    if config.ignored.contains(&slug)
    {
        Err(Box::new(nitpx::SkipError::new()))
    } else {
        let images_identical = nitpx::browser::capture_snapshots(
            &config.trusted,
            &config.testing,
            &slug,
        )?;

        let pic_name = nitpx::url_utils::get_name_from_slug(&slug);
        nitpx::compare(
            format!("{}/{}_trusted.png", config.screenshots, pic_name),
            format!("{}/{}_testing.png", config.screenshots, pic_name),
            format!("{}/{}_diff.png", config.screenshots, pic_name),
            images_identical,
        )
    }
}

fn run_tests(config: &nitpx::config::Config) -> Result<(), Box<dyn Error>> {
    let urls = nitpx::url_utils::get_urls(config)?;

    let test_results = urls.iter().map(|url| (url, test(url, config)));
    let mut passes: Vec<String> = vec![];
    let mut fails: Vec<String> = vec![];
    for (url, diff_result) in test_results {
        let slug = url.replace(&config.trusted, "");

        match diff_result {
            Ok(_) => {
                let test_summary = format!("{} \"{}\"\n", "PASS".black().on_green(), slug);
                println!("{}", test_summary);
                passes.push(test_summary);
            }
            Err(e) => {
                let test_summary = format!(
                    "{} (See \"{}\"): {:?}\n",
                    "FAIL".black().on_red(),
                    format!("{}_diff.png", nitpx::url_utils::get_name_from_slug(&slug)),
                    e
                );

                println!("{}", test_summary);fails.push(test_summary);
            }
        }
    }

    if passes.len() > 0 {
        println!(
            "{}{}{}",
            "Summary of passing tests: (".underline().green().dimmed(),
            (&passes.len().to_string()).underline().green().dimmed(),
            ")\n".underline().green().dimmed(),
        );
        for t in passes {
            print!("{}", t);
        }
    }
    if fails.len() > 0 {
        println!(
            "{}{}{}",
            "Summary of failing tests: (".underline().red().dimmed(),
            (&fails.len().to_string()).underline().red().dimmed(),
            ")\n".underline().red().dimmed(),
        );
        for t in fails {
            print!("{}", t);
        }
    }

    Ok(())
}

fn map_match(matches: &ArgMatches, arg:  &str) -> Option<String> {
    if let Some(x) = matches.value_of(arg) {
        Some(String::from(x))
    } else {
        None
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let config_file_path = nitpx::config::get_config_file_path();

    let cli_result: ArgMatches = App::new("nitpx")
        .version("0.1.0")
        .about("Visual regression testing tool")
        .author("Nathaniel Allred <neallred@gmail.com>")
        .arg(Arg::with_name("config")
            .long("config")
            .takes_value(true)
            .help(&format!("Path to config file. Note that the usual precedence order still takes effect: command line arguments beat environment variables, which still beat config specified in this config file, which beats the program defaults. If this flag is not passed, the operating system specific project dir will be used. On this machine, that is\n{}", config_file_path))
        )
        .arg(Arg::with_name("ignored")
            .long("ignored")
            .takes_value(true)
            .help("Comma separated list of route slugs to ignore")
        )
        .arg(Arg::with_name("routes")
            .long("routes")
            .takes_value(true)
            .help("Use \"sitemap\" to tell nitpx to generate urls to test from the trusted domain's sitemap.xml. Otherwise, pass a comma separate list of route slugs")
        )
        .arg(Arg::with_name("screenshots")
            .long("screenshots")
            .takes_value(true)
            .help("Path to folder in which to store captured screenshots.")
        )
        .arg(Arg::with_name("threshold")
            .long("threshold")
            .takes_value(true)
            .help("Allowed percent difference between testing and trusted urls before test is considered a fail.")
        )
        .arg(Arg::with_name("testing")
            .long("testing")
            .takes_value(true)
            .help("Sets the testing domain")
        )
        .arg(Arg::with_name("trusted")
            .long("trusted")
            .takes_value(true)
            .help("Sets the trusted domain")
        )
        .arg(Arg::with_name("log_config")
            .long("log-config")
            .help("log the computed config to stdout as JSON, then exit. Useful for debugging what config value is actually used and for sharing computed configs.")
        )
        .get_matches();


    let cli_config = nitpx::config::CliConfig {
        config: map_match(&cli_result, "config"),
        ignored: map_match(&cli_result, "ignored"),
        routes: map_match(&cli_result, "routes"),
        screenshots: map_match(&cli_result, "screenshots"),
        threshold: map_match(&cli_result, "threshold"),
        testing: map_match(&cli_result, "testing"),
        trusted: map_match(&cli_result, "trusted"),
    };
    let config = nitpx::config::get_config(&cli_config);

    if cli_result.is_present("log_config") {
        println!("\nConfig as flags:\n");
        println!("{}", nitpx::config::config_to_flags(&config));
        println!("\nConfig as environment variables:");
        println!("{}", nitpx::config::config_to_env(&config));
        println!("\nConfig as JSON:\n");
        println!("{}\n", serde_json::to_string_pretty(&config).expect("Failed to stringify config"));
        std::process::exit(1);
    }

    run_tests(&config)?;

    Ok(())
}
