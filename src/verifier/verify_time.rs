use thiserror::Error;

use crate::input::Config;
use crate::time::{Date, PrettyDuration, TimeSpan, TimeStamp};
use crate::verifier::Verifier;

pub struct VerifyTime;

#[derive(Debug, Clone, Error)]
pub enum InvalidTime {
    #[error("It is forbidden to work at night: Worked on {day} for {duration} at {time_span} into {night_time}")]
    NightWork {
        duration: PrettyDuration,
        day: Date,
        time_span: TimeSpan,
        night_time: TimeSpan,
    },
}

impl Verifier for VerifyTime {
    type Error = InvalidTime;
    type Errors = Vec<Self::Error>;

    fn verify(&self, config: &Config) -> Result<(), Self::Errors> {
        let mut errors = Vec::new();

        let month_config = config.month();
        let month = month_config.month();
        let year = month_config.year();

        for day in year
            .iter_days_in(month)
            .filter(|date| month_config.has_entries_on(*date))
        {
            // TODO: one needs to sum up the times for all entries on a single day!
            for entry in config.month().entries_on_day(day) {
                // https://www.gesetze-im-internet.de/arbzg/BJNR117100994.html

                // this is not a night work, so you are not allowed to work
                // more than 2 hours into the night time
                //
                // night time is from 23:00 to 6:00 and one is not allowed
                let night_time_start = TimeStamp::new(23, 0).unwrap();
                let night_time_end = TimeStamp::new(6, 0).unwrap();
                let night_time = TimeSpan::new(night_time_start, night_time_end);

                if let Some(duration) = entry.time_span().overlapping_duration(&night_time) {
                    errors.push(InvalidTime::NightWork {
                        duration: duration.into(),
                        day,
                        time_span: entry.time_span(),
                        night_time,
                    });
                    continue;
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }
}
