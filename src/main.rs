use std::error::Error;
use nit_px;

use headless_chrome::Browser;

fn test(url: &String, browser: &Browser) -> Result<(), Box<dyn Error>> {
    let config = nit_px::config::make_config();
    println!("config ignored routes {:?}", config.ignored_routes);

    let slug = url.replace(&config.trusted_url, "");
    println!("testing slug: {:?}", slug);

    // Some pages may have problematic HTML that won't parse
    // or be so large the browser times out.
    // This provides a way to ignore those routes
    if config.ignored_routes.contains(&slug)
    {
        println!("skipped testing \"{}\"", slug);
        Ok(())
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

    // TODO: Collect and output something easier to digest after running through all tests
    let diffs = urls.iter().map(|url| test(url, &browser));
    for diff_result in diffs {
        println!("diff_result {:?}\n", diff_result);
    }

    Ok(())
}
