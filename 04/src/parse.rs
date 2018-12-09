pub extern crate chrono;

use chrono::{NaiveDate, NaiveDateTime};
use nom::digit;
use std::str::FromStr;

/// Type-safe way of specifying a guard's ID.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Default)]
pub struct GuardId(u32);

/// Convert the `GuardId` to a u32.
impl From<GuardId> for u32 {
    fn from(g: GuardId) -> Self {
        g.0
    }
}

/// This represents one event of the log.
#[derive(Debug, PartialEq, Eq)]
pub enum GuardEvent {
    /// A guard start his shift (the ID is given).
    ShiftBegin(GuardId),
    WakesUp,
    FallsAsleep,
}

/// This represents one line of the log, with the date and the event.
#[derive(Debug, PartialEq, Eq)]
pub struct DatedEvent {
    pub date: NaiveDateTime,
    pub event: GuardEvent,
}

named!(i32 <&str, i32>,
   map!(map!(digit, FromStr::from_str), Result::unwrap)
);

named!(u32 <&str, u32>,
   map!(map!(digit, FromStr::from_str), Result::unwrap)
);

/// Parse a date from the log.
named!(date <&str, NaiveDateTime>,
       do_parse!(
           char!('[') >>
           year : i32 >>
           char!('-') >>
           month : u32 >>
           char!('-') >>
           day : u32 >>
           char!(' ') >>
           hour : u32 >>
           char!(':') >>
           minute : u32 >>
           char!(']') >>
           (NaiveDate::from_ymd(year, month, day).and_hms(hour, minute, 0))
));

/// Parse a shift change event.
named!(shift_begin <&str, GuardId>,
       do_parse!(
           tag!("Guard #") >>
           id: u32 >>
           tag!(" begins shift") >>
           (GuardId(id))
));

/// Parse an event from the log.
named!(guard_event <&str, GuardEvent>,
       alt!(
           tag!("falls asleep") => { |_| GuardEvent::FallsAsleep } |
           tag!("wakes up") => { |_| GuardEvent::WakesUp } |
           shift_begin => { |id| GuardEvent::ShiftBegin(id) }
));

/// Parse a line from the log.
named!(parse_event <&str, DatedEvent>,
       do_parse!(
           date: date >>
           char!(' ') >>
           event: guard_event >>
           ( DatedEvent { date: date, event: event} )
));

/// Parse a `line` from the log, panics if it fails.
#[allow(clippy::stutter)]
pub fn parse_line(line: &str) -> DatedEvent {
    parse_event(line).unwrap().1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i32() {
        assert_eq!(i32("123 ").unwrap().1, 123);
    }

    #[test]
    fn test_date() {
        let test_date = "[1518-11-03 00:31]";
        assert_eq!(
            format!("{}", date(test_date).unwrap().1.format("[%F %R]")),
            test_date
        );
    }

    #[test]
    fn test_event() {
        assert_eq!(
            guard_event("falls asleep").unwrap().1,
            GuardEvent::FallsAsleep
        );
        assert_eq!(guard_event("wakes up").unwrap().1, GuardEvent::WakesUp);
        assert_eq!(
            guard_event("Guard #132 begins shift").unwrap().1,
            GuardEvent::ShiftBegin(GuardId(132))
        );
    }

    #[test]
    fn test_parse_event() {
        assert_eq!(
            parse_event("[1518-09-01 23:56] Guard #1019 begins shift")
                .unwrap()
                .1,
            DatedEvent {
                date: NaiveDate::from_ymd(1518, 9, 1).and_hms(23, 56, 0),
                event: GuardEvent::ShiftBegin(GuardId(1019)),
            }
        );
    }
}
