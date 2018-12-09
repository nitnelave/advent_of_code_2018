#[macro_use]
extern crate nom;

use std::string::String;
use std::collections::HashMap;

mod parse;

/// Import the datatypes from the parser.
use crate::parse::{DatedEvent, GuardEvent, GuardId};
/// Import the trait so we can access .min() for `NaiveTime`.
use crate::parse::chrono::Timelike;

/// All the events of a given day.
#[derive(Debug, Eq, PartialEq, Default)]
struct DayEvent {
    /// Id of the guard.
    guard: GuardId,
    /// Times at which the guard fell asleep and woke up.
    sleep_times: Vec<(u32, u32)>,
}

/// Parse each line from the logs, and sort them by time.
fn parse_and_sort_lines(lines: &[String]) -> Vec<DatedEvent> {
    let mut result: Vec<DatedEvent> = lines.iter().map(|s| parse::parse_line(s)).collect();
    result.sort_by_key(|e| e.date);
    result
}

/// Summarize the events, grouping them by day (guard).
fn summarize_by_day(parsed_lines: Vec<DatedEvent>) -> Vec<DayEvent> {
    let mut day_events = Vec::new();
    let mut day_event = DayEvent::default();
    let mut asleep_time: Option<u32> = None;
    for event in parsed_lines {
        match event.event {
            GuardEvent::ShiftBegin(id) => {
                // We find a new guard, push the old one in the logs, unless
                // it's the initial, empty one.
                if day_event.guard != GuardId::default() {
                    day_events.push(day_event);
                }
                day_event = DayEvent::default();
                day_event.guard = id;
            }
            //
            GuardEvent::FallsAsleep => {
                // Record the time the guard fell asleep.
                assert!(asleep_time == None);
                asleep_time = Some(event.date.minute());
            }
            GuardEvent::WakesUp => {
                // Record the time the guard woke up, with the time he fell
                // asleep.
                assert!(asleep_time != None);
                day_event.sleep_times.push((
                    asleep_time.unwrap(),
                    event.date.minute(),
                ));
                asleep_time = None;
            }
        }
    }
    day_events
}

/// Summarize the events by guard.
fn summarize_by_guard(events: Vec<DayEvent>) -> HashMap<GuardId, Vec<DayEvent>> {
    let mut events_by_guard = HashMap::new();
    for event in events {
        match events_by_guard.get_mut(&event.guard) {
            None => {
                events_by_guard.insert(event.guard, vec![event]);
            }
            Some(v) => v.push(event),
        }
    }
    events_by_guard
}

/// Sum the duration of the naps of a single day.
fn time_slept_in_day(event: &DayEvent) -> u32 {
    event.sleep_times.iter().fold(0, |sum, (start, end)| {
        sum + end - start
    })
}

/// Sum the duration of all the given naps.
fn time_slept(events: &[DayEvent]) -> u32 {
    events.iter().map(time_slept_in_day).sum()
}

/// Find the guard who slept the longest overall.
fn find_sleepiest_guard(events_by_guard: &HashMap<GuardId, Vec<DayEvent>>) -> GuardId {
    // The ID is from the table, we know it's there.
    *events_by_guard
        .iter()
        .max_by_key(|(_id, events)| time_slept(events))
        .unwrap()
        .0
}

fn build_sleep_histogram(events: &[DayEvent]) -> [u32; 60] {
    let mut histogram = [0; 60];
    for event in events {
        for (nap_start, nap_end) in event.sleep_times.clone() {
            for i in nap_start..nap_end {
                histogram[i as usize] += 1;
            }
        }
    }
    histogram
}

/// Given a single guard, find the minute where he was most often asleep.
fn find_sleepiest_minute_for_guard(events: &[DayEvent]) -> (u32, u32) {
    let histogram = build_sleep_histogram(events);
    let (index, value) = histogram
        .iter()
        .enumerate()
        // Max count per minute.
        .max_by_key(|k| k.1)
        .unwrap();
    #[allow(clippy::cast_possible_truncation)]
    (index as u32, *value)
}

/// Find the guard that is the most likely to be sleeping at a specific minute,
/// along with the minute he is the most likely to be sleeping.
fn find_sleepiest_guard_at_minute(
    events_by_guard: &HashMap<GuardId, Vec<DayEvent>>,
) -> (&GuardId, u32) {
    let (guard, (minute, _count)) = events_by_guard
        .iter()
        // For each guard find the minute they are most likely to be sleeping.
        .map(|(g, events)| (g, find_sleepiest_minute_for_guard(events)))
        // Find the guard with the highest sleep value for their sleepiest
        // minute.
        .max_by_key(|(_g, (_min, count))| *count)
        .unwrap();
    (guard, minute)
}

/// Find the best guard and time to break in, with 2 strategies:
/// - Find the sleepiest guard, then the minute he is the most likely to be
/// asleep.
/// - Find the guard that is the most likely to be asleep at a given minute, and
/// that minute.
///
/// We return every time the product of the guard id and the minute in question.
///
/// The input is the lines of the log, unsorted.
pub fn find_guard_and_time(lines: &[String]) -> (u32, u32) {
    // Organize the logs.
    let parsed_lines = parse_and_sort_lines(lines);
    let day_events = summarize_by_day(parsed_lines);
    let guard_to_events = summarize_by_guard(day_events);

    // Strategy 1:
    let sleepiest_guard = find_sleepiest_guard(&guard_to_events);
    let sleepiest_minute =
        find_sleepiest_minute_for_guard(&guard_to_events[&sleepiest_guard]).0;
    println!(
        "{:?} slept the most at minute {}",
        sleepiest_guard,
        sleepiest_minute
    );

    // Strategy 2:
    let sleepiest_guard_at_minute = find_sleepiest_guard_at_minute(&guard_to_events);
    println!(
        "{:?} was the sleepiest guard at minute {}",
        sleepiest_guard_at_minute.0,
        sleepiest_guard_at_minute.1,
    );

    // Return the products of id * minute.
    (
        u32::from(sleepiest_guard) * sleepiest_minute,
        u32::from(*sleepiest_guard_at_minute.0) * sleepiest_guard_at_minute.1,
    )
}
