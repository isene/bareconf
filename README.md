# bareconf - TUI Config for bare

<img src="img/bareconf.svg" align="left" width="150" height="150">

![Version](https://img.shields.io/badge/version-0.1.0-blue) ![Rust](https://img.shields.io/badge/language-Rust-f74c00) ![License](https://img.shields.io/badge/license-Unlicense-green) ![Platform](https://img.shields.io/badge/platform-Linux-blue) ![Dependencies](https://img.shields.io/badge/dependencies-crust-blue) ![Stay Amazing](https://img.shields.io/badge/Stay-Amazing-important)

TUI configuration tool for the [bare](https://github.com/isene/bare) shell, built on [crust](https://github.com/isene/crust). Provides a visual interface for all bare settings.

<br clear="left"/>

## Features

- 256-color palette picker for all 16 color slots
- Live prompt preview showing changes in real time
- 6 built-in themes (default, solarized, dracula, gruvbox, nord, monokai)
- Browse and manage nicks, gnicks, abbreviations, and bookmarks
- Reads and writes `~/.barerc` directly

## Controls

| Key | Action |
|-----|--------|
| TAB | Cycle between panels |
| Arrows | Navigate within panel |
| Enter | Apply selection |
| s | Save configuration |
| q | Quit |

## Build

```bash
cargo build --release
```

## Part of the [CHasm](https://github.com/isene/chasm) Suite

| Tool | Purpose |
|------|---------|
| [bare](https://github.com/isene/bare)         | Shell (assembly) |
| [glass](https://github.com/isene/glass)       | Terminal emulator (assembly) |
| [tile](https://github.com/isene/tile)         | Window manager + strip status bar (assembly) |
| [show](https://github.com/isene/show)         | File viewer (assembly) |
| [chasm-bits](https://github.com/isene/chasm-bits) | Asmite helpers fed into strip (assembly) |
| **bareconf**                                  | **Config TUI for bare (Rust)** |
| [glassconf](https://github.com/isene/glassconf) | Config TUI for glass (Rust) |
| [tileconf](https://github.com/isene/tileconf) | Config TUI for tile (Rust) |
| [stripconf](https://github.com/isene/stripconf) | Config TUI for strip (Rust) |

## Part of the Fe2O3 Rust Terminal Suite

| Tool | Clones | Type |
|------|--------|------|
| [bare](https://github.com/isene/bare) / [rush](https://github.com/isene/rush) | [rsh](https://github.com/isene/rsh) | Shell |
| **[bareconf](https://github.com/isene/bareconf)** | | **Shell config TUI** |
| [crust](https://github.com/isene/crust) | [rcurses](https://github.com/isene/rcurses) | TUI library |
| [glow](https://github.com/isene/glow) | [termpix](https://github.com/isene/termpix) | Image display |
| [plot](https://github.com/isene/plot) | [termchart](https://github.com/isene/termchart) | Charts |
| [pointer](https://github.com/isene/pointer) | [RTFM](https://github.com/isene/RTFM) | File manager |

## License

[Unlicense](https://unlicense.org/) - public domain.

## Credits

Created by Geir Isene (https://isene.org) with extensive pair-programming with Claude Code.
