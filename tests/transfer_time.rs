//! Tests that the transfer time is correctly declared and dynamic
//! entries fill it up correctly.

use time_sheet::input::json_input::{Entry, MonthFile};
use time_sheet::input::toml_input::{self, Global};
use time_sheet::time::{Month, Year};
use time_sheet::{time_stamp, transfer, working_duration};

use pretty_assertions::assert_eq;

mod common;

#[test]
fn test_transfer_is_optional() {
    let month: toml_input::Month = toml::from_str(concat!(
        //
        "[general]\n",
        "month = 11\n",
        "year = 2022\n",
        "department = \"MENSA\"\n",
        "\n",
    ))
    .expect("transfer section should be optional");

    assert_eq!(month.transfer(), None);
}

#[test]
fn test_transfer_previous_month() {
    let global: Global = toml::from_str(
        &(common::make_global(working_duration!(15:00))
            + concat!(
                "[repeating.\"regular work\"]\n",
                "start = \"08:00\"\n",
                "end = \"10:00\"\n",
                "repeats_on = [\"Tuesday\"]\n",
                "repeats = \"weekly\"\n"
            )),
    )
    .expect("toml should be valid");

    let month: toml_input::Month = toml::from_str(concat!(
        //
        "[general]\n",
        "month = 11\n",
        "year = 2022\n",
        "department = \"MENSA\"\n",
        "\n",
        "[transfer]\n",
        "prev = \"02:00\"\n",
        "next = \"00:00\"\n",
        "\n",
        "[dynamic.\"filler\"]\n",
        "duration = \"20:00\"\n"
    ))
    .expect("toml should be valid");

    assert_eq!(
        common::make_month_file(global, month),
        MonthFile::new(
            Year::new(2022),
            Month::November,
            transfer!(-02:00),
            vec![
                // in the previous month two hours more have been worked
                // -> this month only 13 hours are needed
                // of those 13 hours, 4 * 2 hours = 8 hours are regular work.
                // -> 5 hours are left for filler
                //
                // 25 workdays in that month.
                //
                // Distribution for fillers:
                // - week 1: 4 days -> 4/25 * 13 = 124 mins
                // - week 2: 6 days -> 6/25 * 13 = 187 mins
                // - week 3: 6 days -> 6/25 * 13 = 187 mins + 2 mins
                // - week 4: 6 days -> 6/25 * 13 = 187 mins
                // - week 5: 3 days -> 3/25 * 13 =  93 mins
                Entry::new("filler", 2, time_stamp!(10:00), time_stamp!(12:04), None),
                Entry::new("filler", 7, time_stamp!(10:00), time_stamp!(11:06), None),
                Entry::new(
                    "regular work",
                    8,
                    time_stamp!(08:00),
                    time_stamp!(10:00),
                    None,
                ),
                Entry::new("filler", 14, time_stamp!(10:00), time_stamp!(11:11), None),
                Entry::new(
                    "regular work",
                    15,
                    time_stamp!(08:00),
                    time_stamp!(10:00),
                    None,
                ),
                Entry::new("filler", 21, time_stamp!(10:00), time_stamp!(11:06), None),
                Entry::new(
                    "regular work",
                    22,
                    time_stamp!(08:00),
                    time_stamp!(10:00),
                    None,
                ),
                Entry::new(
                    "regular work",
                    29,
                    time_stamp!(08:00),
                    time_stamp!(10:00),
                    None,
                ),
            ]
        )
    );
}

#[test]
fn test_transfer_previous_and_next_month() {
    let global: Global = toml::from_str(
        &(common::make_global(working_duration!(15:00))
            + concat!(
                "[repeating.\"regular work\"]\n",
                "start = \"08:00\"\n",
                "end = \"10:00\"\n",
                "repeats_on = [\"Tuesday\"]\n",
                "repeats = \"weekly\"\n"
            )),
    )
    .expect("toml should be valid");

    let month: toml_input::Month = toml::from_str(concat!(
        //
        "[general]\n",
        "month = 11\n",
        "year = 2022\n",
        "department = \"MENSA\"\n",
        "\n",
        "[transfer]\n",
        "prev = \"02:00\"\n",
        "next = \"05:00\"\n",
        "\n",
        "[dynamic.\"filler\"]\n",
        "duration = \"20:00\"\n"
    ))
    .expect("toml should be valid");

    assert_eq!(
        common::make_month_file(global, month),
        MonthFile::new(
            Year::new(2022),
            Month::November,
            transfer!(+03:00),
            vec![
                // Of those 18 hours, 4 * 2 hours = 8 hours are regular work.
                // -> 10 hours are left for filler
                //
                // 25 workdays in that month.
                //
                // Distribution:
                // - week 1: 4 days -> 4/25 * 18h = 172 mins
                // - week 2: 6 days -> 6/25 * 18h = 259 mins
                // - week 3: 6 days -> 6/25 * 18h = 259 mins + 2 mins
                // - week 4: 6 days -> 6/25 * 18h = 259 mins
                // - week 5: 3 days -> 3/25 * 18h = 129 mins
                //
                // Time for fillers:
                // - week 1: 172 mins      = 2h 52min
                // - week 2: 259 mins - 2h = 2h 19min
                // - week 3: 261 mins - 2h = 2h 21min
                // - week 4: 259 mins - 2h = 2h 19min
                // - week 5: 129 mins - 2h = 0h 09min
                Entry::new("filler", 2, time_stamp!(10:00), time_stamp!(12:52), None),
                Entry::new("filler", 7, time_stamp!(10:00), time_stamp!(12:18), None),
                Entry::new(
                    "regular work",
                    8,
                    time_stamp!(08:00),
                    time_stamp!(10:00),
                    None,
                ),
                Entry::new("filler", 14, time_stamp!(10:00), time_stamp!(12:23), None),
                Entry::new(
                    "regular work",
                    15,
                    time_stamp!(08:00),
                    time_stamp!(10:00),
                    None,
                ),
                Entry::new("filler", 21, time_stamp!(10:00), time_stamp!(12:18), None),
                Entry::new(
                    "regular work",
                    22,
                    time_stamp!(08:00),
                    time_stamp!(10:00),
                    None,
                ),
                Entry::new("filler", 28, time_stamp!(10:00), time_stamp!(10:09), None),
                Entry::new(
                    "regular work",
                    29,
                    time_stamp!(08:00),
                    time_stamp!(10:00),
                    None,
                ),
            ]
        )
    );
}
