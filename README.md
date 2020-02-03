# Nitpx

A tool to test versions of sites against each other for visual changes.

## Epigraph
```
Ants, beetles, roaches brown,
Silently creep,
Bringing coders frown.

Staring eyes, seeing not,
A noggin feed
Pictures detail fraught.

Did one change, or are both equal?
Will shipping mean a quick sequel?
```

## Explanation

Testing visual regressions is time consuming and error prone. This is a first line of defense tool that inspects pages as they render initially, before user interaction. As diffing images can be very slow and the number of URLs in a site and size of individual pages can be very large, this tool prioritizes diffing speed and discovering what content changed over providing clean, precise diffs.

Pages with nondeterministic content or content that dynamically changes without user action will likely cause false positives.

## Setup
*. Install [Rust][install_rust].
*. Set the configuration values (see below) to match your use case.

Values can be specified in a JSON config file, added to the environment, or passed as flags. Priority order is flags, environment, JSON config file. If a JSON config file is present, it must have all values or the file won't be used and `nitpx` will rely on environment variables and command line flags. Note that if the routes value is `"sitemap"`, It will look on the trusted domain for a `/sitemap.xml` and generate routes to test based on that.

Configuration values as environment variables (assumes a bash shell).

```
export NITPX_ROUTES="blog,explore,about"
export NITPX_IGNORED="huge-route,broken/route"
export NITPX_SCREENSHOTS="/path/to/where/you/want/to/store/screenshots"
export NITPX_TESTING="https://changed.version-of.site/"
export NITPX_TRUSTED="https://trusted.domain.com/"
export NITPX_THRESHOLD="0"
```

Configuration values as a JSON config value. The default path to the JSON config follows the rust crate [`directories`]'s ProjectDirs config dir logic, and the file is named `config.json`. The use can pass an alternate, absolute path to a config file by passing the `--config` command line flag.

```
{
  "ignored": [
    "huge-route",
    "broken/route"
  ],
  "routes": "blog,explore,about",
  "screenshots": "/path/to/where/you/want/to/store/screenshots",
  "threshold": 0.0,
  "testing": "https://changed.version-of.site/",
  "trusted": "https://trusted.domain.com/"
}
```

Config as command line values

```
--ignored huge-route,broken/route --routes blog,explore,about --screenshots /path/to/where/you/want/to/store/screenshots --testing https://changed.version-of.site/ --threshold 0 --trusted https://trusted.domain.com/
```

*. Run `cargo run --release` from a command line, from the project root directory.
*. For routes that have diverged, inspect the relevant `..._diff.png` image in the screenshot directory. Differences are marked in orange.

[install_rust]: https://www.rust-lang.org/tools/install
