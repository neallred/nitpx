use std::error::Error;
use nitpx;
use colored::*;

fn test(url: &String) -> Result<(), Box<dyn Error>> {
    let config = nitpx::config::make_config();

    let slug = url.replace(&config.trusted_url, "");
    println!("{}{}{}", "testing \"".underline(), &slug.underline(), "\"".underline());

    if config.ignored_routes.contains(&slug)
    {
        Err(Box::new(nitpx::SkipError::new()))
    } else {
        let images_identical = nitpx::browser::capture_snapshots(
            &config.trusted_url,
            &config.testing_url,
            &slug,
        )?;

        let pic_name = nitpx::url_utils::get_name_from_slug(&slug);
        nitpx::compare(
            format!("{}/{}_trusted.png", config.screenshot_dir, pic_name),
            format!("{}/{}_testing.png", config.screenshot_dir, pic_name),
            format!("{}/{}_diff.png", config.screenshot_dir, pic_name),
            images_identical,
        )
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let urls = nitpx::url_utils::get_urls()?;
    let config = nitpx::config::make_config();

    // TODO: Collect and output something easier to digest after running through all tests
    let test_results = urls.iter().map(|url| (url, test(url)));
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
        print!(
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
        print!(
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
