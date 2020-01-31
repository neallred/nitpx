use std::error::Error;
use std::fs;
use std::fmt;
extern crate image;
extern crate ureq;
extern crate md5;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_xml_rs;

#[macro_use]
extern crate cached;

pub mod config;
pub mod browser;
pub mod url_utils;


// TODO: Better error for diffs
#[derive(Debug)]
struct DiffError {
    pct_diff: f64
}

impl DiffError {
    fn new(pct_diff: f64) -> DiffError {
        DiffError{pct_diff: pct_diff}
    }
}

impl fmt::Display for DiffError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"diff showed a {}%", self.pct_diff)
    }
}

impl Error for DiffError {
    fn description(&self) -> &str {
        "still no num in this str"
        // format!("{}", %self.pct_diff)
        // String::from(self.pct_diff.to_string().clone())
    }
}

// TODO: Better error for diffs
#[derive(Debug)]
pub struct SkipError {
}

impl SkipError {
    pub fn new() -> SkipError {
        SkipError{}
    }
}

impl fmt::Display for SkipError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"skip")
    }
}

impl Error for SkipError {
    fn description(&self) -> &str {
        "what goes here?"
        // format!("{}", %self.pct_diff)
        // String::from(self.pct_diff.to_string().clone())
    }
}

use image::{
    GenericImageView,
    Rgba,
    RgbaImage,
    DynamicImage,
    ImageBuffer,
    Pixel,
};

fn diff_rgba3(rgba1 : Rgba<u8>, rgba2 : Rgba<u8>) -> i32 {
    (rgba1[0] as i32 - rgba2[0] as i32).abs()
        + (rgba1[1] as i32 - rgba2[1] as i32).abs()
        + (rgba1[2] as i32 - rgba2[2] as i32).abs()
}

fn get_pct_diff(img_trusted: &DynamicImage, img_testing: &DynamicImage) -> (f64, RgbaImage) {
    let highlight_rgba = Rgba([255,165,0,188]);
    let mut accum = 0;
    let zipper = img_trusted.pixels().zip(img_testing.pixels());

    let (width, height) = img_trusted.dimensions();
    let mut diff_img: RgbaImage = ImageBuffer::new(width, height);

    for (px_trusted, px_testing) in zipper {
        let amount_different = diff_rgba3(px_trusted.2, px_testing.2);
        let diffed_px = if amount_different > 0 {
            let mut px_to_mark = px_testing.2.clone();
            px_to_mark.blend(&highlight_rgba);
            px_to_mark
        } else {
            px_trusted.2
        };
        diff_img.put_pixel(px_trusted.0, px_trusted.1, diffed_px);
        accum += amount_different;
    }
    let pct_diff = accum as f64 * 100.0/ (255.0 * 3.0 * (width * height) as f64);
    println!("Percent difference {}", pct_diff);

    (pct_diff, diff_img)
}

pub fn compare(
    trusted_path: String,
    testing_path: String,
    diff_path: String,
    images_identical: bool,
) -> Result<(), Box<dyn Error>> {
    if images_identical {
        fs::copy(&trusted_path, &diff_path)?;
        Ok(())
    } else {
        let before = image::open(trusted_path)?;
        let after = image::open(testing_path)?;

        let (pct_diff, diff_img) = get_pct_diff(&before, &after);

        diff_img.save(&diff_path)?;
        // TODO: Shouldn't ever be negative, work through why that happens.
        if pct_diff > 0.00000000000001 || pct_diff < -0.00000000000001 {
            Err(Box::new(DiffError::new(pct_diff)))
        } else {
            Ok(())
        }
    }
}
