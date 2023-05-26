use anyhow::Result;

use crate::embedded::html;

pub fn run() -> Result<()> {
    html::run_with_html(String::from(include_str!("label_web.html")), None, None)
}
