# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

calx (Calendar Expanded) is a CLI tool for displaying calendar information with terminal graphics. It shows various calendar metrics like ISO week numbers, week numbers (Monday/Sunday start), day of year, biweek numbers, and more.

## Development Commands

```bash
# Install dependencies
uv sync --group dev

# Run the CLI
uv run calx --help

# Type checking (ty)
uv run ty check src/

# Linting and formatting (ruff)
uv run ruff check src/
uv run ruff format src/
```

## Architecture

Single-module CLI application using Typer (CLI framework) and Rich (terminal output):

- **src/calx/cli.py** - Main entry point, contains all logic
  - Helper functions for calendar calculations (`get_first_week_of_month`, `get_biweek_number`, `get_week_info`)
  - `display_calendar_info()` renders output using Rich tables
  - `show()` command with boolean flags for each calendar metric

## Key Dependencies

- `typer` - CLI framework (built on Click)
- `rich` - Terminal formatting and tables
- `ruff` - Linting/formatting (dev)
- `ty` - Type checking (dev)

## Git Workflow

- Main branch: `main`
- Development branch: `dev`
