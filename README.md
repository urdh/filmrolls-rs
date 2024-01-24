# Film Rolls EXIF tagger

[![Github](https://img.shields.io/badge/github-urdh/filmrolls--rs-8da0cb?style=flat-square&labelColor=555555&logo=github)](https://github.com/dtolnay/cargo-expand)
[![Crate](https://img.shields.io/crates/v/filmrolls?style=flat-square&color=fc8d62&logo=rust)][release]
[![License](https://img.shields.io/github/license/urdh/filmrolls-rs?style=flat-square)](LICENSE.md)

This is a utility designed to read the XML files used by the [Film Rolls iOS app][film-rolls],
and enable batch EXIF tagging of scanned negatives in TIFF format based on the information in
these XML files. Support for [Lightme Logbook][lightme] JSON data is also planned.
It is essentially a Rust rewrite of the [filmrolls gem][gem].

The utility is released under the [ISC license](LICENSE.md).
Eventually there will be some sort of [changelog](CHANGELOG.md) as well.

[film-rolls]: https://itunes.apple.com/se/app/film-rolls-app-for-film-photographers/id675626559
[lightme]: https://apps.apple.com/us/app/lightme-logbook/id1544518308

[github]: https://github.com/urdh/filmrolls-rs
[release]: https://crates.io/crates/filmrolls
[gem]: https://rubygems.org/gems/filmrolls
