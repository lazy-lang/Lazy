pub mod builder;
pub mod diagnostics;
use std::fmt;

use diagnostics;

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

pub struct Error {
    pub range: Range,
    pub msg: String,
    pub highlighted: bool,
    pub labels: Vec<ErrorLabel>
}

impl Error {

    pub fn new(msg: String, range: Range) -> Error {
        Error {
            msg,
            range: range,
            labels: vec![],
            highlighted: true
        }
    }

    pub fn new_with_labels(msg: String, range: Range, labels: Vec<ErrorLabel>) -> Error {
        Error {
            msg,
            range: range,
            labels,
            highlighted: true
        }
    }

}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
         .field("msg", &self.msg.to_string())
         .field("range", &self.range.to_string())
         .finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Error: {}", self.msg.to_string())
    }
}

impl std::error::Error for Error {}

pub type LazyResult<T> = Result<T, Error>;
pub type LazyMultiResult<T> = Result<T, Vec<Error>>;

macro_rules! err {
    ($diagnostic: expr, $range: expr, $($vars: expr),*; $([$label_text: expr, $label_range: expr]),*) => {
        {
            let dia_str = diagnostic::format_diagnostic($diagnostic, vec![$($vars),*]);
            let labels = vec![$(
                ErrorLabel {
                    msg: $label_text,
                    variant: ErrorLabelVariants::Secondary,
                    range: $label_range
                }
            ),*];
            Error {
                msg: dia_str,
                highlighted: true,
                range: $range,
                labels
            }
        }
    }
}

fn a() {
    err!(Diagnostics::UNEXPECTED_OP, 1..4, "0");
}