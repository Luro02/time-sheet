//! Tests that flex attribute of dynamic entries is working correctly.

use time_sheet::input::toml_input::{self, Global};
use time_sheet::working_duration;

use pretty_assertions::assert_eq;

use crate::common::IndexMap;

mod common;

#[test]
fn test_only_flex_divides_exact() {
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
        "\n",
        "[dynamic.\"task b\"]\n",
        "flex = 2\n",
        "\n",
        "[dynamic.\"task c\"]\n",
        "flex = 3\n",
        "\n",
        "[dynamic.\"task d\"]\n",
        "flex = 4\n",
    ))
    .expect("toml should be valid");

    let json_month_file = common::make_month_file(global, month);

    // task a: 1/10 =  4h
    // task b: 2/10 =  8h
    // task c: 3/10 = 12h
    // task d: 4/10 = 16h

    assert_eq!(
        common::get_proportions(&json_month_file),
        IndexMap::from(vec![
            ("task a", working_duration!(04:00)),
            ("task b", working_duration!(08:00)),
            ("task c", working_duration!(12:00)),
            ("task d", working_duration!(16:00)),
        ])
    );
}

#[test]
fn test_only_flex_remainder() {
    let global: Global = toml::from_str(&common::make_global(working_duration!(41:00)))
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
        "flex = 2\n",
        "\n",
        "[dynamic.\"task c\"]\n",
        "flex = 97\n",
    ))
    .expect("toml should be valid");

    let json_month_file = common::make_month_file(global, month);

    // task a:  1/100 = 00:24 + remainder (00:01)
    // task b:  2/100 = 00:49
    // task c: 97/100 = 39:46

    assert_eq!(
        common::get_proportions(&json_month_file),
        IndexMap::from(vec![
            ("task a", working_duration!(00:25)),
            ("task b", working_duration!(00:49)),
            ("task c", working_duration!(39:46)),
        ])
    );
}

#[test]
fn test_no_remaining_time() {
    let global: Global = toml::from_str(&common::make_global(working_duration!(41:00)))
        .expect("toml should be valid");

    let month: toml_input::Month = toml::from_str(concat!(
        //
        "[general]\n",
        "month = 8\n",
        "year = 2022\n",
        "department = \"MENSA\"\n",
        "\n",
        "[dynamic.\"task a\"]\n",
        "duration = \"41:00\"\n",
        "\n",
        "[dynamic.\"task b\"]\n",
        "flex = 1\n",
        "\n",
        "[dynamic.\"task c\"]\n",
        "flex = 1\n",
    ))
    .expect("toml should be valid");

    let json_month_file = common::make_month_file(global, month);

    // no remaining time, so all flex entries are not present

    assert_eq!(
        common::get_proportions(&json_month_file),
        IndexMap::from(vec![("task a", working_duration!(41:00))])
    );
}

#[test]
fn test_with_remaining_time() {
    let global: Global = toml::from_str(&common::make_global(working_duration!(41:00)))
        .expect("toml should be valid");

    let month: toml_input::Month = toml::from_str(concat!(
        //
        "[general]\n",
        "month = 8\n",
        "year = 2022\n",
        "department = \"MENSA\"\n",
        "\n",
        "[entries.1]\n",
        "action = \"task a\"\n",
        "start = \"09:00\"\n",
        "end = \"12:00\"\n",
        "pause = \"00:30\"\n",
        "\n",
        "[dynamic.\"task b\"]\n",
        "duration = \"10:00\"\n",
        "\n",
        "[dynamic.\"task c\"]\n",
        "flex = 1\n",
        "\n",
        "[dynamic.\"task d\"]\n",
        "flex = 2\n",
    ))
    .expect("toml should be valid");

    let json_month_file = common::make_month_file(global, month);

    // task a: 02:30h
    // task b: 10:00h
    // => 41:00h - 12:30h = 28:30h remaining
    // task c: 1/3 =  570min = 09:30h
    // task d: 2/3 = 1140min = 19:00h

    assert_eq!(
        common::get_proportions(&json_month_file),
        IndexMap::from(vec![
            ("task a", working_duration!(02:30)),
            ("task b", working_duration!(10:00)),
            ("task c", working_duration!(09:30)),
            ("task d", working_duration!(19:00)),
        ])
    );
}
