use std::error::Error;
use std::fs::File;
use std::io::{Write};

use headless_chrome::{
    protocol::browser::Bounds,
    protocol::page::ScreenshotFormat,
    Browser,
    LaunchOptionsBuilder,
};
use md5;

use super::url_utils;

use std::time::Duration;
use std::thread::sleep;

fn calculate_render_sleep(px_in_capture: &u32) -> Duration {
    // Very large pages need more time to render.
    // This seems like a reasonable default scale
    //
    // a 2000 px by 20000 px page would be 40 million px,
    // and this function would calculate a delay of 4 seconds.

    // rounding occurs, so don't use from_seconds or too much precision is lost
    Duration::from_micros((px_in_capture / 10).into())
}

pub fn make_browser() -> Result<Browser, Box<dyn Error>> {
    let browser_options = LaunchOptionsBuilder::default()
        .headless(true)
        .window_size(Some((1600, 1000)))
        .idle_browser_timeout(Duration::new(120, 0))
        .build()?;
    let browser = Browser::new(browser_options)?;
    let _tab = browser.wait_for_initial_tab()?;
    Ok(browser)
}

pub fn capture_snapshots(
    trusted_domain: &String,
    testing_domain: &String,
    slug: &String,
    browser: &Browser,
) -> Result<bool, Box<dyn Error>> {
    let pic_name = url_utils::get_name_from_slug(&slug);

    let filepath_trusted = format!("screenshots/{}_trusted.png", pic_name);
    let filepath_testing = format!("screenshots/{}_testing.png", pic_name);

    let tab = browser.new_tab()?;
    tab.set_default_timeout(Duration::from_secs(120));

    let initial_bounds = tab.get_bounds()?;
    println!("initial_bounds {:?}", initial_bounds);

    tab.navigate_to(&(trusted_domain.clone() + &slug))?
        .wait_until_navigated()?;


    let content_size = tab.wait_for_element("html")?
        .get_box_model()?;
    let viewport = content_size.margin_viewport();
    // TODO: Scroll bar presence is unreliable,
    // especially when capturing the testing route
    // that already has a large viewport set.
    // If possible, set scrollbar to always show.

    let scrollbar_offset = if content_size.height > initial_bounds.height {
        0
    } else {
        15
    };

    let estimated_render_time = calculate_render_sleep(&(content_size.width * content_size.height));

    println!("waiting {:?} for testing render...", estimated_render_time);
    sleep(estimated_render_time);

    tab.set_bounds(Bounds::Normal {
        left: Some(0),
        top: Some(0),
        width: Some(content_size.width + scrollbar_offset), // scroll bar
        height: Some(content_size.height)
    })?;
    println!("trusted viewport:\n {:?}", viewport);
    println!("capturing trusted image...");
    let pic_trusted = tab.capture_screenshot(ScreenshotFormat::PNG, Some(viewport ), true)?;
    let pic_trusted_len = pic_trusted.len();

    println!("trusted pic_trusted length:\n {:?}", pic_trusted_len);
    let mut out_trusted = File::create(filepath_trusted)?;
    out_trusted.write(&pic_trusted)?;








    tab.navigate_to(&(testing_domain.clone() + &slug))?
        .wait_until_navigated()?;

    let content_size = tab.wait_for_element("html")?
        .get_box_model()?;
    println!("got testing box model...");
    let viewport = content_size.margin_viewport();
    println!("testing viewport:\n {:?}", viewport);
    // To really be sure and get true full snapshots this should probably set_bounds again here.
    // but the getWindowForTarget can return negative top when you grow the top a lot, and if you do
    // that the CurrentBounds type asplodes in parsing. So don't do that...

    // e.g.
    // method_call MethodCall { method_name: "Browser.getWindowForTarget", id: 12, params: GetWindowForTarget { target_id: "D27B318B1E2A43F837C7D05370C72ACD" } }
    // message text for browser:
    // "{\"method\":\"Browser.getWindowForTarget\",\"id\":12,\"params\":{\"targetId\":\"D27B318B1E2A43F837C7D05370C72ACD\"}}"
    // thing_to_parse: Object({"bounds": Object({"height": Number(5201), "left": Number(0), "top": Number(-4001), "width": Number(1600), "windowState": String("normal")}), "windowId": Number(1)})

    println!("waiting {:?} for testing render...", estimated_render_time);
    sleep(estimated_render_time);
    println!("capturing testing image...");
    let pic_testing = tab.capture_screenshot(ScreenshotFormat::PNG, Some(viewport), true)?;
    let pic_testing_len = pic_testing.len();

    println!("testing pic_testing length:\n {:?}", pic_testing_len);
    let mut out_testing = File::create(filepath_testing)?;
    out_testing.write(&pic_testing)?;


    let images_are_same = pic_trusted_len == pic_testing_len &&
        md5::compute(pic_trusted) == md5::compute(pic_testing);

    Ok(images_are_same)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sleeps_one_second_per_ten_million_px() {
        assert_eq!(calculate_render_sleep(&10_000_000), Duration::from_secs(1));
    }
}
