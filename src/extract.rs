use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs, io};

pub(crate) fn extract_package(package_path: &str) -> Result<(), Box<dyn Error>> {
    let output_path = env::current_dir().unwrap();
    let tmp_dir = tempfile::tempdir()?;
    let tmp_dir_path = tmp_dir.path();

    let tar_command = Command::new("tar")
        .arg("xzf")
        .arg(package_path)
        .arg("-C")
        .arg(tmp_dir_path)
        .output()?;
    if !tar_command.status.success() {
        return Err(format!(
            "Failed to extract package to temp directory: {}",
            String::from_utf8_lossy(&tar_command.stderr)
        )
        .into());
    }

    // Extract each file to final destination
    let entries = fs::read_dir(tmp_dir_path)?;
    for entry in entries {
        let file = entry?;
        let asset_entry_dir = file.path();
        let pathname_path = asset_entry_dir.join("pathname");
        let asset_path = asset_entry_dir.join("asset");

        // Check if file has required info to extract
        if !pathname_path.exists() {
            println!(
                "WARNING: Skipping '{}' as it does not contain a pathname file.",
                file.file_name().to_string_lossy()
            );
            continue;
        }

        // Get pathname from pathname file
        let pathname = fs::read_to_string(&pathname_path)?.trim().to_string();
        let pathname = if cfg!(windows) {
            let re = Regex::new(r#"[<>:"|?*]"#)?;
            re.replace_all(&pathname, "_").into_owned()
        } else {
            pathname
        };

        // Determine output path for asset
        let asset_out_path = output_path.join(&pathname);
        if !asset_out_path.starts_with(&output_path) {
            println!(
                "WARNING: Skipping '{}' as '{}' is outside of '{}'.",
                file.file_name().to_string_lossy(),
                asset_out_path.to_string_lossy(),
                output_path.to_string_lossy()
            );
            continue;
        }

        // Check if the asset file exists before moving
        if !asset_path.exists() {
            println!(
                "WARNING: Skipping '{}' as asset file is missing.",
                file.file_name().to_string_lossy()
            );
            continue;
        }

        // Move asset file to output path
        println!(
            "Extracting '{}' as '{}'",
            file.file_name().to_string_lossy(),
            pathname
        );

        let mut output_file_path = PathBuf::from("./output/");
        output_file_path.push(&pathname);
        move_asset_file(&asset_path, &output_file_path).expect("TODO: panic message");
    }

    Ok(())
}

fn move_asset_file(asset_path: &Path, asset_out_path: &Path) -> Result<(), Box<dyn Error>> {
    // Check if the source asset_path is a directory
    let source_metadata = fs::metadata(asset_path)?;
    if source_metadata.is_dir() {
        // Create the directory for the asset if it doesn't exist
        fs::create_dir_all(asset_out_path)?;

        // Copy the contents of the directory recursively
        let entries = fs::read_dir(asset_path)?;
        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();
            let dest_path = asset_out_path.join(entry.file_name());

            move_asset_file(&entry_path, &dest_path)?;
        }
    } else if source_metadata.is_file() {
        // Create the directory for the asset if it doesn't exist
        fs::create_dir_all(
            asset_out_path
                .parent()
                .ok_or("Failed to get parent directory")?,
        )?;

        // Move the asset file to the output path
        println!("Extracting to {}", asset_out_path.to_string_lossy());
        fs::copy(asset_path, asset_out_path)?;
    } else {
        return Err("Source is not a file or directory".into());
    }

    Ok(())
}
