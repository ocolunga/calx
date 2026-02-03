"""Calendar view rendering functions for calx CLI."""

import calendar
from datetime import date, timedelta
from rich.table import Table
from rich.text import Text

ALL_DAY_NAMES = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]


def get_day_names(first_day: int) -> list[str]:
    """Get day name abbreviations starting from first_day (1=Mon...7=Sun)."""
    idx = first_day - 1
    return ALL_DAY_NAMES[idx:] + ALL_DAY_NAMES[:idx]


def compute_week_number(d: date, first_day: int = 1, min_days: int = 4) -> int:
    """Compute week number for a date.

    Args:
        d: The date
        first_day: First day of week (1=Mon ... 7=Sun)
        min_days: Minimum days in first week (1=simple, 4=ISO-style)

    Returns:
        Week number (1-based)
    """
    # Fast path for standard ISO Monday
    if first_day == 1 and min_days == 4:
        return d.isocalendar()[1]

    py_first = (first_day - 1) % 7
    year = d.year
    jan1 = date(year, 1, 1)
    jan1_offset = (jan1.weekday() - py_first) % 7
    doy = (d - jan1).days

    if min_days == 1:
        # Simple mode: week containing Jan 1 is always week 1, no year spillover
        return (doy + jan1_offset) // 7 + 1

    # ISO-style (min_days > 1): year boundary spillover possible
    days_in_first_partial = 7 - jan1_offset if jan1_offset > 0 else 7

    if days_in_first_partial >= min_days:
        week = (doy + jan1_offset) // 7 + 1
    else:
        adjusted = doy - (7 - jan1_offset)
        if adjusted < 0:
            # Belongs to last week of previous year
            return compute_week_number(date(year - 1, 12, 31), first_day, min_days)
        week = adjusted // 7 + 1

    # Check spillover into next year's week 1
    next_jan1 = date(year + 1, 1, 1)
    next_jan1_offset = (next_jan1.weekday() - py_first) % 7
    next_first_partial = 7 - next_jan1_offset if next_jan1_offset > 0 else 7
    if next_first_partial >= min_days:
        next_week1_start = next_jan1 - timedelta(days=next_jan1_offset)
        if d >= next_week1_start:
            return 1

    return week


def compute_biweek_number(week: int) -> int:
    """Compute biweek number from a week number (for invoicing periods)."""
    return (week + 1) // 2


def render_month_calendar(
    year: int,
    month: int,
    today: date | None = None,
    show_week_numbers: bool = True,
    first_day: int = 1,
    min_days: int = 4,
) -> Table:
    """Render a single month calendar grid.

    Args:
        year: The year to render
        month: The month to render (1-12)
        today: Optional date to highlight
        show_week_numbers: Whether to show week numbers on the left
        first_day: First day of week (1=Mon...7=Sun)
        min_days: Minimum days in first week (1=simple, 4=ISO-style)
    """
    py_first = (first_day - 1) % 7
    cal = calendar.Calendar(firstweekday=py_first)
    weeks = cal.monthdatescalendar(year, month)

    table = Table.grid(padding=(0, 1))

    if show_week_numbers:
        table.add_column(justify="right", style="dim")
    for _ in range(7):
        table.add_column(justify="right", width=2)
    if show_week_numbers:
        table.add_column(justify="right", style="dim")

    day_names = get_day_names(first_day)
    header = [Text(d, style="bold cyan") for d in day_names]
    if show_week_numbers:
        table.add_row("", *header, "")
    else:
        table.add_row(*header)

    seen_biweeks: set[int] = set()

    for week in weeks:
        row: list[Text | str] = []
        week_num = None

        if show_week_numbers:
            for day in week:
                if day.month == month:
                    week_num = compute_week_number(day, first_day, min_days)
                    break
            row.append(
                Text(str(week_num) if week_num is not None else "", style="dim yellow")
            )

        for day in week:
            if day.month != month:
                row.append(Text("", style="dim"))
            elif today and day == today:
                row.append(Text(str(day.day), style="bold reverse"))
            else:
                row.append(Text(str(day.day)))

        if show_week_numbers and week_num is not None:
            bw = compute_biweek_number(week_num)
            if bw not in seen_biweeks:
                row.append(Text(f"B{bw}", style="dim magenta"))
                seen_biweeks.add(bw)
            else:
                row.append("")
        elif show_week_numbers:
            row.append("")

        table.add_row(*row)

    return table


def render_month_with_title(
    year: int,
    month: int,
    today: date | None = None,
    show_week_numbers: bool = True,
    first_day: int = 1,
    min_days: int = 4,
) -> Table:
    """Render a month calendar with month name header."""
    outer = Table.grid()
    outer.add_column()

    month_name = calendar.month_name[month]
    title = Text(f"{month_name} {year}", style="bold blue", justify="center")
    outer.add_row(title)

    cal_table = render_month_calendar(
        year, month, today, show_week_numbers, first_day, min_days
    )
    outer.add_row(cal_table)

    return outer


def render_three_months(
    center_year: int,
    center_month: int,
    today: date | None = None,
    first_day: int = 1,
    min_days: int = 4,
) -> Table:
    """Render three months side-by-side (previous, current, next)."""
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

    prev_cal = render_month_with_title(
        prev_year, prev_month, today, True, first_day, min_days
    )
    curr_cal = render_month_with_title(
        center_year, center_month, today, True, first_day, min_days
    )
    next_cal = render_month_with_title(
        next_year, next_month, today, True, first_day, min_days
    )

    table.add_row(prev_cal, curr_cal, next_cal)

    return table


def render_year(
    year: int,
    today: date | None = None,
    first_day: int = 1,
    min_days: int = 4,
) -> Table:
    """Render a full year calendar in a 4x3 grid."""
    outer = Table.grid()
    outer.add_column()

    title = Text(str(year), style="bold blue", justify="center")
    outer.add_row(title)
    outer.add_row("")

    grid = Table.grid(padding=(1, 2))
    grid.add_column()
    grid.add_column()
    grid.add_column()

    for row_idx in range(4):
        months_in_row = []
        for col_idx in range(3):
            month = row_idx * 3 + col_idx + 1
            month_cal = render_month_with_title(
                year, month, today, True, first_day, min_days
            )
            months_in_row.append(month_cal)
        grid.add_row(*months_in_row)

    outer.add_row(grid)

    return outer
