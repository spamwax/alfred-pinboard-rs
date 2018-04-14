use failure::{err_msg, Error};
use std::process::Command;

#[derive(Debug)]
pub struct BrowserActiveTabInfo {
    pub url: String,
    pub title: String,
}

const OSASCRIPT_OUTPUT_SPECIAL_SEPERATOR: &str = " fd850fc2e63511e79f720023dfdf24ec ";

pub fn get() -> Result<BrowserActiveTabInfo, Error> {
    debug!("Starting in browser_info::get");
    let output = Command::new("osascript")
        .arg("-s")
        .arg("so")
        .arg("get-current-url.applescript")
        .output()
        .map_err(|e| err_msg(format!("{:?}: osascript", e)))?;
    if !output.status.success() {
        return Err(err_msg(format!("osascript error: code {}", output.status)));
    }
    // Get output of above command
    let osascript_result = String::from_utf8(output.stdout)?;

    // Extract theURL and theTitle from output (assumed they are separated
    // by ' fd850fc2e63511e79f720023dfdf24ec ' (note spaces))
    let trim_chars: &[_] = &['{', '}', '\n'];
    let tab_info: Vec<&str> = osascript_result
        .trim_matches(trim_chars)
        .split(OSASCRIPT_OUTPUT_SPECIAL_SEPERATOR)
        .map(|s| s.trim().trim_matches('"').trim())
        .collect();
    assert_eq!(2, tab_info.len());

    // If theTitle is missing use theURL for title as well.
    match (tab_info[0].is_empty(), tab_info[1].is_empty()) {
        (true, _) => Err(err_msg("Cannot get browser's URL")),
        (_, true) => Ok(BrowserActiveTabInfo {
            url: tab_info[0].to_string(),
            title: tab_info[0].to_string(),
        }),
        _ => Ok(BrowserActiveTabInfo {
            url: tab_info[0].to_string(),
            title: tab_info[1].to_string(),
        }),
    }
}
