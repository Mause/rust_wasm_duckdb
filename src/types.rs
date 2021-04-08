use libc::c_void;
use std::fmt::{Display, Error, Formatter};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct duckdb_interval {
    months: i32,
    days: i32,
    micros: i64,
}
impl Display for duckdb_interval {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("duckdb_interval")
            .field("months", &self.months)
            .field("days", &self.days)
            .field("micros", &self.micros)
            .finish()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct duckdb_hugeint {
    lower: u64,
    upper: i64,
}
impl Display for duckdb_hugeint {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use std::convert::TryInto;

        let sign = if self.upper >= 0 { 1 } else { -1 };
        let upper = if sign == -1 { -self.upper } else { self.upper };

        let mut twisted: i128 = upper.into();
        let mut twisted: u128 = twisted.try_into().unwrap();
        twisted <<= 64;
        let step: u128 = self.lower.into();
        twisted &= step;

        f.debug_struct("duckdb_hugeint")
            .field("value", &twisted)
            .finish()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct duckdb_blob {
    data: *const c_void,
    pub size: i64,
}
impl Display for duckdb_blob {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("duckdb_blob")
            .field("size", &self.size)
            .finish_non_exhaustive()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct duckdb_date {
    year: i32,
    month: i8,
    day: i8,
}
impl duckdb_date {
    pub fn new(year: i32, month: i8, day: i8) -> Self {
        Self { year, month, day }
    }
}
impl Display for duckdb_date {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_fmt(format_args!(
            "{:0>4}-{:0>2}-{:0>2}",
            self.year, self.month, self.day
        ))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct duckdb_time {
    hour: i8,
    min: i8,
    sec: i8,
    micros: i16,
}
impl duckdb_time {
    pub fn new(hour: i8, min: i8, sec: i8, micros: i16) -> Self {
        Self {
            hour,
            min,
            sec,
            micros,
        }
    }
}
impl Display for duckdb_time {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_fmt(format_args!(
            "{:0>2}:{:0>2}:{:0>2}.{}",
            self.hour, self.min, self.sec, self.micros
        ))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct duckdb_timestamp {
    date: duckdb_date,
    time: duckdb_time,
}
impl duckdb_timestamp {
    pub fn new(date: duckdb_date, time: duckdb_time) -> Self {
        Self { date, time }
    }
}
impl Display for duckdb_timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_fmt(format_args!("{}T{}", self.date, self.time))
    }
}
