# Film Rolls EXIF tagger

[![Github](https://img.shields.io/badge/github-urdh/filmrolls--rs-8da0cb?style=flat-square&labelColor=555555&logo=github)][github]
[![Crate](https://img.shields.io/crates/v/filmrolls?style=flat-square&color=fc8d62&logo=rust)][release]
[![License](https://img.shields.io/crates/l/filmrolls?style=flat-square)](LICENSE.md)
[![Workflow](https://img.shields.io/github/actions/workflow/status/urdh/filmrolls-rs/push.yml?style=flat-square)][build]
[![Coverage](https://img.shields.io/codecov/c/gh/urdh/filmrolls-rs?style=flat-square)][codecov]

This is a utility designed to read the XML files used by the [Film Rolls iOS app][film-rolls]
(and JSON data exported from the [Lightme Logbook iOS app][lightme]), to enable batch EXIF tagging
of scanned negatives in TIFF format based on the information in these XML/JSON files.
It is essentially a Rust rewrite of the [filmrolls gem][gem], with added functionality.

The utility is released under the [ISC license](LICENSE.md), and the [changelog](CHANGELOG.md)
provides a list of releases and their contents.

## Usage

At the moment, the utility supports reading and displaying Film Rolls XML and Lightme JSON data.
Using the `list-rolls` and `list-frames` sub-commands, you can explore the data to get a brief
summary of the film rolls present:

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

```

After verifying the metadata for a specific roll, you update the original images with EXIF and XMP
data using the `tag` sub-command (here using the dry-run flag to *not* actually perform the update):

```console
$ filmrolls tag --dry-run -r tests/data/filmrolls.xml -i A0012 tests/data/20160513-A0012+001.tiff
──────────────────────────────────────────────────────────────────
 Roll    Date                  Path
══════════════════════════════════════════════════════════════════
 A0012   2016-05-13 14:12:40   tests/data/20160513-A0012+001.tiff
──────────────────────────────────────────────────────────────────

```

Finally, the `apply-metadata` sub-command can be used to tag images with author and licensing
metadata from a TOML file, independently of any XML/JSON film roll data:

```console
$ filmrolls apply-metadata --dry-run -m tests/data/metadata.toml tests/data/20160513-A0012+001.tiff
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
