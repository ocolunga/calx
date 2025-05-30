#!/usr/bin/env python3

import typer
from datetime import datetime, timedelta
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich import box
from typing import Optional

console = Console()
app = typer.Typer(
    help="A CLI tool for displaying calendar information with terminal graphics"
)


def get_first_week_of_month():
    today = datetime.now()
    first_day_of_month = today.replace(day=1)
    # Get ISO week number of the first day of the month
    return first_day_of_month.isocalendar()[1]


def get_biweek_number():
    today = datetime.now()
    day_of_year = today.timetuple().tm_yday
    # Calculate biweek number (1-26 or 27 in leap years)
    # Ceiling division to get the biweek number
    return (day_of_year + 13) // 14


def get_week_info():
    today = datetime.now()
    iso_week = today.isocalendar()

    # Calculate week starting Monday
    monday_week = (today - timedelta(days=today.weekday())).isocalendar()[1]

    # Calculate week starting Sunday
    sunday_week = (
        today - timedelta(days=(today.weekday() + 1) % 7)
    ).isocalendar()[1]

    return {
        "iso_week": iso_week[1],
        "monday_week": monday_week,
        "sunday_week": sunday_week,
        "year": iso_week[0],
        "day_of_week": today.strftime("%A"),
        "first_week_of_month": get_first_week_of_month(),
        "day_of_year": today.timetuple().tm_yday,
        "biweek": get_biweek_number(),
    }


def display_calendar_info(
    show_first_week: bool = True,
    show_day_of_year: bool = True,
    show_biweek: bool = True,
):
    week_info = get_week_info()

    # Create a table for the week information
    table = Table(box=box.ROUNDED, show_header=False, padding=(0, 1))
    table.add_column("Property", style="cyan")
    table.add_column("Value", style="green")

    table.add_row("ISO Week Number", str(week_info["iso_week"]))
    table.add_row("Week (Monday Start)", str(week_info["monday_week"]))
    table.add_row("Week (Sunday Start)", str(week_info["sunday_week"]))
    table.add_row("Year", str(week_info["year"]))
    table.add_row("Current Day", week_info["day_of_week"])
    if show_first_week:
        table.add_row(
            "First Week of Month", str(week_info["first_week_of_month"])
        )
    if show_day_of_year:
        table.add_row("Day of Year", str(week_info["day_of_year"]))
    if show_biweek:
        table.add_row("Biweek Number", str(week_info["biweek"]))

    # Create a panel with the table
    panel = Panel(
        table,
        title="[bold blue]Calendar Information[/bold blue]",
        border_style="blue",
        padding=(1, 2),
    )

    console.print(panel)


@app.command()
def show(
    iso: Optional[bool] = typer.Option(True, help="Show ISO week number"),
    monday: Optional[bool] = typer.Option(
        True, help="Show week number starting on Monday"
    ),
    sunday: Optional[bool] = typer.Option(
        True, help="Show week number starting on Sunday"
    ),
    year: Optional[bool] = typer.Option(True, help="Show current year"),
    day: Optional[bool] = typer.Option(True, help="Show current day of week"),
    first_week: Optional[bool] = typer.Option(
        True, help="Show first week number of the current month"
    ),
    day_of_year: Optional[bool] = typer.Option(
        True, help="Show day of year (1-366)"
    ),
    biweek: Optional[bool] = typer.Option(
        True, help="Show biweek number (1-26 or 27 in leap years)"
    ),
):
    """Display calendar information in a formatted way."""
    display_calendar_info(
        show_first_week=bool(first_week),
        show_day_of_year=bool(day_of_year),
        show_biweek=bool(biweek),
    )


if __name__ == "__main__":
    app()
