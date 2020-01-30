use std::error::Error;
use nit_px;
use colored::*;

use headless_chrome::Browser;

fn test(url: &String, browser: &Browser) -> Result<(), Box<dyn Error>> {
    let config = nit_px::config::make_config();

    let slug = url.replace(&config.trusted_url, "");
    println!("{}{}{}", "testing \"".underline(), &slug.underline(), "\"".underline());

    if config.ignored_routes.contains(&slug)
    {
        Err(Box::new(nit_px::SkipError::new()))
    } else {
        let images_identical = nit_px::browser::capture_snapshots(
            &config.trusted_url,
            &config.testing_url,
            &slug,
            &browser,
        )?;

        let pic_name = nit_px::url_utils::get_name_from_slug(&slug);
        nit_px::compare(
            format!("{}/{}_trusted.png", config.screenshot_dir, pic_name),
            format!("{}/{}_testing.png", config.screenshot_dir, pic_name),
            format!("{}/{}_diff.png", config.screenshot_dir, pic_name),
            images_identical,
        )
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let browser = nit_px::browser::make_browser()?;
    let urls = nit_px::url_utils::get_urls()?;
    let config = nit_px::config::make_config();

    // TODO: Collect and output something easier to digest after running through all tests
    let test_results = urls.iter().map(|url| (url, test(url, &browser)));
    let mut passes: Vec<String> = vec![];
    let mut fails: Vec<String> = vec![];
    for (url, diff_result) in test_results {
        let slug = url.replace(&config.trusted_url, "");

        match diff_result {
            Ok(_) => {
                let test_summary = format!("{} \"{}\"\n", "PASS".black().on_green(), slug);
                println!("{}", test_summary);
                passes.push(test_summary);
            }
            Err(e) => {
                let test_summary = format!("{} \"{}\": {:?}\n", "FAIL".black().on_red(), slug, e);
                println!("{}", test_summary);
                fails.push(test_summary);
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
            println!("{}", t);
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
            println!("{}", t);
        }
    }

    Ok(())
}
