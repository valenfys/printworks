# printworks

A command-line tool that converts camera RAW photos (CR2, NEF, ARW, RAF,
RW2, ORF, PEF, DNG, and more) into full-resolution JPEGs.

RAW decoding and the develop pipeline (demosaic, white balance, color
space conversion, tone curve) are handled by
[`rawloader`](https://github.com/pedrocr/rawloader) +
[`imagepipe`](https://github.com/pedrocr/imagepipe) — pure Rust, no
native build toolchain required. EXIF metadata (camera, lens, exposure,
date, GPS) is read from the RAW file and copied into the output JPEG.

## Install / build

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

The binary links against `imagepipe`, which is LGPL-3.0-only.
