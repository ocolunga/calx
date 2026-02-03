"""Calendar view rendering functions for calx CLI."""

import calendar
from datetime import date
from rich.table import Table
from rich.text import Text


def render_month_calendar(
    year: int, month: int, today: date | None = None, show_week_numbers: bool = True
) -> Table:
    """Render a single month calendar with optional week numbers.

    Args:
        year: The year to render
        month: The month to render (1-12)
        today: Optional date to highlight as today
        show_week_numbers: Whether to show week numbers on the left

    Returns:
        A Rich Table containing the calendar
    """
    cal = calendar.Calendar(firstweekday=6)  # Sunday start
    weeks = cal.monthdatescalendar(year, month)

    table = Table.grid(padding=(0, 1))

    # Add columns
    if show_week_numbers:
        table.add_column(justify="right", style="dim")  # Week number
    for _ in range(7):
        table.add_column(justify="right", width=2)

    # Add header row with day names
    day_names = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"]
    if show_week_numbers:
        table.add_row("", *[Text(d, style="bold cyan") for d in day_names])
    else:
        table.add_row(*[Text(d, style="bold cyan") for d in day_names])

    # Add weeks
    for week in weeks:
        row = []

        if show_week_numbers:
            # Get week number from the first day that's in this month
            week_num = None
            for day in week:
                if day.month == month:
                    week_num = day.isocalendar()[1]
                    break
            row.append(Text(str(week_num) if week_num else "", style="dim yellow"))

        for day in week:
            if day.month != month:
                # Day from adjacent month
                row.append(Text("", style="dim"))
            elif today and day == today:
                # Today - highlight
                row.append(Text(str(day.day), style="bold reverse"))
            else:
                row.append(Text(str(day.day)))

        table.add_row(*row)

    return table


def render_month_with_title(
    year: int, month: int, today: date | None = None, show_week_numbers: bool = True
) -> Table:
    """Render a month calendar with month name and year header.

    Args:
        year: The year to render
        month: The month to render (1-12)
        today: Optional date to highlight as today
        show_week_numbers: Whether to show week numbers on the left

    Returns:
        A Rich Table containing the titled calendar
    """
    outer = Table.grid()
    outer.add_column()

    # Month/year title
    month_name = calendar.month_name[month]
    title = Text(f"{month_name} {year}", style="bold blue", justify="center")
    outer.add_row(title)

    # Calendar grid
    cal_table = render_month_calendar(year, month, today, show_week_numbers)
    outer.add_row(cal_table)

    return outer


def render_three_months(
    center_year: int, center_month: int, today: date | None = None
) -> Table:
    """Render three months side-by-side (previous, current, next).

    Args:
        center_year: The year of the center month
        center_month: The center month (1-12)
        today: Optional date to highlight as today

    Returns:
        A Rich Table containing three months
    """
    # Calculate previous and next months
    if center_month == 1:
        prev_year, prev_month = center_year - 1, 12
    else:
        prev_year, prev_month = center_year, center_month - 1

    if center_month == 12:
        next_year, next_month = center_year + 1, 1
    else:
        next_year, next_month = center_year, center_month + 1

    table = Table.grid(padding=(0, 2))
    table.add_column()
    table.add_column()
    table.add_column()

    prev_cal = render_month_with_title(prev_year, prev_month, today)
    curr_cal = render_month_with_title(center_year, center_month, today)
    next_cal = render_month_with_title(next_year, next_month, today)

    table.add_row(prev_cal, curr_cal, next_cal)

    return table


def render_year(year: int, today: date | None = None) -> Table:
    """Render a full year calendar in a 4x3 grid.

    Args:
        year: The year to render
        today: Optional date to highlight as today

    Returns:
        A Rich Table containing the year calendar
    """
    outer = Table.grid()
    outer.add_column()

    # Year title
    title = Text(str(year), style="bold blue", justify="center")
    outer.add_row(title)
    outer.add_row("")  # Spacer

    # 4 rows of 3 months each
    grid = Table.grid(padding=(1, 2))
    grid.add_column()
    grid.add_column()
    grid.add_column()

    for row_idx in range(4):
        months_in_row = []
        for col_idx in range(3):
            month = row_idx * 3 + col_idx + 1
            month_cal = render_month_with_title(
                year, month, today, show_week_numbers=True
            )
            months_in_row.append(month_cal)
        grid.add_row(*months_in_row)

    outer.add_row(grid)

    return outer
