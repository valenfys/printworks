use std::path::Path;

use anyhow::{Context, Result};
use imagepipe::{Pipeline, Rotation, SRGBImage};

use crate::cli::{Rotate, WhiteBalance};

pub struct DevelopSettings {
    pub exposure: f32,
    pub white_balance: WhiteBalance,
    pub rotate: Rotate,
}

/// Decode a RAW file and run it through imagepipe's develop pipeline
/// (demosaic, white balance, color space conversion, tone curve),
/// applying the requested CLI overrides on top of the camera defaults.
pub fn develop(path: &Path, settings: &DevelopSettings) -> Result<SRGBImage> {
    let mut pipeline = Pipeline::new_from_file(path)
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("decoding {}", path.display()))?;

    pipeline.ops.basecurve.exposure = settings.exposure;

    if let WhiteBalance::Temp(temp, tint) = settings.white_balance {
        pipeline.ops.tolab.set_temp(temp, tint);
    }

    match settings.rotate {
        Rotate::Auto => {}
        Rotate::None => set_transform(&mut pipeline, Rotation::Normal),
        Rotate::R90 => set_transform(&mut pipeline, Rotation::Rotate90),
        Rotate::R180 => set_transform(&mut pipeline, Rotation::Rotate180),
        Rotate::R270 => set_transform(&mut pipeline, Rotation::Rotate270),
    }

    pipeline
        .output_8bit(None)
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("developing {}", path.display()))
}

fn set_transform(pipeline: &mut Pipeline, rotation: Rotation) {
    pipeline.ops.transform.rotation = rotation;
    pipeline.ops.transform.fliph = false;
    pipeline.ops.transform.flipv = false;
}
