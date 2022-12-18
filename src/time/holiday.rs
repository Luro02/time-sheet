use crate::time::{Date, Month, WeekDay};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HolidayEntry {
    name: &'static str,
    is_mandatory: bool,
}

impl HolidayEntry {
    #[must_use]
    pub const fn new_mandatory(name: &'static str) -> Self {
        Self {
            name,
            is_mandatory: true,
        }
    }
}

/// Returns `true` when the given date is on easter sunday.
///
/// The algorithm is based on <https://en.wikipedia.org/wiki/Date_of_Easter#Anonymous_Gregorian_algorithm>
const fn is_easter_sunday(date: Date) -> bool {
    let year = date.year().as_usize() as usize;

    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let n = (h + l - 7 * m + 114) / 31;
    let o = (h + l - 7 * m + 114) % 31;

    Month::new(n).is_eq(&date.month()) && o + 1 == date.day()
}

pub const fn get_holiday_entry(date: Date) -> Option<HolidayEntry> {
    let fixed_holidays = [
        (1, Month::January, HolidayEntry::new_mandatory("Neujahr")),
        (
            6,
            Month::January,
            HolidayEntry::new_mandatory("Heilige Drei Könige"),
        ),
        (1, Month::May, HolidayEntry::new_mandatory("Tag der Arbeit")),
        (
            3,
            Month::October,
            HolidayEntry::new_mandatory("Tag der deutschen Einheit"),
        ),
        (
            1,
            Month::November,
            HolidayEntry::new_mandatory("Allerheiligen"),
        ),
        (
            25,
            Month::December,
            HolidayEntry::new_mandatory("1. Weihnachtsfeiertag"),
        ),
        (
            26,
            Month::December,
            HolidayEntry::new_mandatory("2. Weihnachtsfeiertag"),
        ),
    ];

    let mut i = 0;
    while i < fixed_holidays.len() {
        let (day, month, entry) = fixed_holidays[i];

        if date.day() == day && date.month().is_eq(&month) {
            return Some(entry);
        }

        i += 1;
    }

    if is_easter_sunday(date.sub_days(1)) {
        return Some(HolidayEntry::new_mandatory("Ostermontag"));
    }

    if date.week_day().is_eq(&WeekDay::Thursday) && is_easter_sunday(date.sub_days(39)) {
        return Some(HolidayEntry::new_mandatory("Christi Himmelfahrt"));
    }

    if date.week_day().is_eq(&WeekDay::Friday) && is_easter_sunday(date.add_days(2)) {
        return Some(HolidayEntry::new_mandatory("Karfreitag"));
    }

    if date.week_day().is_eq(&WeekDay::Monday) && is_easter_sunday(date.sub_days(50)) {
        return Some(HolidayEntry::new_mandatory("Pfingstmontag"));
    }

    if date.week_day().is_eq(&WeekDay::Thursday) && is_easter_sunday(date.sub_days(60)) {
        return Some(HolidayEntry::new_mandatory("Fronleichnam"));
    }

    None
}

pub const fn is_holiday(date: Date) -> bool {
    get_holiday_entry(date).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use crate::date;

    #[test]
    fn test_is_holiday() {
        let holidays = [
            date!(2023:01:01),
            date!(2023:01:06),
            date!(2023:04:07),
            date!(2023:04:10),
            date!(2023:05:01),
            date!(2023:05:18),
            date!(2023:05:29),
            date!(2023:06:08),
            date!(2023:10:03),
            date!(2023:11:01),
            date!(2023:12:25),
            date!(2023:12:26),
        ];

        for date in holidays[0]..=holidays[holidays.len() - 1] {
            if holidays.contains(&date) {
                assert_eq!(is_holiday(date), true, "date {} should be a holiday", date);
            } else {
                assert_eq!(
                    is_holiday(date),
                    false,
                    "date {} should not be a holiday",
                    date
                );
            }
        }
    }

    #[test]
    fn test_is_easter_sunday() {
        for date in [
            date!(2022:04:17),
            date!(2017:04:16),
            date!(2018:04:01),
            date!(2019:04:21),
            date!(2020:04:12),
            date!(2021:04:04),
            date!(2022:04:17),
            date!(2023:04:09),
            date!(2024:03:31),
            date!(2025:04:20),
            date!(2026:04:05),
            date!(2027:03:28),
            date!(2028:04:16),
            date!(2029:04:01),
            date!(2030:04:21),
            date!(2031:04:13),
            date!(2032:03:28),
            date!(2033:04:17),
            date!(2034:04:09),
            date!(2035:03:25),
            date!(2036:04:13),
            date!(2037:04:05),
        ] {
            assert_eq!(
                is_easter_sunday(date),
                true,
                "date {} should be easter sunday",
                date
            );
        }
    }

    #[test]
    #[ignore = "This test is ignored because it requires an internet connection"]
    fn test_is_up_to_date() {
        use serde::Deserialize;
        use std::collections::HashMap;

        #[derive(Debug, Clone, Deserialize)]
        struct Entry {
            #[serde(rename = "datum")]
            date: Date,
            #[serde(rename = "hinweis")]
            hint: String,
        }

        for year in 2022..=2025 {
            let res = minreq::get(format!(
                "https://feiertage-api.de/api/?jahr={}&nur_land=BW",
                year
            ))
            .send()
            .expect("Can not reach web api");

            let data: HashMap<String, Entry> =
                serde_json::from_slice(res.as_bytes()).expect("Format seems to have changed");

            for (_name, entry) in data {
                if !entry.hint.is_empty() {
                    continue;
                }

                assert_eq!(
                    is_holiday(entry.date),
                    true,
                    "date {} should be a holiday",
                    entry.date
                );
            }
        }
    }
}
