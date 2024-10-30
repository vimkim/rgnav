use anyhow::{Context, Result};
use atty::Stream;
use serde::Deserialize;
use std::io::{self, BufRead};

#[derive(Debug, Deserialize)]
pub struct RgMatch {
    pub data: Option<MatchData>,
}

#[derive(Debug, Deserialize)]
pub struct MatchData {
    pub path: PathInfo,
    pub line_number: usize,
}

#[derive(Debug, Deserialize)]
pub struct PathInfo {
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct LineInfo {}

// Function to read ripgrep output from stdin
pub fn get_rg_matches() -> Result<Vec<RgMatch>> {
    // Exit immediately if `stdin` is a terminal (not piped)
    if atty::is(Stream::Stdin) {
        return Err(anyhow::anyhow!(
            "No piped input detected. Please pipe `rg` output to `rgnav`."
        ));
    }

    let stdin = io::stdin();
    let reader = stdin.lock();

    let mut matches = Vec::new();
    for line in reader.lines() {
        let line = line.context("Failed to read line from ripgrep output")?;
        if let Ok(rg_match) = serde_json::from_str::<RgMatch>(&line) {
            matches.push(rg_match);
        }
    }

    Ok(matches)
}
