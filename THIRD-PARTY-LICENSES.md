# Third-party licenses

printworks itself is licensed under Apache-2.0 (see [`LICENSE`](LICENSE)).
It statically links two LGPL-licensed libraries, whose terms require the
notices and license copies below.

## imagepipe — LGPL-3.0-only

[`imagepipe`](https://github.com/pedrocr/imagepipe) is used, unmodified,
as a library dependency (the RAW develop pipeline: demosaic, white
balance, color space conversion, tone curve). It is covered by the GNU
Lesser General Public License, version 3.

Full license texts are bundled in this repository:

- [`licenses/LGPL-3.0.txt`](licenses/LGPL-3.0.txt)
- [`licenses/GPL-3.0.txt`](licenses/GPL-3.0.txt) — LGPLv3 incorporates
  GPLv3 by reference, so both are included.

Because printworks is a statically linked Rust binary (no shared-library
mechanism), the LGPLv3 §4(d)(0) route is used to preserve your right to
relink against a modified version of imagepipe: this repository's full
source is published, with the exact imagepipe version pinned in
`Cargo.lock`. To build printworks against a modified imagepipe, point a
`[patch]` entry in `Cargo.toml` at your modified source and rebuild with
`cargo build --release`.

## rawloader — LGPL-2.1

[`rawloader`](https://github.com/pedrocr/rawloader) is used, unmodified,
as a library dependency (RAW file decoding). It is covered by the GNU
Lesser General Public License, version 2.1.

Full license text is bundled in this repository:

- [`licenses/LGPL-2.1.txt`](licenses/LGPL-2.1.txt)

The same relinking mechanism described above (published source, pinned
version, `[patch]` + rebuild) applies to rawloader as well.

## Other dependencies

The remaining direct dependencies are permissively licensed
(MIT / Apache-2.0 / BSD-2-Clause) and impose no bundling requirements
beyond their own copyright notices, which are preserved in each crate's
own repository. See `Cargo.toml` for the full dependency list.
