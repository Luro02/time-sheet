use log::debug;

use crate::input::scheduler::{Scheduler, TimeSpanScheduler};
use crate::input::toml_input::Transfer;
use crate::time::{Date, DurationExt, Month, WorkingDuration, Year};
use crate::utils::{self, ArrayExt};
use crate::working_duration;

#[derive(Debug, Clone, PartialEq)]
pub struct MonthScheduler {
    weeks: [TimeSpanScheduler; 6],
    current_week: usize,
}

impl MonthScheduler {
    fn make_scheduler(
        year: Year,
        month: Month,
        available_time: impl Fn(usize) -> WorkingDuration,
    ) -> [TimeSpanScheduler; 6] {
        <[TimeSpanScheduler; 6]>::init_with(|mut week_number| {
            week_number += 1;
            year.days_in_week(month, week_number).map_or_else(
                || TimeSpanScheduler::empty(),
                |days| {
                    TimeSpanScheduler::new(*days.start(), *days.end(), available_time(week_number))
                },
            )
        })
    }

    pub fn new(year: Year, month: Month, maximum_time: WorkingDuration) -> Self {
        Self::new_with_available_time(year, month, maximum_time, |date| {
            if date.is_workday() {
                working_duration!(00:01)
            } else {
                working_duration!(00:00)
            }
        })
    }

    #[must_use]
    pub fn new_with_available_time(
        year: Year,
        month: Month,
        maximum_time: WorkingDuration,
        // Returns how much time is available on the given date
        // This can be used to indicate through absences or holidays
        // that less time is available on a given date.
        mut available_time: impl FnMut(Date) -> WorkingDuration,
    ) -> Self {
        debug!(
            "MonthScheduler: maximum working time per month: {}",
            maximum_time
        );

        let mut iter = year.days_in(month);

        let workday_distribution = [(); 31].map(|_| {
            if let Some(next_date) = iter.next() {
                available_time(next_date)
            } else {
                working_duration!(00:00)
            }
        });

        let mut distribution = workday_distribution.map(|w| w.to_duration().as_mins() as usize);
        let remainder = utils::divide_proportionally(
            maximum_time.to_duration().as_mins() as usize,
            &mut distribution,
        );

        let week_with_remainder = (year.number_of_weeks_in_month(month) + 1) / 2;

        Self {
            weeks: Self::make_scheduler(year, month, |week_number| {
                let mut result = working_duration!(00:00);

                for day in year.days_in_week(month, week_number).into_iter().flatten() {
                    result += WorkingDuration::from_mins(distribution[day.day() - 1] as u16);
                }

                if week_number == week_with_remainder {
                    result += WorkingDuration::from_mins(remainder as u16);
                }

                result
            }),
            current_week: 0,
        }
    }

    fn transfer_from_week_to_week(&self, from: usize, to: usize) -> [TimeSpanScheduler; 6] {
        let mut result = self.weeks.clone();

        if to > from {
            let mut target = result[to].clone();
            result[from..to].iter_mut().fold(&mut target, |acc, week| {
                acc.add_transfer(week.take_transfer());
                acc
            });
            result[to] = target;
        }

        result
    }

    pub fn transfer_time(&self) -> Transfer {
        let last_week = self.weeks.len() - 1;
        self.transfer_from_week_to_week(self.current_week, last_week)[last_week].transfer()
    }

    fn transfer_overflow(&mut self, from: usize) {
        let mut next_week = from + 1;

        // check if too much has been scheduled in that week:
        while next_week < self.weeks.len() && self.weeks[next_week - 1].transfer().is_positive() {
            // then transfer it to the next week:
            let taken_transfer = self.weeks[next_week - 1].take_transfer();
            self.weeks[next_week].add_transfer(taken_transfer);

            next_week += 1;
        }

        // if the last week has a positive transfer, then it will overflow
        // into the next month
        //
        // this is not desired if there is still time left in the month,
        // so the transfer time from the last week will be added to the remaining
        // weeks with time
        if self.weeks[self.weeks.len() - 1].transfer().is_positive() {
            let mut transfer = Transfer::default();

            for week in self.weeks.iter_mut().rev() {
                week.add_transfer(transfer);
                transfer = Transfer::default();

                if week.transfer().is_positive() {
                    transfer = week.take_transfer();
                } else {
                    break;
                }
            }

            // if the transfer can not be distributed over the weeks,
            // it will accumulate at the beginning of the month (will be stored in
            // the transfer variable)
            // This is not ideal, so transfer it all to the last week:
            if transfer.is_positive() {
                self.weeks[self.weeks.len() - 1].add_transfer(transfer);
            }
        }
    }
}

impl Scheduler for MonthScheduler {
    fn has_time_for(&self, date: Date, wanted_duration: WorkingDuration) -> WorkingDuration {
        let week_number = date.week_number() - 1;

        self.transfer_from_week_to_week(self.current_week, week_number)[week_number]
            .has_time_for(date, wanted_duration)
    }

    fn schedule(&mut self, date: Date, worked: WorkingDuration) {
        let week_number = date.week_number() - 1;
        if week_number > self.current_week {
            self.weeks = self.transfer_from_week_to_week(self.current_week, week_number);
            self.current_week = week_number;
        } else if week_number < self.current_week {
            panic!("Work must be scheduled in chronological order");
        }

        self.weeks[week_number].schedule(date, worked)
    }

    fn schedule_in_advance(&mut self, date: Date, worked: WorkingDuration) {
        let week_number = date.week_number() - 1;
        self.weeks[week_number].schedule(date, worked);
        self.transfer_overflow(week_number);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use crate::{date, transfer};

    #[test]
    fn test_transfer_from_week_to_week() {
        let scheduler = MonthScheduler::new(Year::new(2023), Month::July, working_duration!(12:35));

        // No holidays in july, therefore 1 + 6 * 4 + 1 = 26 days
        // 12 * 60 + 35 = 755 minutes
        // 755 / 26 = 29 minutes per day and 1 minute remainder

        let expected_results = [
            // week 0 to 0, 1, 2, 3, 4, 5
            [
                // week 0
                transfer!(-00:29) * 1,
                // week 1
                transfer!(-00:29) * 7,
                // week 2
                transfer!(-00:29) * 13 + transfer!(-00:01),
                // week 3
                transfer!(-00:29) * 19 + transfer!(-00:01),
                // week 4
                transfer!(-00:29) * 25 + transfer!(-00:01),
                // week 5
                transfer!(-00:29) * 26 + transfer!(-00:01),
            ],
            // week 1 to 0, 1, 2, 3, 4, 5
            [
                // week 0
                transfer!(+00:00), // for now transfer from week 1 to week 0 is not supported
                // week 1
                transfer!(-00:29) * 6,
                // week 2
                transfer!(-00:29) * 12 + transfer!(-00:01),
                // week 3
                transfer!(-00:29) * 18 + transfer!(-00:01),
                // week 4
                transfer!(-00:29) * 24 + transfer!(-00:01),
                // week 5
                transfer!(-00:29) * 25 + transfer!(-00:01),
            ],
            // week 2 to 0, 1, 2, 3, 4, 5
            [
                // week 0
                transfer!(+00:00),
                // week 1
                transfer!(+00:00),
                // week 2
                transfer!(-00:29) * 6 + transfer!(-00:01),
                // week 3
                transfer!(-00:29) * 12 + transfer!(-00:01),
                // week 4
                transfer!(-00:29) * 18 + transfer!(-00:01),
                // week 5
                transfer!(-00:29) * 19 + transfer!(-00:01),
            ],
            // week 3 to 0, 1, 2, 3, 4, 5
            [
                // week 0
                transfer!(+00:00),
                // week 1
                transfer!(+00:00),
                // week 2
                transfer!(+00:00),
                // week 3
                transfer!(-00:29) * 6,
                // week 4
                transfer!(-00:29) * 12,
                // week 5
                transfer!(-00:29) * 13,
            ],
            // week 4 to 0, 1, 2, 3, 4, 5
            [
                // week 0
                transfer!(+00:00),
                // week 1
                transfer!(+00:00),
                // week 2
                transfer!(+00:00),
                // week 3
                transfer!(+00:00),
                // week 4
                transfer!(-00:29) * 6,
                // week 5
                transfer!(-00:29) * 7,
            ],
            // week 5 to 0, 1, 2, 3, 4, 5
            [
                // week 0
                transfer!(+00:00),
                // week 1
                transfer!(+00:00),
                // week 2
                transfer!(+00:00),
                // week 3
                transfer!(+00:00),
                // week 4
                transfer!(+00:00),
                // week 5
                transfer!(-00:29) * 1,
            ],
        ];

        for (week_number_from, weeks) in expected_results.into_iter().enumerate() {
            for (week_number_to, expected_results_to) in weeks.into_iter().enumerate() {
                if week_number_from > week_number_to {
                    continue;
                }

                let result = scheduler.transfer_from_week_to_week(week_number_from, week_number_to);

                assert_eq!(
                    result[week_number_to].transfer(),
                    expected_results_to,
                    "week_number_from: {}, week_number_to: {}",
                    week_number_from,
                    week_number_to
                );
            }
        }
    }

    #[test]
    fn test_new() {
        let time_per_day = working_duration!(01:38);
        let remainder = working_duration!(00:10);

        // 4 * 98 = 392
        // 6 * 98 = 588
        // 3 * 98 = 294
        // = 2450

        // week 1 = 393
        // week 2 = 590
        // week 3 = 590
        // week 4 = 590
        // week 5 = 295
        // week 6 =   0
        // = 2458

        assert_eq!(
            MonthScheduler::new(Year::new(2022), Month::November, working_duration!(41:00)),
            MonthScheduler {
                current_week: 0,
                weeks: [
                    TimeSpanScheduler::new(date!(2022:11:01), date!(2022:11:06), time_per_day * 4),
                    TimeSpanScheduler::new(date!(2022:11:07), date!(2022:11:13), time_per_day * 6),
                    TimeSpanScheduler::new(
                        date!(2022:11:14),
                        date!(2022:11:20),
                        time_per_day * 6 + remainder
                    ),
                    TimeSpanScheduler::new(date!(2022:11:21), date!(2022:11:27), time_per_day * 6),
                    TimeSpanScheduler::new(date!(2022:11:28), date!(2022:11:30), time_per_day * 3),
                    TimeSpanScheduler::empty(),
                ]
            }
        );

        let time_per_day = working_duration!(00:46);
        let remainder = working_duration!(00:04);

        assert_eq!(
            MonthScheduler::new(Year::new(2022), Month::July, working_duration!(20:00)),
            MonthScheduler {
                current_week: 0,
                weeks: [
                    TimeSpanScheduler::new(date!(2022:07:01), date!(2022:07:03), time_per_day * 2),
                    TimeSpanScheduler::new(date!(2022:07:04), date!(2022:07:10), time_per_day * 6),
                    TimeSpanScheduler::new(
                        date!(2022:07:11),
                        date!(2022:07:17),
                        time_per_day * 6 + remainder
                    ),
                    TimeSpanScheduler::new(date!(2022:07:18), date!(2022:07:24), time_per_day * 6),
                    TimeSpanScheduler::new(date!(2022:07:25), date!(2022:07:31), time_per_day * 6),
                    TimeSpanScheduler::empty(),
                ]
            }
        );
    }

    #[test]
    fn test_asking_too_much_time_with_transfer() {
        let scheduler =
            MonthScheduler::new(Year::new(2022), Month::November, working_duration!(41:00));

        // workable_days: 25
        // time_per_day: 41 / 25 = 1.64
        // => 1 hour and 38 minutes per day
        // remainder: 41 * 60 % 25 = 10mins (will get week 3)

        let time_per_day = working_duration!(01:38);
        let remainder = working_duration!(00:10);

        assert_eq!(
            scheduler.transfer_time(),
            Transfer::new(working_duration!(41:00), working_duration!(00:00))
        );

        // week 1: 4 days
        assert_eq!(
            scheduler.has_time_for(date!(2022:11:01), working_duration!(41:00)),
            time_per_day * 4,
        );

        // week 2: 6 days
        assert_eq!(
            scheduler.has_time_for(date!(2022:11:08), working_duration!(41:00)),
            time_per_day * (6 + 4),
        );

        // week 3: 6 days
        assert_eq!(
            scheduler.has_time_for(date!(2022:11:15), working_duration!(41:00)),
            time_per_day * (6 * 2 + 4) + remainder,
        );

        // week 4: 6 days
        assert_eq!(
            scheduler.has_time_for(date!(2022:11:22), working_duration!(41:00)),
            time_per_day * (6 * 3 + 4) + remainder,
        );

        // week 5: 3 days
        assert_eq!(
            scheduler.has_time_for(date!(2022:11:29), working_duration!(41:00)),
            time_per_day * (6 * 3 + 4 + 3) + remainder,
        );

        assert_eq!(
            scheduler.transfer_time(),
            Transfer::new(working_duration!(41:00), working_duration!(00:00))
        );
    }

    #[test]
    fn test_reverse_transfer() {
        let mut scheduler =
            MonthScheduler::new(Year::new(2022), Month::November, working_duration!(10:00));

        // 25 workable days:
        // - week 1: 4 days -> 4/25 * 10h =  96 mins
        // - week 2: 6 days -> 6/25 * 10h = 144 mins
        // - week 3: 6 days -> 6/25 * 10h = 144 mins
        // - week 4: 6 days -> 6/25 * 10h = 144 mins
        // - week 5: 3 days -> 3/25 * 10h =  72 mins

        // worked way more in the last week
        scheduler.schedule_in_advance(date!(2022:11:29), working_duration!(05:00));

        // this influences the distribution
        // (228 mins need to be distributed across the previous weeks):
        // - week 1: 4 days -> 4/25 * 10h =  96 mins
        // - week 2: 6 days -> 6/25 * 10h = 144 mins
        // - week 3: 6 days -> 6/25 * 10h =  60 mins
        // - week 4: 6 days -> 6/25 * 10h =   0 mins
        // - week 5: 3 days -> 3/25 * 10h =   0 mins

        assert_eq!(
            scheduler.has_time_for(date!(2022:11:02), working_duration!(10:00)),
            working_duration!(01:36)
        );

        assert_eq!(
            scheduler.has_time_for(date!(2022:11:08), working_duration!(10:00)),
            working_duration!(04:00)
        );

        assert_eq!(
            scheduler.has_time_for(date!(2022:11:15), working_duration!(10:00)),
            working_duration!(05:00)
        );

        assert_eq!(
            scheduler.has_time_for(date!(2022:11:22), working_duration!(10:00)),
            working_duration!(05:00)
        );

        assert_eq!(
            scheduler.has_time_for(date!(2022:11:29), working_duration!(10:00)),
            working_duration!(05:00)
        );

        assert_eq!(scheduler.transfer_time(), transfer!(-05:00));
    }

    #[test]
    fn test_impossible_transfer() {
        // this will happen when every week is full
        let mut scheduler =
            MonthScheduler::new(Year::new(2022), Month::November, working_duration!(10:00));

        // 25 workable days:
        // - week 1: 4 days -> 4/25 * 10h =  96 mins
        // - week 2: 6 days -> 6/25 * 10h = 144 mins
        // - week 3: 6 days -> 6/25 * 10h = 144 mins
        // - week 4: 6 days -> 6/25 * 10h = 144 mins
        // - week 5: 3 days -> 3/25 * 10h =  72 mins

        // worked too much:
        scheduler.schedule_in_advance(date!(2022:11:29), working_duration!(05:00));
        scheduler.schedule_in_advance(date!(2022:11:02), working_duration!(07:00));

        // this influences the distribution
        // - week 1: 4 days -> 4/25 * 10h =   0 mins
        // - week 2: 6 days -> 6/25 * 10h =   0 mins
        // - week 3: 6 days -> 6/25 * 10h =   0 mins
        // - week 4: 6 days -> 6/25 * 10h =   0 mins
        // - week 5: 3 days -> 3/25 * 10h =   0 mins
        // + transfer of 120 mins

        for date in [
            date!(2022:11:02),
            date!(2022:11:08),
            date!(2022:11:15),
            date!(2022:11:22),
            date!(2022:11:29),
        ] {
            assert_eq!(
                scheduler.has_time_for(date, working_duration!(10:00)),
                working_duration!(00:00)
            );
        }

        assert_eq!(scheduler.transfer_time(), transfer!(+02:00));
    }
}
