//! Tests that the proportional distribution strategy works correctly.

use time_sheet::input::json_input::{Entry, MonthFile};
use time_sheet::input::toml_input::{self, Global, Transfer};
use time_sheet::time::{Month, Year};
use time_sheet::{time_stamp, working_duration};

use pretty_assertions::assert_eq;

use crate::common::IndexMap;

mod common;

#[test]
fn test_proportional_dynamic_too_large() {
    let global: Global = toml::from_str(&(common::make_global(working_duration!(40:00))))
        .expect("toml should be valid");

    let month: toml_input::Month = toml::from_str(concat!(
        //
        "[general]\n",
        "month = 8\n",
        "year = 2022\n",
        "department = \"MENSA\"\n",
        "strategy = \"proportional\"\n",
        "\n",
        "[dynamic.\"task a\"]\n",
        "duration = \"16:00\"\n",
        "\n",
        "[dynamic.\"task b\"]\n",
        "duration = \"32:00\"\n",
        "\n",
        "[dynamic.\"task c\"]\n",
        "duration = \"16:00\"\n",
        "\n",
        "[dynamic.\"task d\"]\n",
        "duration = \"16:00\"\n",
    ))
    .expect("toml should be valid");

    let json_month_file = common::make_month_file(global, month);

    assert_eq!(
        common::get_proportions(&json_month_file),
        IndexMap::from(vec![
            ("task a", working_duration!(08:00)),
            ("task b", working_duration!(16:00)),
            ("task c", working_duration!(08:00)),
            ("task d", working_duration!(08:00)),
        ])
    );

    assert_eq!(
        json_month_file,
        MonthFile::new(
            Year::new(2022),
            Month::August,
            Transfer::default(),
            vec![
                Entry::new(
                    "task a",
                    1,
                    time_stamp!(10:00),
                    time_stamp!(16:30),
                    Some(working_duration!(00:30)),
                ),
                Entry::new("task a", 2, time_stamp!(10:00), time_stamp!(12:00), None),
                Entry::new("task b", 3, time_stamp!(10:00), time_stamp!(10:48), None),
                Entry::new(
                    "task b",
                    8,
                    time_stamp!(10:00),
                    time_stamp!(16:30),
                    Some(working_duration!(00:30))
                ),
                Entry::new("task b", 9, time_stamp!(10:00), time_stamp!(12:48), None),
                Entry::new(
                    "task b",
                    15,
                    time_stamp!(10:00),
                    time_stamp!(16:30),
                    Some(working_duration!(00:30))
                ),
                Entry::new("task b", 16, time_stamp!(10:00), time_stamp!(10:24), None),
                Entry::new("task c", 17, time_stamp!(10:00), time_stamp!(12:48), None),
                Entry::new(
                    "task c",
                    22,
                    time_stamp!(10:00),
                    time_stamp!(15:42),
                    Some(working_duration!(00:30))
                ),
                Entry::new("task d", 23, time_stamp!(10:00), time_stamp!(13:36), None),
                Entry::new(
                    "task d",
                    29,
                    time_stamp!(10:00),
                    time_stamp!(14:54),
                    Some(working_duration!(00:30))
                ),
            ]
        )
    );
}
