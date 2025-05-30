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
    }


def display_calendar_info():
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
):
    """Display calendar information in a formatted way."""
    display_calendar_info()


if __name__ == "__main__":
    app()
