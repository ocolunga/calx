# calx

Calendar Expanded -- a CLI tool for displaying calendar information with terminal graphics.

```
╭───────── ISO Week ─────────╮  ╭────── February 2026 ──────╮
│                            │  │                           │
│  ╭─────────────┬────────╮  │  │    Mo Tu We Th Fr Sa  Su  │
│  │ Week        │ 6      │  │  │  5                     1  │
│  │ Year        │ 2026   │  │  │  6  2  3  4  5  6  7   8  │
│  │ Day         │ Monday │  │  │  7  9 10 11 12 13 14  15  │
│  │ Day of Year │ 33     │  │  │  8 16 17 18 19 20 21  22  │
│  ╰─────────────┴────────╯  │  │  9 23 24 25 26 27 28      │
│                            │  │                           │
╰────────────────────────────╯  ╰───────────────────────────╯
```

## Features

- ISO and simple week numbering
- Configurable first day of week (Monday through Sunday)
- Current day highlighting
- Week numbers alongside the calendar grid
- Single month, three-month, and full-year views
- Shortcut flags for common configurations (`--simple-sunday`, `--iso-sunday`, `--simple-monday`)
- Custom configuration via `calx show` with `-d` (day) and `-s`/`-i` (style) flags

## Installation

### Homebrew (recommended)

```bash
brew tap ocolunga/calx
brew install calx-cli
```

### pip

```bash
pip install calx-cli
```

### uv

```bash
uv pip install calx-cli
```

## Usage

```bash
# Default view (ISO week, Monday start)
calx

# Three-month view
calx -m

# Full-year view
calx -y

# Sunday-start with simple week numbering
calx --simple-sunday

# Custom: Wednesday start, simple numbering
calx show -d 3 -s

# Custom: Sunday start, ISO-style numbering
calx show -d 7 -i
```

## Development

```bash
uv sync --group dev
uv run calx --help

# Lint and format
uv run ruff check src/
uv run ruff format src/

# Type check
uv run ty check src/
```

## License

MIT
