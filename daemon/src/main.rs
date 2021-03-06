extern crate chrono;

use std::path::Path;
use chrono::prelude::*;
use chrono::TimeZone;

mod files;

use std::io;
use std::{thread, time};

const THIS_WEEK_FILENAME: &'static str = "screen-time/week.txt";
const LAST_WEEK_FILENAME: &'static str = "screen-time/last-week.txt";

fn main() {
    start_daemon().unwrap();
}

fn start_daemon() -> io::Result<()> {
    let delay = 1000;
    println!("Logging screen time in the background.");

    sleep(delay);

    let mut last_log_timestamp = get_last_log(files::get_full_path(THIS_WEEK_FILENAME)?)?;

    loop {
        if should_archive(get_time(), last_log_timestamp){
            archive_log(files::get_full_path(THIS_WEEK_FILENAME)?, files::get_full_path(LAST_WEEK_FILENAME)?)?;
        }
        last_log_timestamp = log_screen_time(files::get_full_path(THIS_WEEK_FILENAME)?)?;
        sleep(delay);
    }
}

fn get_last_log<P: AsRef<Path>>(log_path: P) -> io::Result<i64> {
    let mut last_log_timestamp = get_time();
    let file_contents = files::read(log_path)?;
    let lines = file_contents.split("\n").collect::<Vec<&str>>();

    if lines.len() > 0 {
        // TODO: check to make sure first line is in the same week
        let mut last = lines[lines.len() - 1];
        if last.is_empty() && lines.len() > 1 {
            last = lines[lines.len() - 2];
        }

        if !last.is_empty() {
            last_log_timestamp = last.parse::<i64>().unwrap();
        }
    }

    Ok(last_log_timestamp)
}

fn archive_log<P: AsRef<Path>>(log_path: P, archive_path: P) -> io::Result<()> {
    files::archive(log_path, archive_path)?;
    Ok(())
}

fn log_screen_time<P: AsRef<Path>>(log_path: P) -> io::Result<i64> {
    let current_time = get_time();
    files::append(log_path, format!("{}\n", current_time).as_ref())?;
    Ok(current_time)
}


fn get_time() -> i64 {
    let dt = Local::now();
    dt.timestamp_millis()
}


fn sleep(millis: u64) {
    let timeout = time::Duration::from_millis(millis);
    thread::sleep(timeout);
}

#[test]
fn in_same_week_test() {
    let wed1_w1 = Local.ymd(2018, 1, 10).and_hms(1, 0, 0);
    let wed2_w1 = Local.ymd(2018, 1, 10).and_hms(2, 0, 0);
    let sat1_w1 = Local.ymd(2018, 1, 13).and_hms(2, 0, 0);
    let sun1_w2 = Local.ymd(2018, 1, 14).and_hms(2, 0, 0);
    let mon1_w3 = Local.ymd(2018, 1, 1).and_hms(0, 0, 0);
    let sun2_w3 = Local.ymd(2017, 12, 31).and_hms(23, 59, 59);
    let sat2_w4 = Local.ymd(2017, 12, 30).and_hms(23, 59, 59);

    // Same time
    assert_eq!(true, in_same_week(wed1_w1, wed1_w1));

    // Same day
    assert_eq!(true, in_same_week(wed1_w1, wed2_w1));
    assert_eq!(true, in_same_week(wed2_w1, wed1_w1));

    // Same week, different day
    assert_eq!(true, in_same_week(wed1_w1, sat1_w1));
    assert_eq!(true, in_same_week(sat1_w1, wed1_w1));

    // Different week
    assert_eq!(false, in_same_week(sat1_w1, sun1_w2));
    assert_eq!(false, in_same_week(sun1_w2, sat1_w1));

    // Same week, different year
    assert_eq!(true, in_same_week(mon1_w3, sun2_w3));
    assert_eq!(true, in_same_week(sun2_w3, mon1_w3));

    // Different week, different year
    assert_eq!(false, in_same_week(sat2_w4, sun2_w3));
    assert_eq!(false, in_same_week(sun2_w3, sat2_w4));
}

fn in_same_week(date1: DateTime<Local>, date2: DateTime<Local>) -> bool {
    let within_7_days = (date1.num_days_from_ce() - date2.num_days_from_ce()).abs() < 7;
    let newest_date = if date1 > date2 { date1 } else { date2 };
    let oldest_date = if newest_date == date1 { date2 } else { date1 };
    let sequential_weekdays = newest_date.weekday().number_from_sunday() >= oldest_date.weekday().number_from_sunday();
    within_7_days && sequential_weekdays
}

fn should_archive(current_time: i64, last_updated: i64) -> bool {
    let dt1 = Local.timestamp_millis(current_time);
    let dt2 = Local.timestamp_millis(last_updated);

    // less than 7 days apart and current_time is on a day of the week after last_updated
    !in_same_week(dt1, dt2)
}
