use std::error::Error;
use ureq;
use serde_xml_rs;

#[derive(Debug, Deserialize, Default)]
struct Loc {
    #[serde(rename = "$value")]
    pub name: String
}

#[derive(Debug, Deserialize)]
struct SitemapUrl {

    #[serde(rename = "loc", default)]
    pub loc: Loc
}

#[derive(Debug, Deserialize)]
struct UrlSet {
    #[serde(rename = "url", default)]
    pub urls: Vec<SitemapUrl>
}

fn url_set_to_urls(urlset: UrlSet) -> Vec<String> {
    let v: Vec<_> = urlset.urls.iter().map(|u| u.loc.name.clone()).collect();
    v
}

fn fetch_sitemap(sitemap_location: &String) -> Result<Vec<String>, Box<dyn Error>> {
    let resp = ureq::get(sitemap_location)
        .call()
        .into_string()?;

    let blah: UrlSet = serde_xml_rs::from_reader(resp.as_bytes())?;
    let sitemap_urls: Vec<String> = url_set_to_urls(blah);
    Ok(sitemap_urls)
}

pub fn get_name_from_slug(slug: &String) -> String {
    if slug == "" || slug == "/" {
        String::from("HOMEPAGE")
    } else {
        let lower = slug.clone().to_lowercase();

        let lower1 = lower.replace(r"\.", "_");
        let lower2 = lower1.replace(r"_", "_");
        let lower3 = lower2.replace(r"/", "_");
        let trimmed = lower3.trim();
        trimmed.to_string()
    }
}

pub fn get_urls() -> Result<Vec<String>, Box<dyn Error>> {
    let the_config = super::config::make_config(None);
    if the_config.routes == "sitemap" {
        println!("Getting urls to test from sitemap...");
        let sitemap_location = String::from(the_config.trusted.clone() + "sitemap.xml");
        fetch_sitemap(&sitemap_location)
    } else {
        println!("Test urls provided by environment");
        // TODO: Add options for reading from a json config.
        Ok(the_config.routes.split(',').map(|x| x.to_string()).collect())
    }
}
