use std::fmt;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug)]
#[repr(C)]
pub enum DuckDBState {
    DuckDBSuccess = 0,
    DuckDBError = 1,
}
impl DuckDBState {
    #[allow(dead_code)]
    fn is_success(&self) -> bool {
        matches!(self, DuckDBState::DuckDBSuccess)
    }
}
impl fmt::Display for DuckDBState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?})", self)
    }
}
impl std::error::Error for DuckDBState {}
impl std::ops::Try for DuckDBState {
    type Ok = DuckDBState;
    type Error = DuckDBState;

    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        match &self {
            DuckDBState::DuckDBSuccess => Ok(self),
            DuckDBState::DuckDBError => Err(self),
        }
    }
    fn from_ok(_: <Self as std::ops::Try>::Ok) -> Self {
        todo!()
    }
    fn from_error(_: <Self as std::ops::Try>::Error) -> Self {
        todo!()
    }
}
