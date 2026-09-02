use anyhow::{Context, Result};
use serde::Serialize;

use crate::cli::InfoArgs;
use crate::discover;
use crate::metadata;

#[derive(Serialize)]
struct RawFileInfo {
    path: String,
    make: String,
    model: String,
    width: usize,
    height: usize,
    orientation: String,
    #[serde(flatten)]
    shot: metadata::ShotInfo,
}

pub fn run(args: &InfoArgs) -> Result<()> {
    let files = discover::discover(&args.inputs, args.recursive)?;
    if files.is_empty() {
        anyhow::bail!("no RAW files found in the given inputs");
    }

    let mut infos = Vec::with_capacity(files.len());
    for file in &files {
        match rawloader::decode_file(&file.path) {
            Ok(raw) => {
                let shot = metadata::read_exif(&file.path)
                    .map(|exif| metadata::shot_info(&exif))
                    .unwrap_or_default();
                infos.push(RawFileInfo {
                    path: file.path.display().to_string(),
                    make: raw.clean_make,
                    model: raw.clean_model,
                    width: raw.width,
                    height: raw.height,
                    orientation: format!("{:?}", raw.orientation),
                    shot,
                });
            }
            Err(e) => {
                log::error!("{}: {}", file.path.display(), e);
            }
        }
    }

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&infos).context("serializing info as JSON")?
        );
    } else {
        for info in &infos {
            print_text(info);
        }
    }

    Ok(())
}

fn print_text(info: &RawFileInfo) {
    println!("{}", info.path);
    println!("  camera:      {} {}", info.make, info.model);
    println!("  dimensions:  {}x{}", info.width, info.height);
    println!("  orientation: {}", info.orientation);
    if let Some(lens) = combined_lens(info) {
        println!("  lens:        {lens}");
    }
    if let Some(v) = &info.shot.exposure_time {
        println!("  exposure:    {v} s");
    }
    if let Some(v) = info.shot.f_number {
        println!("  aperture:    f/{v:.1}");
    }
    if let Some(v) = info.shot.iso {
        println!("  iso:         {v}");
    }
    if let Some(v) = info.shot.focal_length_mm {
        println!("  focal len:   {v:.1} mm");
    }
    if let Some(v) = &info.shot.date_time_original {
        println!("  captured:    {v}");
    }
    if let (Some(lat), Some(lon)) = (info.shot.gps_latitude, info.shot.gps_longitude) {
        println!("  gps:         {lat:.6}, {lon:.6}");
    }
    println!();
}

fn combined_lens(info: &RawFileInfo) -> Option<String> {
    match (&info.shot.lens_make, &info.shot.lens_model) {
        (Some(make), Some(model)) => Some(format!("{make} {model}")),
        (None, Some(model)) => Some(model.clone()),
        (Some(make), None) => Some(make.clone()),
        (None, None) => None,
    }
}
