//! Host conversion cost benchmarks (Criterion).
//!
//! **In scope:** nanoseconds per call on the machine that ran `cargo bench`; relative
//! change vs a previous run on the same host; default `std` + `earth` + `ccsds` profile.
//!
//! **Out of scope:** scientific residual (DUT1 interpolation, TDB model, ERA without polar
//! motion) — see `tests/erfa_golden.rs` and `tests/iers_ut1_golden.rs`; Cortex-M4F cycle
//! counts; CI pass/fail on ns/op; comparison vs other crates; heap allocation in format paths.

#![allow(missing_docs)] // Criterion's `criterion_group!` expands to undocumented `main`.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use satellite_datetime::ccsds::{encode_cuc, CucConfig};
use satellite_datetime::earth::{
    dut1, dut1_at, era_at_utc, format_rfc3339, parse_rfc3339, CivilUtc, UtcDay,
};
use satellite_datetime::{civil_from_unix_days, Duration, Instant};

/// Last MJD in the pinned C04 table (must match `src/earth/ut1_table.rs`).
const LAST_MJD: i32 = 61_045;

fn instant_at(y: i32, m: u8, d: u8, h: u8, min: u8, s: u8, ns: u32) -> Instant {
    CivilUtc::new(y, m, d, h, min, s, ns)
        .unwrap()
        .to_instant()
        .unwrap()
}

fn instant_from_mjd(mjd: i32) -> Instant {
    let unix = (mjd - 40_587) as i64;
    let (y, m, d) = civil_from_unix_days(unix);
    instant_at(y, m, d, 0, 0, 0, 0)
}

fn bench_instant_add(c: &mut Criterion) {
    let t = Instant::TAI_EPOCH;
    c.bench_function("instant_add", |b| {
        b.iter(|| black_box(t).checked_add(black_box(Duration::SECOND)))
    });
}

fn bench_utc_roundtrip(c: &mut Criterion) {
    let civil = CivilUtc::new(2017, 1, 1, 0, 0, 0, 0).unwrap();
    c.bench_function("utc_roundtrip", |b| {
        b.iter(|| {
            let inst = black_box(civil).to_instant().unwrap();
            black_box(inst).to_utc()
        })
    });
}

fn bench_utc_leap_day(c: &mut Criterion) {
    let civil = CivilUtc::new(2016, 12, 31, 23, 59, 60, 0).unwrap();
    c.bench_function("utc_leap_day", |b| {
        b.iter(|| {
            let inst = black_box(civil).to_instant().unwrap();
            black_box(inst).to_utc()
        })
    });
}

fn bench_dut1_sequential(c: &mut Criterion) {
    let base = instant_at(1999, 12, 1, 0, 0, 0, 0);
    let step = Duration::from_seconds(5 * 86_400);
    let seq: [Instant; 8] = core::array::from_fn(|i| {
        base.checked_add(step.checked_mul(i as i128).unwrap())
            .unwrap()
    });
    let mut idx = 0usize;
    c.bench_function("dut1_sequential", |b| {
        b.iter(|| {
            let inst = black_box(seq[idx % 8]);
            idx += 1;
            black_box(dut1(inst))
        })
    });
}

fn bench_dut1_scattered(c: &mut Criterion) {
    let scattered: [Instant; 4] = [
        instant_at(1972, 1, 1, 0, 0, 0, 0),
        instant_from_mjd(50_000),
        instant_from_mjd(55_000),
        instant_from_mjd(LAST_MJD),
    ];
    let mut idx = 0usize;
    c.bench_function("dut1_scattered", |b| {
        b.iter(|| {
            let inst = black_box(scattered[idx % 4]);
            idx += 1;
            black_box(dut1(inst))
        })
    });
}

fn bench_era_rad(c: &mut Criterion) {
    let t = instant_at(2010, 7, 24, 11, 18, 7, 318_000_000);
    c.bench_function("era_rad", |b| {
        b.iter(|| black_box(t).earth_rotation_angle_rad())
    });
}

fn bench_rfc3339_parse(c: &mut Criterion) {
    let s = "2010-07-24T11:18:07.318Z";
    c.bench_function("rfc3339_parse", |b| {
        b.iter(|| black_box(parse_rfc3339(black_box(s))))
    });
}

fn bench_rfc3339_format(c: &mut Criterion) {
    let t = instant_at(2010, 7, 24, 11, 18, 7, 318_000_000);
    let mut buf = [0u8; 40];
    c.bench_function("rfc3339_format", |b| {
        b.iter(|| black_box(format_rfc3339(black_box(t), black_box(&mut buf))))
    });
}

fn bench_dut1_at_civil(c: &mut Criterion) {
    let civil = CivilUtc::new(2010, 7, 24, 11, 18, 7, 318_000_000).unwrap();
    c.bench_function("dut1_at_civil", |b| {
        b.iter(|| black_box(dut1_at(black_box(civil))))
    });
}

fn bench_era_at_utc_civil(c: &mut Criterion) {
    let civil = CivilUtc::new(2010, 7, 24, 11, 18, 7, 318_000_000).unwrap();
    c.bench_function("era_at_utc_civil", |b| {
        b.iter(|| black_box(era_at_utc(black_box(civil))))
    });
}

fn bench_utc_day_civil_from_instant(c: &mut Criterion) {
    let day = UtcDay::new(2010, 7, 24).unwrap();
    let base = day.midnight_tai();
    let samples: [Instant; 8] =
        core::array::from_fn(|i| Instant::from_tai_nanos(base + (i as i128) * 3_600_000_000_000));
    let mut idx = 0usize;
    c.bench_function("utc_day_civil_from_instant", |b| {
        b.iter(|| {
            let inst = black_box(samples[idx % 8]);
            idx += 1;
            black_box(day.civil_from_instant(inst))
        })
    });
}

fn bench_cuc_c4f2(c: &mut Criterion) {
    let t = instant_at(2010, 7, 24, 11, 18, 7, 318_000_000);
    let mut buf = [0u8; 8];
    c.bench_function("cuc_c4f2", |b| {
        b.iter(|| {
            black_box(encode_cuc(
                black_box(t),
                black_box(CucConfig::C4_F2),
                black_box(&mut buf),
            ))
        })
    });
}

criterion_group!(
    conversions,
    bench_instant_add,
    bench_utc_roundtrip,
    bench_utc_leap_day,
    bench_dut1_sequential,
    bench_dut1_scattered,
    bench_era_rad,
    bench_rfc3339_parse,
    bench_rfc3339_format,
    bench_cuc_c4f2,
    bench_dut1_at_civil,
    bench_era_at_utc_civil,
    bench_utc_day_civil_from_instant,
);
criterion_main!(conversions);
