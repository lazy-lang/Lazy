pub mod builder;
pub mod diagnostics;
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

    pub fn end(self, other: &LoC) -> Range {
        Range { start: self, end: other.clone() }
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

impl Range {

    pub fn end_with(self, end: &LoC) -> Range {
        Range { start: self.start, end: end.clone() }
    }
}

pub enum ErrorLabelVariants {
    Primary,
    Secondary
}

pub struct ErrorLabel {
    pub msg: String,
    pub range: Range,
    pub variant: ErrorLabelVariants
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
    pub filename: String,
    pub labels: Vec<ErrorLabel>
}

impl Error {

    pub fn new(msg: String, range: Range, filename: String) -> Error {
        Error {
            msg,
            range: range,
            labels: vec![],
            highlighted: true,
            filename
        }
    }

    pub fn new_with_labels(msg: String, range: Range, filename: String, labels: Vec<ErrorLabel>) -> Error {
        Error {
            msg,
            range: range,
            labels,
            highlighted: true,
            filename
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

#[macro_export]
macro_rules! err {
    ($diagnostic: ident, $range: expr, $filename: expr, $($vars: expr),*; $([$label_text: expr, $label_range: expr]),*) => {
            Error {
                msg: format_diagnostic(&Diagnostics::$diagnostic, vec![$($vars),*]),
                highlighted: true,
                range: $range,
                filename: $filename.to_string(),
                labels: vec![$(
                    ErrorLabel {
                        msg: String::from($label_text),
                        variant: ErrorLabelVariants::Secondary,
                        range: $label_range
                    }
                ),*]
            }
    };
    ($diagnostic: ident, $range: expr, $filename: expr, $($vars: expr),*) => {
        Error {
            msg: format_diagnostic(&Diagnostics::$diagnostic, vec![$($vars),*]),
            highlighted: true,
            filename: $filename.to_string(),
            range: $range,
            labels: vec![]
        }
    };
    ($diagnostic: ident, $range: expr, $filename: expr) => {
        Error {
            msg: Diagnostics::$diagnostic.message.to_string(),
            highlighted: true,
            filename: $filename.to_string(),
            range: $range,
            labels: vec![]
        }
    }
}

#[macro_export]
macro_rules! dia {
    ($diagnostic: ident, $($vars: expr),*) => {
        format_diagnostic(&Diagnostics::$diagnostic, vec![$($vars),*])
    } 
}