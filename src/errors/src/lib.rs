
pub mod builder;
use std::fmt;

#[derive(Copy)]
#[derive(Default)]
pub struct LoC {
    pub line: usize,
    pub col: usize,
    pub pos: usize
}

impl std::clone::Clone for LoC {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::fmt::Display for LoC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}:{})", self.line, self.col)
    }
}

#[derive(Copy)]
pub struct Range {
    pub start: LoC,
    pub end: LoC
}

impl std::clone::Clone for Range {
    fn clone(&self) -> Self {
        *self
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}:{} - {}:{})", self.start.line, self.start.col, self.end.line, self.end.col)
    }
}

pub struct ErrorLabel(String, Range);

// These errors are used only for the sterr output
pub struct Error<T> where T: fmt::Display  {
    pub range: Range,
    pub msg: T,
    pub labels: Option<Vec<ErrorLabel>>
}

impl<T> Error<T> where T: fmt::Display {

    pub fn new(msg: T, range: Range) -> Error<T> {
        Error {
            msg,
            range: range,
            labels: None
        }
    }

    pub fn new_with_labels(msg: T, range: Range, labels: Vec<ErrorLabel>) -> Error<T> {
        Error {
            msg,
            range: range,
            labels: Some(labels)
        }
    }

}

