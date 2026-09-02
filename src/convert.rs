use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Context, Result};
use image::ColorType;
use image::codecs::jpeg::JpegEncoder;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use crate::cli::ConvertArgs;
use crate::develop::{self, DevelopSettings};
use crate::discover::{self, DiscoveredFile};
use crate::metadata;

enum Outcome {
    Converted,
    Skipped,
}

pub fn run(args: &ConvertArgs) -> Result<()> {
    let files = discover::discover(&args.inputs, args.recursive)?;
    if files.is_empty() {
        anyhow::bail!("no RAW files found in the given inputs");
    }

    if let Some(jobs) = args.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build_global()
            .ok();
    }

    let settings = DevelopSettings {
        exposure: args.exposure,
        white_balance: args.wb,
        rotate: args.rotate,
    };

    let pb = ProgressBar::new(files.len() as u64);
    if let Ok(style) = ProgressStyle::with_template("{bar:40.cyan/blue} {pos}/{len} {msg}") {
        pb.set_style(style);
    }

    let converted = AtomicUsize::new(0);
    let skipped = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);

    files.par_iter().for_each(|file| {
        pb.set_message(
            file.path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
        );
        match convert_one(file, args, &settings) {
            Ok(Outcome::Converted) => {
                converted.fetch_add(1, Ordering::Relaxed);
            }
            Ok(Outcome::Skipped) => {
                skipped.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                failed.fetch_add(1, Ordering::Relaxed);
                log::error!("{}: {:#}", file.path.display(), e);
            }
        }
        pb.inc(1);
    });

    pb.finish_and_clear();

    let converted = converted.into_inner();
    let skipped = skipped.into_inner();
    let failed = failed.into_inner();
    println!("{converted} converted, {skipped} skipped, {failed} failed");

    if failed > 0 {
        anyhow::bail!("{failed} file(s) failed to convert");
    }
    Ok(())
}

fn convert_one(
    file: &DiscoveredFile,
    args: &ConvertArgs,
    settings: &DevelopSettings,
) -> Result<Outcome> {
    let out_path = output_path(file, args);

    if out_path.exists() && !args.overwrite {
        log::warn!(
            "skipping {} (output exists): {}",
            file.path.display(),
            out_path.display()
        );
        return Ok(Outcome::Skipped);
    }

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating output directory {}", parent.display()))?;
    }

    let srgb = develop::develop(&file.path, settings)?;

    let mut jpeg_bytes = Vec::new();
    {
        let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_bytes, args.quality);
        encoder
            .encode(
                &srgb.data,
                srgb.width as u32,
                srgb.height as u32,
                ColorType::Rgb8.into(),
            )
            .with_context(|| format!("encoding JPEG for {}", file.path.display()))?;
    }

    if let Err(e) = metadata::stamp_jpeg(&mut jpeg_bytes, &file.path) {
        log::warn!(
            "{}: could not copy EXIF metadata: {:#}",
            file.path.display(),
            e
        );
    }

    std::fs::write(&out_path, &jpeg_bytes)
        .with_context(|| format!("writing {}", out_path.display()))?;

    Ok(Outcome::Converted)
}

fn output_path(file: &DiscoveredFile, args: &ConvertArgs) -> PathBuf {
    let ext = args.ext.as_str();
    match &args.output {
        Some(out_dir) => {
            let rel = file.path.strip_prefix(&file.base).unwrap_or(&file.path);
            out_dir.join(rel).with_extension(ext)
        }
        None => file.path.with_extension(ext),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{OutputExt, Rotate, WhiteBalance};

    fn args(output: Option<PathBuf>, ext: OutputExt) -> ConvertArgs {
        ConvertArgs {
            inputs: vec![],
            output,
            recursive: false,
            quality: 90,
            jobs: None,
            overwrite: false,
            exposure: 0.0,
            wb: WhiteBalance::AsShot,
            rotate: Rotate::Auto,
            ext,
        }
    }

    #[test]
    fn defaults_to_alongside_source_when_no_output_dir() {
        let file = DiscoveredFile {
            path: PathBuf::from("/photos/a.CR2"),
            base: PathBuf::from("/photos"),
        };
        let a = args(None, OutputExt::Jpg);
        assert_eq!(output_path(&file, &a), PathBuf::from("/photos/a.jpg"));
    }

    #[test]
    fn mirrors_relative_structure_under_output_dir() {
        let file = DiscoveredFile {
            path: PathBuf::from("/photos/2024/trip/a.CR2"),
            base: PathBuf::from("/photos"),
        };
        let a = args(Some(PathBuf::from("/out")), OutputExt::Jpeg);
        assert_eq!(
            output_path(&file, &a),
            PathBuf::from("/out/2024/trip/a.jpeg")
        );
    }

    #[test]
    fn single_file_input_flattens_into_output_dir() {
        // For a file given directly (not via a directory), base == parent,
        // so the relative path is just the filename.
        let file = DiscoveredFile {
            path: PathBuf::from("/anywhere/a.CR2"),
            base: PathBuf::from("/anywhere"),
        };
        let a = args(Some(PathBuf::from("/out")), OutputExt::Jpg);
        assert_eq!(output_path(&file, &a), PathBuf::from("/out/a.jpg"));
    }
}
