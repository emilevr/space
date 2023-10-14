use anyhow::bail;
use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use std::{
    fs::{create_dir_all, read_dir, File},
    path::PathBuf,
};
use xtaskops::ops::cmd;
use zip::ZipArchive;

use crate::command::BuildItCommand;

#[derive(Args, Debug)]
pub struct BenchmarkCommandArgs {}

#[derive(Debug)]
pub struct BenchmarkCommand {}

impl BenchmarkCommand {
    pub fn new(_: BenchmarkCommandArgs) -> Self {
        BenchmarkCommand {}
    }
}

impl BuildItCommand for BenchmarkCommand {
    fn run(&mut self) -> anyhow::Result<()> {
        println!("👷 Running benchmark tests ...");

        let temp_dir_path = PathBuf::from("tmp.sample");
        let temp_file_path = PathBuf::from("tmp.sample.zip");

        if temp_dir_path.exists() && read_dir(&temp_dir_path)?.count() > 0 {
            println!(
                "✔️ Sample files and directories found at {}",
                temp_dir_path.display()
            );
            println!("Containing {}", read_dir(&temp_dir_path)?.count());
        } else {
            println!("👷 Creating directory {}", temp_dir_path.display());
            create_dir_all(&temp_dir_path)?;
            println!("✔️ Created successfully.");

            if !temp_file_path.exists() || temp_file_path.metadata()?.len() != 231_784_731 {
                println!("👷 Downloading archive containing sample files and directories ...");
                download_file(
                    "https://github.com/torvalds/linux/archive/refs/tags/v5.12.zip",
                    &temp_file_path,
                )?;
                println!("✔️ Downloaded sample successfully.");
            } else {
                println!("✔️ Found sample zip file at {}", temp_file_path.display());
            }

            println!("👷 Extracting the zip file ...");
            unzip(&temp_file_path, &temp_dir_path)?;
        }

        println!("👷 Running benchmarks ...");
        cmd!("cargo", "bench").run()?;
        println!("✔️ Benchmarks completed successfully.");

        Ok(())
    }
}

pub fn download_file(url: &str, path: &PathBuf) -> anyhow::Result<()> {
    // Create a reqwest Client
    let client = Client::new();

    // Send a GET request to the URL
    let mut response = client.get(url).send()?;

    // Check if the request was successful
    if response.status().is_success() {
        let mut file = File::create(path)?;
        std::io::copy(&mut response, &mut file)?;
    } else {
        bail!("Failed to download the file: {:?}", response.status());
    }

    Ok(())
}

fn unzip(zip_file_path: &PathBuf, extract_dir_path: &PathBuf) -> anyhow::Result<()> {
    let file = File::open(zip_file_path)?;
    let mut archive = ZipArchive::new(file)?;

    let file_count = archive.len();
    let progress = ProgressBar::new(file_count as u64);
    progress.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {pos}/{len} ({eta})")
        .unwrap()
        .progress_chars("█  "));
    progress.set_message(format!("Extracting {}", zip_file_path.display()));

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_path = entry.name();
        let extract_path: PathBuf = PathBuf::from(extract_dir_path).join(entry_path);

        if entry.is_dir() {
            std::fs::create_dir_all(&extract_path)?;
        } else {
            if let Some(parent) = extract_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }

            let mut extract_file = File::create(&extract_path)?;
            std::io::copy(&mut entry, &mut extract_file)?;
        }

        progress.inc(1);
    }

    progress.finish_with_message("✔️ Extracted successfully.");
    Ok(())
}
