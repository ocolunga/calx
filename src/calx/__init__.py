"""calx - Calendar Expanded."""

from importlib.metadata import version

__version__ = version("calx-cli")

from calx.cli import app

__all__ = ["app"]
