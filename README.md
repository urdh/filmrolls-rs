# Film Rolls EXIF tagger

[![Github](https://img.shields.io/badge/github-urdh/filmrolls--rs-8da0cb?style=flat-square&labelColor=555555&logo=github)][github]
[![Crate](https://img.shields.io/crates/v/filmrolls?style=flat-square&color=fc8d62&logo=rust)][release]
[![License](https://img.shields.io/github/license/urdh/filmrolls-rs?style=flat-square)](LICENSE.md)
[![Workflow](https://img.shields.io/github/actions/workflow/status/urdh/filmrolls-rs/push.yml?style=flat-square)][build]
[![Coverage](https://img.shields.io/codecov/c/gh/urdh/filmrolls-rs?style=flat-square)][codecov]

This is a utility designed to read the XML files used by the [Film Rolls iOS app][film-rolls],
and enable batch EXIF tagging of scanned negatives in TIFF format based on the information in
these XML files. Support for [Lightme Logbook][lightme] JSON data is also implemented.
It is essentially a Rust rewrite of the [filmrolls gem][gem].

The utility is released under the [ISC license](LICENSE.md).
Eventually there will be some sort of [changelog](CHANGELOG.md) as well.

## Usage

At the moment, the utility only supports reading and displaying Film Rolls XML data. Using the
`list-rolls` and `list-frames` sub-commands, you can explore the data to get a brief summary of
the film rolls present:

```console
$ filmrolls list-rolls -r tests/data/filmrolls.xml -r tests/data/lightme.json
─────────────────────────────────────────────────────────────────────────────────────────────────────────────────
 ID      Frames   Film                         Camera                  Loaded                Unloaded
═════════════════════════════════════════════════════════════════════════════════════════════════════════════════
 A0012   1        Ilford Delta 100 @ 100/21°   Voigtländer Bessa R2M   2016-03-28 15:16:36   2016-05-21 14:13:15
─────────────────────────────────────────────────────────────────────────────────────────────────────────────────
 A0020   1        Ilford SFX 200 @ 200/24°     Voigtländer Bessa R2M   2022-04-30 17:57:00   2022-05-01 15:12:00
─────────────────────────────────────────────────────────────────────────────────────────────────────────────────

$ filmrolls list-frames -r tests/data/filmrolls.xml -i A0012
──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
 #   Lens                             Focal len.   Aperture   Shutter   Comp.   Date                  Location                              Notes
══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════
 1   Color Skopar 35/2.5 Pancake II                ƒ/5.6      1/500 s           2016-05-13 14:12:40   57° 42′ 2.761″ N, 11° 57′ 13.374″ E
──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

$ filmrolls tag -n -r tests/data/filmrolls.xml -i A0012 tests/data/20160513-A0012+001.tiff
──────────────────────────────────────────────────────────────────
 Roll    Date                  Path
══════════════════════════════════════════════════════════════════
 A0012   2016-05-13 14:12:40   tests/data/20160513-A0012+001.tiff
──────────────────────────────────────────────────────────────────

$ filmrolls apply-metadata -n -m tests/data/metadata.toml tests/data/20160513-A0012+001.tiff
──────────────────────────────────────────────────
 Roll   Date   Path
══════════════════════════════════════════════════
               tests/data/20160513-A0012+001.tiff
──────────────────────────────────────────────────

```

[film-rolls]: https://itunes.apple.com/se/app/film-rolls-app-for-film-photographers/id675626559
[lightme]: https://apps.apple.com/us/app/lightme-logbook/id1544518308

[github]: https://github.com/urdh/filmrolls-rs
[release]: https://crates.io/crates/filmrolls
[build]: https://github.com/urdh/filmrolls-rs/actions/workflows/push.yml
[codecov]: https://codecov.io/gh/urdh/filmrolls-rs
[gem]: https://rubygems.org/gems/filmrolls
