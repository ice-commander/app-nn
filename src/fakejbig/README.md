# fakejbig

A ten-function stand-in for **libjbig**, so the Windows bundle carries no GPL code.

## Why

Ice Commander is MIT OR Apache-2.0, but the Windows installer would otherwise be distributed
under GPL-2.0-or-later because of a library nobody asked for. The DLL identifies itself:

```
JBIG-KIT 2.1 -- (c) 1995-2014 Markus Kuhn -- Licence: GPL
```

It arrives at the end of a chain of **hard, load-time imports**:

```
libgtk-4-1 / libgdk_pixbuf-2.0-0  ->  libtiff-6  ->  libjbig-0
```

The MSYS2 build of gdk-pixbuf compiles TIFF support in rather than shipping it as a loadable
module, so `libtiff-6.dll` is a load-time dependency, and libtiff imports ten symbols from
`libjbig-0.dll`. Neither can be dropped from the bundle — the application would not start.

**We do not use TIFF.** There is no mention of it anywhere in this codebase. The viewer
([`src/viewer-ui/src/content.rs`](../viewer-ui/src/content.rs)) and the file manager
([`src/fm-ui/src/view_factories.rs`](../fm-ui/src/view_factories.rs)) hand bytes to
gdk-pixbuf and let it sniff the format; the ability to display TIFF is a side effect of
using GTK.

JBIG is a compression scheme for bi-level (1-bit) images, meant for faxes and scanned
documents. libtiff calls into it only when a TIFF's Compression tag is JBIG (34661) — one of
its fifteen codecs, and one that effectively nothing produces. Bi-level scans use CCITT
G3/G4; everything else uses LZW, Deflate, PackBits or JPEG.

## Measured, not assumed

The same chain exists on Linux (`libtiff.so.6 → libjbig.so.0`), so it was tested there. An
instrumented stand-in logging every call was preloaded ahead of the real library, and eight
TIFFs were loaded through gdk-pixbuf:

| Compression | control | with the stand-in |
| --- | --- | --- |
| raw, LZW, Deflate, PackBits, JPEG | loads | loads |
| CCITT G3, CCITT G4, CCITT RLE (1-bit) | loads | loads |

All eight decoded identically, and **not one `jbg_*` function was called**. The CCITT cases
are the ones that matter: they are the closest neighbours to JBIG's purpose, so if libtiff
reached this code for bi-level images at all, it would show there.

## What it does

Exports the ten symbols libtiff imports — `jbg_dec_init`, `jbg_dec_in`, `jbg_dec_getsize`,
`jbg_dec_getimage`, `jbg_dec_free`, `jbg_enc_init`, `jbg_enc_out`, `jbg_enc_free`,
`jbg_newlen`, `jbg_strerror` — with the signatures from the public `jbig.h` interface. The
decoder entry points report `JBG_EINVAL`, the encoder emits nothing, and `jbg_strerror`
returns a static explanatory string. libtiff owns the state structs by value and this code
never writes to them.

It contains no JBIG-KIT code. The algorithm was not reimplemented and no JBIG-KIT source was
consulted; only the function signatures are shared, and signatures are not copyrightable.

## Trade-off

A TIFF that really is JBIG-compressed fails to open instead of displaying. Nothing this
application offers depends on that. If it ever matters, the honest fixes in order of
preference are:

1. Rebuild libtiff with `--disable-jbig` — then neither library ships at all.
2. Rebuild gdk-pixbuf without TIFF support, dropping the whole branch.

Both are MSYS2 package rebuilds of `artifacts/gtk4-win32-x64/`.

## Building

**Windows** — called by `builder/build.sh exe` before NSIS runs:

```sh
./src/fakejbig/build-windows.sh          # writes bin/distr/fakejbig/libjbig-0.dll
```

Nothing is overwritten in place. `src/gtk-app/setup.nsi` excludes `libjbig-0.dll` from the
recursive `File /r` over `artifacts/gtk4-win32-x64` and packs ours instead, so the GPL
library never enters the installer archive. The real DLL is also absent from `artifacts/` in
this repository; the exclusion stays as a guard for anyone who repopulates that directory
from MSYS2.

**macOS** — nothing here, and deliberately so. Homebrew's libtiff is not built against
jbigkit, so `dylibbundler` has nothing to collect. That has not been verified on a Mac, so
`builder/build.sh dmg` **fails the build** if a libjbig ever turns up in the bundle rather
than signing it quietly.

**Linux** — nothing to do. The `deb`, `rpm` and `zst` packages take the imaging stack from
the distribution and bundle no libraries at all.

## License

MIT, like the rest of the project.
