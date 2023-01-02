//! Tests that explicitly set attributes for dynamic entries are not ignored.

use time_sheet::input::toml_input::{self, Global};
use time_sheet::time::{Date, Month, WeekDay, Year};
use time_sheet::{time_stamp, working_duration};

use pretty_assertions::assert_eq;

mod common;

use common::IndexMap;

#[test]
fn test_repeating_dynamic() {
    // common::debug_setup();

    // The problem with the scheduler:
    // Suppose an entry repeats on a tuesday, then one can schedule work
    // on monday.
    //
    // It will schedule as much work as possible on monday and then
    // leave no time for the work on tuesday (the average time/week would be
    // exceeded).
    //
    // The problem is that the flex entries are adjusted to sum up with the
    // dynamic entries to exactly 40:00.
    //
    // So this will result in an invalid time-sheet, where the 40:00 are not
    // reached.
    //
    // Solution: The repeating entries can bypass the weekly limit
    // NOTE: this will cause problems when repeating entries exceed the monthly limit

    let global: Global = toml::from_str(
        &(common::make_global(working_duration!(40:00))
            + concat!(
                "[repeating.\"regular work\"]\n",
                "start = \"08:00\"\n",
                "repeats_on = [\"Tuesday\"]\n",
                "repeats = \"weekly\"\n",
                "duration = \"03:00\"\n",
            )),
    )
    .expect("toml should be valid");

    let month: toml_input::Month = toml::from_str(concat!(
        //
        "[general]\n",
        "month = 8\n",
        "year = 2022\n",
        "department = \"MENSA\"\n",
        "\n",
        "[dynamic.\"task a\"]\n",
        "flex = 1\n",
        "\n",
        "[dynamic.\"task b\"]\n",
        "flex = 1\n",
        "\n",
    ))
    .expect("toml should be valid");

    let json_month_file = common::make_month_file(global, month);

    let mut count = 0;
    for entry in json_month_file.entries() {
        if entry.action() == "regular work" {
            count += 1;
            assert_eq!(entry.time_span().start(), time_stamp!(08:00));
            assert_eq!(
                Date::new(2022, Month::August, entry.day())
                    .unwrap()
                    .week_day(),
                WeekDay::Tuesday
            );
        }
    }

    assert_eq!(
        count,
        Year::new(2022)
            .days_in(Month::August)
            .filter(|day| day.week_day() == WeekDay::Tuesday)
            .count()
    );

    assert_eq!(
        common::get_proportions(&json_month_file),
        IndexMap::from(vec![
            ("task a", working_duration!(12:30)),
            ("regular work", working_duration!(15:00)),
            ("task b", working_duration!(12:30)),
        ])
    );
}
