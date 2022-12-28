//! Tests that explicitly set attributes for dynamic entries are not ignored.

use time_sheet::input::toml_input::{self, Global};
use time_sheet::{time_stamp, working_duration};

use pretty_assertions::assert_eq;

mod common;

#[test]
fn test_explicit_start_and_pause() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let global: Global = toml::from_str(&common::make_global(working_duration!(40:00)))
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
        "start = \"13:00\"\n",
        "\n",
        "[dynamic.\"task b\"]\n",
        "flex = 1\n",
        "start = \"14:00\"\n",
        "pause = \"02:00\"\n",
        "\n",
    ))
    .expect("toml should be valid");

    let json_month_file = common::make_month_file(global, month);

    let mut task_a_duration = working_duration!(00:00);
    let mut task_b_duration = working_duration!(00:00);
    for entry in json_month_file.entries().iter() {
        if entry.action() == "task a" {
            assert_eq!(entry.time_span().start(), time_stamp!(13:00));
            task_a_duration += entry.work_duration();
        } else if entry.action() == "task b" {
            assert_eq!(entry.time_span().start(), time_stamp!(14:00));
            assert_eq!(entry.break_duration(), working_duration!(02:00));
            task_b_duration += entry.work_duration();
        }
    }

    assert_eq!(task_a_duration, working_duration!(20:00), "task a duration");
    assert_eq!(task_b_duration, working_duration!(20:00), "task b duration");
}
