//! Tests that repeating events are skipped if they are on a day
//! where there is an absence.

use time_sheet::input::json_input::{Entry, MonthFile};
use time_sheet::input::toml_input::{self, Global, Transfer};
use time_sheet::input::Config;
use time_sheet::time::{Month, Year};
use time_sheet::{time_stamp, working_duration};

use pretty_assertions::assert_eq;

#[test]
fn test_repeating_and_absence() {
    let global: Global = toml::from_str(concat!(
        //
        "[about]\n",
        "name = \"John Smith\"\n",
        "staff_id = 1234567\n",
        "\n",
        "[contract.MENSA]\n",
        "working_time = \"40:00\"\n",
        "area = \"gf\"\n",
        "wage = 12.00\n",
        "start_date = 2009-10-01\n",
        "end_date = 2239-09-30\n",
        "\n",
        "[repeating.\"regular work\"]\n",
        "start = \"08:00\"\n",
        "end = \"12:00\"\n",
        "repeats_on = [\"Tuesday\", \"Friday\"]\n",
        "repeats = \"weekly\"\n"
    ))
    .expect("toml should be valid");

    let month: toml_input::Month = toml::from_str(concat!(
        //
        "[general]\n",
        "month = 11\n",
        "year = 2022\n",
        "\n",
        "[absence.08]\n",
        "start = \"08:00\"\n",
        "end = \"12:00\"\n",
        "\n",
        "[dynamic.\"filler\"]\n",
        "duration = \"08:00\"\n"
    ))
    .expect("toml should be valid");

    let config = Config::try_from_toml(month, global, "MENSA")
        .expect("config should be valid")
        .build();

    let json_month_file: MonthFile = serde_json::from_str(
        &config
            .to_month_json()
            .expect("should be able to make a json"),
    )
    .expect("should be able to parse the json to a MonthFile");

    assert_eq!(
        json_month_file,
        MonthFile::new(
            Year::new(2022),
            Month::November,
            Transfer::default(),
            vec![
                Entry::new("filler", 2, time_stamp!(10:00), time_stamp!(12:24), None,),
                Entry::new(
                    "regular work",
                    4,
                    time_stamp!(08:00),
                    time_stamp!(12:00),
                    None,
                ),
                Entry::new(
                    "filler",
                    7,
                    time_stamp!(10:00),
                    time_stamp!(16:06),
                    Some(working_duration!(00:30)),
                ),
                Entry::new(
                    "regular work",
                    11,
                    time_stamp!(08:00),
                    time_stamp!(12:00),
                    None,
                ),
                Entry::new(
                    "regular work",
                    15,
                    time_stamp!(08:00),
                    time_stamp!(12:00),
                    None,
                ),
                Entry::new(
                    "regular work",
                    18,
                    time_stamp!(08:00),
                    time_stamp!(12:00),
                    None,
                ),
                Entry::new(
                    "regular work",
                    22,
                    time_stamp!(08:00),
                    time_stamp!(12:00),
                    None,
                ),
                Entry::new(
                    "regular work",
                    25,
                    time_stamp!(08:00),
                    time_stamp!(12:00),
                    None,
                ),
                Entry::new(
                    "regular work",
                    29,
                    time_stamp!(08:00),
                    time_stamp!(12:00),
                    None,
                ),
            ]
        )
    );
}
