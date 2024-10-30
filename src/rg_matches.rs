use anyhow::{Context, Result};
use serde::Deserialize;
use std::io::{self, BufRead};

#[derive(Debug, Deserialize)]
pub struct RgMatch {
    pub data: Option<MatchData>,
}

#[derive(Debug, Deserialize)]
pub struct MatchData {
    pub path: PathInfo,
    pub lines: LineInfo,
    pub line_number: usize, // Capture the line number of the match
}

#[derive(Debug, Deserialize)]
pub struct PathInfo {
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct LineInfo {
    pub text: String,
}

// Function to read ripgrep output from stdin
pub fn get_rg_matches() -> Result<Vec<RgMatch>> {
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
