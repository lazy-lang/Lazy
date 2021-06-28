
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

impl LoC {

    pub fn to_range(&self) -> Range {
        Range { start: self.clone(), end: self.clone()}
    }
}

#[derive(Copy, Default)]
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

pub enum ErrorLabelVariants {
    Primary,
    Secondary
}

pub struct ErrorLabel {
    msg: String,
    range: Range,
    variant: ErrorLabelVariants
}

impl ErrorLabel {
    
    pub fn new<T: Into<String>>(msg: T, range: Range) -> Self {
        ErrorLabel {
            msg: msg.into(),
            range,
            variant: ErrorLabelVariants::Secondary
        }
    }

    pub fn new_primary<T: Into<String>>(msg: T, range: Range) -> Self {
        ErrorLabel {
            msg: msg.into(),
            range,
            variant: ErrorLabelVariants::Primary
        }
    }

}

/* These errors are used only for the sterr output

    Error labels must:
    - Be between the main error range
*/

pub struct Error<T> where T: fmt::Display  {
    pub range: Range,
    pub msg: T,
    pub highlighted: bool,
    pub labels: Option<Vec<ErrorLabel>>
}

impl<T> Error<T> where T: fmt::Display {

    pub fn new(msg: T, range: Range) -> Error<T> {
        Error {
            msg,
            range: range,
            labels: None,
            highlighted: true,
        }
    }

    pub fn new_with_labels(msg: T, range: Range, labels: Vec<ErrorLabel>, highlighted: bool) -> Error<T> {
        Error {
            msg,
            range: range,
            labels: Some(labels),
            highlighted
        }
    }

}

pub trait ErrorCollector<T> where T: fmt::Display {
    fn error(&mut self, e_type: T, range: Range);
    fn error_lbl(&mut self, e_type: T, range: Range, labels: Vec<ErrorLabel>, highlight: bool);
}

impl<T> fmt::Debug for Error<T> where T: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
         .field("msg", &self.msg.to_string())
         .field("range", &self.range.to_string())
         .finish()
    }
}
