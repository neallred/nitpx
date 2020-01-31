# Nitpx

A tool to test versions of sites against each other for visual changes.

## Explanation

Testing visual regressions is time consuming and error prone. This is a first line of defense tool that inspects pages as they render initially, before user interaction. As diffing images can be very slow and the number of URLs in a site and size of individual pages can be very large, this tool prioritizes diffing speed and discovering what content changed over providing clean, precise diffs.

Pages with nondeterministic content or content that dynamically changes without user action will likely cause false positives.


## Setup
*. Install [Rust][install_rust].
*. Add the following environment variables, configuring values to your use case. The example below setup assumes a bash shell. Note that if `NIT_PX_ROUTES` is set to `"sitemap"`, It will look on the trusted domain for a `/sitemap.xml` and generate routes to test based on that.

```
export NIT_PX_ROUTES="blog,explore,about"
export NIT_PX_IGNORED="huge-route,broken/route"
export NIT_PX_SCREENSHOTS="/path/to/where/you/want/to/store/screenshots"
export NIT_PX_TESTING="https://changed.version-of.site/"
export NIT_PX_TRUSTED="https://trusted.domain.com/"
```

*. Run `cargo run --release` from a command line, from the project root directory.
*. For routes that have diverged, inspect the relevant `..._diff.png` image in the screenshot directory. Differences are marked in orange.

[install_rust]: https://www.rust-lang.org/tools/install
