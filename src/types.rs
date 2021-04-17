pub use crate::bindings::{
    duckdb_column as DuckDBColumn, duckdb_connection, duckdb_database, duckdb_date, duckdb_hugeint,
    duckdb_interval, duckdb_result as DuckDBResult, duckdb_time, duckdb_timestamp, duckdb_type,
};
use libc::c_void;
use std::convert::TryInto;
use std::fmt::{Display, Error, Formatter};

impl Display for duckdb_interval {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("duckdb_interval")
            .field("months", &self.months)
            .field("days", &self.days)
            .field("micros", &self.micros)
            .finish()
    }
}
impl From<&duckdb_hugeint> for i128 {
    fn from(inst: &duckdb_hugeint) -> i128 {
        let sign = if inst.upper >= 0 { 1 } else { -1 };
        let upper = if sign == -1 { -inst.upper } else { inst.upper };

        let mut twisted: i128 = upper.into();
        let mut twisted: u128 = twisted.try_into().unwrap();
        twisted <<= 64;
        let step: u128 = inst.lower.into();
        twisted &= step;

        let twisted: i128 = twisted.try_into().unwrap();
        if sign == 1 {
            twisted
        } else {
            -twisted
        }
    }
}

impl Display for duckdb_hugeint {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let value: i128 = self.into();
        f.debug_struct("duckdb_hugeint")
            .field("value", &value)
            .finish()
    }
}

extern "C" {
    fn free(ptr: *const c_void);
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct duckdb_blob {
    data: *const c_void,
    pub size: u64,
}
impl Drop for duckdb_blob {
    fn drop(&mut self) {
        unsafe {
            free(self.data);
        };
    }
}
impl Display for duckdb_blob {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("duckdb_blob")
            .field("size", &self.size)
            .finish_non_exhaustive()
    }
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
