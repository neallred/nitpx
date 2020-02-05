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
use colored::*;

use std::time::Duration;
use std::thread::sleep;

fn calculate_render_sleep(px_in_capture: &u32) -> Duration {
    // Very large pages need more time to render.
    // This seems like a reasonable default scale
    //
    // a 2000 px by 20000 px page would be 40 million px,
    // and this function would calculate a delay of 4 seconds.

    // rounding occurs, so don't use from_seconds or too much precision is lost

    // 30 ms delay is an arbitrary amount to account for device/network slowness
    // TODO: Make these values configurable to users
    Duration::from_micros((30_000 + px_in_capture / 10).into())
}

pub fn make_browser(config: &crate::config::Config) -> Result<Browser, Box<dyn Error>> {
    let browser_options = LaunchOptionsBuilder::default()
        .headless(config.headless)
        .window_size(Some((1600, 1000)))
        .idle_browser_timeout(Duration::new(40, 0))
        .build()?;
    let browser = Browser::new(browser_options)?;
    let _tab = browser.wait_for_initial_tab()?;
    Ok(browser)
}

pub fn capture_snapshots(
    config: &crate::config::Config,
    slug: &String,
) -> Result<bool, Box<dyn Error>> {
    let trusted_domain = &config.trusted;
    let testing_domain = &config.testing;

    // We make a new browser per test, because a given browser stops responding
    // after about 20-23 tests.
    // Maybe there's a cleanup bug and tabs are not properly closed?
    // It would be more efficient to reuse the same browser for the whole suite.

    let browser = make_browser(config)?;
    let pic_name = url_utils::get_name_from_slug(&slug);

    let filepath_trusted = format!("{}/{}_trusted.png", config.screenshots, pic_name);
    let filepath_testing = format!("{}/{}_testing.png", config.screenshots, pic_name);

    println!("{}", "trusted url...".blue().dimmed());
    let tab = browser.new_tab()?;
    tab.set_default_timeout(Duration::from_secs(40));

    tab.navigate_to(&(trusted_domain.clone() + &slug))?
        .wait_until_navigated()?;

    tab.set_bounds(Bounds::Normal {
        left: Some(0),
        top: Some(0),
        width: Some(1600),
        height: None,
    })?;

    let body = tab.wait_for_element("body")?;
    body.call_js_fn("function() { this.style.overflowY = \"scroll\"; }", false)?;
    body.move_mouse_over()?;


    let content_size = tab.wait_for_element("html")?
        .get_box_model()?;
    let viewport = content_size.margin_viewport();


    tab.set_bounds(Bounds::Normal {
        left: None,
        top: None,
        width: None,
        height: Some(content_size.height + 1)
    })?;

    // TODO: Clean up code that insures equal scroll bar presence/ page widths.

    let estimated_render_time = calculate_render_sleep(&(content_size.width * content_size.height));

    println!("wait {:?} for render...", estimated_render_time);
    sleep(estimated_render_time);

    println!("viewport: w: {:?} h: {:?}", viewport.width, viewport.height);
    println!("capturing image...");
    let pic_trusted = tab.capture_screenshot(ScreenshotFormat::PNG, Some(viewport ), true)?;
    let pic_trusted_len = pic_trusted.len();

    println!("pic length: {:?}", pic_trusted_len);
    let mut out_trusted = File::create(filepath_trusted)?;
    out_trusted.write(&pic_trusted)?;









    println!("{}", "testing url...".blue().dimmed());
    tab.navigate_to(&(testing_domain.clone() + &slug))?
        .wait_until_navigated()?;

    let body = tab.wait_for_element("body")?;
    body.call_js_fn("function() { this.style.overflowY = \"scroll\"; }", false)?;
    // move mouse to similar place on both of them,
    // so that the trusted tab is forced to have a mouse hover,
    // so that testing doesn't get focused elements that trusted url doesn't get
    body.move_mouse_over()?;

    println!("setting bounds");
    tab.set_bounds(Bounds::Normal {
        left: None,
        top: Some(0),
        width: Some(1600),
        height: None,
    })?;

    println!("getting html box model");
    let content_size = tab.wait_for_element("html")?
        .get_box_model()?;
    let viewport = content_size.margin_viewport();
    println!("viewport: w: {:?} h: {:?}", viewport.width, viewport.height);
    // To really be sure and get true full snapshots this should probably set_bounds again here.
    // but the getWindowForTarget can return negative top when you grow the top a lot, and if you do
    // that the CurrentBounds type asplodes in parsing. So don't do that...

    // e.g.
    // method_call MethodCall { method_name: "Browser.getWindowForTarget", id: 12, params: GetWindowForTarget { target_id: "D27B318B1E2A43F837C7D05370C72ACD" } }
    // message text for browser:
    // "{\"method\":\"Browser.getWindowForTarget\",\"id\":12,\"params\":{\"targetId\":\"D27B318B1E2A43F837C7D05370C72ACD\"}}"
    // thing_to_parse: Object({"bounds": Object({"height": Number(5201), "left": Number(0), "top": Number(-4001), "width": Number(1600), "windowState": String("normal")}), "windowId": Number(1)})

    println!("waiting {:?} for render...", estimated_render_time);
    sleep(estimated_render_time);
    println!("capturing image...");
    let pic_testing = tab.capture_screenshot(ScreenshotFormat::PNG, Some(viewport), true)?;
    let pic_testing_len = pic_testing.len();

    println!("pic length: {:?}", pic_testing_len);
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
