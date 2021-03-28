use std::fmt::{Display, Error, Formatter};

#[repr(C)]
#[derive(Debug)]
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
            "{:0<4}-{:0<2}-{:0<2}",
            self.year, self.month, self.day
        ))
    }
}

#[repr(C)]
#[derive(Debug)]
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
            "{:0<2}:{:0<2}:{:0<2}.{}",
            self.hour, self.min, self.sec, self.micros
        ))
    }
}

#[repr(C)]
#[derive(Debug)]
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
