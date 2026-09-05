//! Parse a UTC leap second from RFC 3339.

fn main() {
    let leap = satellite_datetime::parse_rfc3339("2016-12-31T23:59:60Z").expect("leap second");
    let civil = leap.to_utc().expect("utc");
    println!(
        "UTC {year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} (TAI ns {})",
        leap.as_tai_nanos(),
        year = civil.year,
        month = civil.month,
        day = civil.day,
        hour = civil.hour,
        minute = civil.minute,
        second = civil.second,
    );
}
