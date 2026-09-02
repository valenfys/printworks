use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::{Context, Result};
use exif::{Exif, In, Tag, Value};
use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata as ExifMetadata;
use little_exif::rational::uR64;

/// Shooting parameters pulled out of a RAW file's EXIF block, for the `info` command.
#[derive(Debug, Default, serde::Serialize)]
pub struct ShotInfo {
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
    pub exposure_time: Option<String>,
    pub f_number: Option<f64>,
    pub iso: Option<u32>,
    pub focal_length_mm: Option<f64>,
    pub date_time_original: Option<String>,
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
}

pub fn read_exif(path: &Path) -> Option<Exif> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    exif::Reader::new().read_from_container(&mut reader).ok()
}

pub fn shot_info(exif: &Exif) -> ShotInfo {
    ShotInfo {
        lens_make: ascii_string(exif, Tag::LensMake),
        lens_model: ascii_string(exif, Tag::LensModel),
        exposure_time: rational(exif, Tag::ExposureTime)
            .map(|r| format!("{}/{}", r.nominator, r.denominator)),
        f_number: rational(exif, Tag::FNumber).map(|r| r.nominator as f64 / r.denominator as f64),
        iso: unsigned(exif, Tag::PhotographicSensitivity),
        focal_length_mm: rational(exif, Tag::FocalLength)
            .map(|r| r.nominator as f64 / r.denominator as f64),
        date_time_original: ascii_string(exif, Tag::DateTimeOriginal),
        gps_latitude: gps_coord(exif, Tag::GPSLatitude, Tag::GPSLatitudeRef, "S"),
        gps_longitude: gps_coord(exif, Tag::GPSLongitude, Tag::GPSLongitudeRef, "W"),
    }
}

/// Read the EXIF tags of a RAW file and stamp the common ones (camera, lens,
/// exposure, date, GPS) into an in-memory JPEG buffer. A RAW file with no
/// readable EXIF block is left untouched (not an error).
pub fn stamp_jpeg(jpeg: &mut Vec<u8>, raw_path: &Path) -> Result<()> {
    let Some(exif) = read_exif(raw_path) else {
        return Ok(());
    };

    let mut meta = ExifMetadata::new();

    if let Some(v) = ascii_string(&exif, Tag::Make) {
        meta.set_tag(ExifTag::Make(v));
    }
    if let Some(v) = ascii_string(&exif, Tag::Model) {
        meta.set_tag(ExifTag::Model(v));
    }
    if let Some(v) = ascii_string(&exif, Tag::LensMake) {
        meta.set_tag(ExifTag::LensMake(v));
    }
    if let Some(v) = ascii_string(&exif, Tag::LensModel) {
        meta.set_tag(ExifTag::LensModel(v));
    }
    if let Some(v) = ascii_string(&exif, Tag::DateTimeOriginal) {
        meta.set_tag(ExifTag::DateTimeOriginal(v));
    }
    if let Some(v) = rational(&exif, Tag::ExposureTime) {
        meta.set_tag(ExifTag::ExposureTime(vec![v]));
    }
    if let Some(v) = rational(&exif, Tag::FNumber) {
        meta.set_tag(ExifTag::FNumber(vec![v]));
    }
    if let Some(v) = rational(&exif, Tag::FocalLength) {
        meta.set_tag(ExifTag::FocalLength(vec![v]));
    }
    if let Some(v) = unsigned(&exif, Tag::PhotographicSensitivity) {
        meta.set_tag(ExifTag::ISO(vec![v as u16]));
    }
    if let Some(v) = rational_triplet(&exif, Tag::GPSLatitude) {
        meta.set_tag(ExifTag::GPSLatitude(v));
    }
    if let Some(v) = ascii_string(&exif, Tag::GPSLatitudeRef) {
        meta.set_tag(ExifTag::GPSLatitudeRef(v));
    }
    if let Some(v) = rational_triplet(&exif, Tag::GPSLongitude) {
        meta.set_tag(ExifTag::GPSLongitude(v));
    }
    if let Some(v) = ascii_string(&exif, Tag::GPSLongitudeRef) {
        meta.set_tag(ExifTag::GPSLongitudeRef(v));
    }
    meta.set_tag(ExifTag::Software("printworks".to_string()));
    // Pixels are already physically rotated by the develop pipeline, so the
    // output is always "normal" orientation.
    meta.set_tag(ExifTag::Orientation(vec![1]));

    meta.write_to_vec(jpeg, FileExtension::JPEG)
        .context("writing EXIF into JPEG buffer")?;
    Ok(())
}

fn field(exif: &Exif, tag: Tag) -> Option<&Value> {
    exif.get_field(tag, In::PRIMARY).map(|f| &f.value)
}

fn ascii_string(exif: &Exif, tag: Tag) -> Option<String> {
    match field(exif, tag)? {
        Value::Ascii(v) => v.first().map(|b| {
            String::from_utf8_lossy(b)
                .trim_end_matches('\0')
                .to_string()
        }),
        _ => None,
    }
}

fn rational(exif: &Exif, tag: Tag) -> Option<uR64> {
    match field(exif, tag)? {
        Value::Rational(v) => v.first().map(|r| uR64 {
            nominator: r.num,
            denominator: r.denom,
        }),
        _ => None,
    }
}

fn rational_triplet(exif: &Exif, tag: Tag) -> Option<Vec<uR64>> {
    match field(exif, tag)? {
        Value::Rational(v) if v.len() == 3 => Some(
            v.iter()
                .map(|r| uR64 {
                    nominator: r.num,
                    denominator: r.denom,
                })
                .collect(),
        ),
        _ => None,
    }
}

fn unsigned(exif: &Exif, tag: Tag) -> Option<u32> {
    match field(exif, tag)? {
        Value::Short(v) => v.first().map(|&x| x as u32),
        Value::Long(v) => v.first().copied(),
        _ => None,
    }
}

fn gps_coord(exif: &Exif, coord_tag: Tag, ref_tag: Tag, negative_ref: &str) -> Option<f64> {
    let Value::Rational(v) = field(exif, coord_tag)? else {
        return None;
    };
    if v.len() != 3 {
        return None;
    }
    let degrees = v[0].to_f64() + v[1].to_f64() / 60.0 + v[2].to_f64() / 3600.0;
    let sign = match ascii_string(exif, ref_tag) {
        Some(r) if r == negative_ref => -1.0,
        _ => 1.0,
    };
    Some(degrees * sign)
}
