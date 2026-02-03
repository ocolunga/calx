#!/usr/bin/env python3

import typer
from datetime import date
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich import box

from calx.calendar_views import (
    compute_week_number,
    render_month_with_title,
    render_three_months,
    render_year,
)

console = Console()
app = typer.Typer(
    help="A CLI tool for displaying calendar information with terminal graphics"
)


def create_info_table(first_day: int = 1, min_days: int = 4) -> Table:
    """Create the calendar info table."""
    today = date.today()
    week = compute_week_number(today, first_day, min_days)
    day_of_year = today.timetuple().tm_yday

    table = Table(box=box.ROUNDED, show_header=False, padding=(0, 1))
    table.add_column("Property", style="cyan")
    table.add_column("Value", style="green")

    table.add_row("Week", str(week))
    table.add_row("Year", str(today.year))
    table.add_row("Day", today.strftime("%A"))
    table.add_row("Day of Year", str(day_of_year))

    return table


def display_default_view(first_day: int = 1, min_days: int = 4):
    """Display info table and current month calendar side-by-side."""
    today = date.today()

    layout = Table.grid(padding=(0, 2))
    layout.add_column()
    layout.add_column()

    info_table = create_info_table(first_day, min_days)
    info_panel = Panel(
        info_table,
        title="[bold blue]Calendar Info[/bold blue]",
        border_style="blue",
        padding=(1, 2),
    )

    month_cal = render_month_with_title(
        today.year, today.month, today, True, first_day, min_days
    )
    cal_panel = Panel(
        month_cal,
        title="[bold blue]Calendar[/bold blue]",
        border_style="blue",
        padding=(1, 2),
    )

    layout.add_row(info_panel, cal_panel)
    console.print(layout)


def resolve_shortcut_format(
    simple_sunday: bool,
    iso_sunday: bool,
    simple_monday: bool,
) -> tuple[int, int]:
    """Resolve shortcut flags into (first_day, min_days).

    Returns:
        Tuple of (first_day, min_days) where first_day is 1=Mon...7=Sun
        and min_days is 1 (simple) or 4 (ISO-style).
    """
    shortcuts = sum([simple_sunday, iso_sunday, simple_monday])
    if shortcuts > 1:
        console.print("[red]Error: Only one shortcut flag allowed[/red]")
        raise typer.Exit(1)

    if simple_sunday:
        return 7, 1
    elif iso_sunday:
        return 7, 4
    elif simple_monday:
        return 1, 1
    else:
        return 1, 4


@app.callback(invoke_without_command=True)
def main(
    ctx: typer.Context,
    month: bool = typer.Option(
        False, "-m", "--month", help="Show 3 months (previous, current, next)"
    ),
    year: bool = typer.Option(False, "-y", "--year", help="Show full year calendar"),
    simple_sunday: bool = typer.Option(
        False, "--simple-sunday", help="Sunday start, simple weeks"
    ),
    iso_sunday: bool = typer.Option(
        False, "--iso-sunday", help="Sunday start, ISO weeks"
    ),
    simple_monday: bool = typer.Option(
        False, "--simple-monday", help="Monday start, simple weeks"
    ),
):
    """Display calendar information with terminal graphics."""
    if ctx.invoked_subcommand is not None:
        return

    first_day, min_days = resolve_shortcut_format(
        simple_sunday, iso_sunday, simple_monday
    )
    today = date.today()

    if year:
        console.print(render_year(today.year, today, first_day, min_days))
    elif month:
        console.print(
            render_three_months(today.year, today.month, today, first_day, min_days)
        )
    else:
        display_default_view(first_day, min_days)


@app.command()
def show(
    day: int = typer.Option(
        1, "-d", "--day", min=1, max=7, help="First day of week (1=Mon...7=Sun)"
    ),
    simple: bool = typer.Option(
        False, "-s", "--simple", help="Simple week numbering (Jan 1 = week 1)"
    ),
    iso: bool = typer.Option(False, "-i", "--iso", help="ISO week numbering"),
    month: bool = typer.Option(
        False, "-m", "--month", help="Show 3 months (previous, current, next)"
    ),
    year: bool = typer.Option(False, "-y", "--year", help="Show full year calendar"),
):
    """Show calendar with custom week configuration."""
    if simple and iso:
        console.print("[red]Error: --simple and --iso are mutually exclusive[/red]")
        raise typer.Exit(1)

    min_days = 1 if simple else 4
    today = date.today()

    if year:
        console.print(render_year(today.year, today, day, min_days))
    elif month:
        console.print(
            render_three_months(today.year, today.month, today, day, min_days)
        )
    else:
        display_default_view(day, min_days)


if __name__ == "__main__":
    app()
