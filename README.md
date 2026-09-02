# printworks

[![Crates.io](https://img.shields.io/crates/v/printworks.svg)](https://crates.io/crates/printworks)

A command-line tool that converts camera RAW photos (CR2, NEF, ARW, RAF,
RW2, ORF, PEF, DNG, and more) into full-resolution JPEGs.

RAW decoding and the develop pipeline (demosaic, white balance, color
space conversion, tone curve) are handled by
[`rawloader`](https://github.com/pedrocr/rawloader) +
[`imagepipe`](https://github.com/pedrocr/imagepipe) — pure Rust, no
native build toolchain required. EXIF metadata (camera, lens, exposure,
date, GPS) is read from the RAW file and copied into the output JPEG.

## Install / build

### From crates.io

```sh
cargo install printworks
```

### From source

```sh
cargo build --release
```

The binary is written to `target/release/printworks`.

## Usage

### Convert RAW files to JPEG

```sh
printworks convert <FILES OR DIRS...> [OPTIONS]
```

```sh
# A single file, written alongside the source
printworks convert IMG_1234.CR2

# A folder, recursively, into a separate output directory
printworks convert ~/Photos/2024-Trip -r -o ~/Photos/2024-Trip/jpg

# With adjustments
printworks convert IMG_1234.CR2 --exposure 0.5 --wb daylight --rotate none
```

| Flag | Default | Description |
| --- | --- | --- |
| `-o, --output <DIR>` | alongside each source file | Directory to write JPEGs into; the input's relative directory structure is mirrored underneath it |
| `-r, --recursive` | off | Recurse into subdirectories of any directory input |
| `--quality <1-100>` | `90` | JPEG quality |
| `-j, --jobs <N>` | number of CPUs | Parallel worker threads |
| `--overwrite` | off | Overwrite an existing output file instead of skipping it |
| `--exposure <EV>` | `0.0` | Exposure compensation in stops, applied on top of the camera default |
| `--wb <VALUE>` | `as-shot` | White balance: `as-shot`, a preset (`daylight`, `cloudy`, `shade`, `tungsten`, `fluorescent`, `flash`), or `<temp_kelvin>:<tint>` (e.g. `5000:1.1`) |
| `--rotate <VALUE>` | `auto` | `auto` uses the orientation recorded in the RAW file, `none` ignores it, or force `90`/`180`/`270` |
| `--ext <jpg\|jpeg>` | `jpg` | Output file extension |

Per-file errors (a corrupt or unsupported RAW) are logged and skipped
rather than aborting the whole batch. The command exits non-zero if any
file failed.

### Inspect a RAW file's metadata

```sh
printworks info <FILES OR DIRS...> [OPTIONS]
```

```sh
printworks info IMG_1234.CR2
printworks info ~/Photos/2024-Trip -r --json
```

Prints camera make/model, dimensions, orientation, and shooting
parameters (lens, exposure time, aperture, ISO, focal length, capture
date, GPS if present) without converting anything. `--json` prints a
machine-readable array instead of text.

### Global flags

`-v` / `-vv` increase log verbosity; `-q, --quiet` suppresses
non-error output. Both apply to either subcommand.

## Examples

[`samples/raw`](samples/raw) has a handful of real RAW files (Canon
CR2, Nikon NEF, Panasonic RW2, Sony ARW — see
[`ATTRIBUTION.md`](samples/raw/ATTRIBUTION.md) for licensing) so you
can try the commands above without hunting down a RAW file first:

```sh
# Inspect all four without converting anything
printworks info samples/raw -r

# Convert them all into samples/raw/jpg
printworks convert samples/raw -r -o samples/raw/jpg

# Convert a single file with adjustments
printworks convert samples/raw/canon-rebelxt.CR2 --exposure 0.5 --wb daylight
```

`printworks info samples/raw -r` prints, among other things:

```
samples/raw/canon-rebelxt.CR2
  camera:      Canon Rebel XT
  dimensions:  3516x2328
  orientation: Rotate270
  exposure:    1/640 s
  aperture:    f/5.6
  iso:         100
  focal len:   10.0 mm
  captured:    2008:05:25 18:32:29
```

## Supported RAW formats

Any format `rawloader` can decode: ARW, CR2, CRW, DCR, DCS, DNG, ERF,
IIQ, KDC, MEF, MOS, MRW, NEF, NRW, ORF, PEF, RAF, RW2, SRW, X3F, and a
few others. Coverage depends on `rawloader`'s internal camera database —
an unsupported camera/mode combination fails with a clear per-file
error rather than a crash.

## Development

```sh
cargo build
cargo test
cargo clippy --all-targets
cargo fmt
```

## License

printworks is licensed under [Apache-2.0](LICENSE).

It statically links against `imagepipe` (LGPL-3.0-only) and `rawloader`
(LGPL-2.1); both permit this as long as their license terms are
satisfied, which does not require printworks itself to be LGPL/GPL. See
[`THIRD-PARTY-LICENSES.md`](THIRD-PARTY-LICENSES.md) for the required
notices and the bundled license texts in [`licenses/`](licenses).
