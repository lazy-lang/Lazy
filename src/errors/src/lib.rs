pub mod builder;
pub mod diagnostics;
use std::fmt;

pub use diagnostics::*;
pub use builder::*;

#[derive(Copy, Default)]
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
    Help,
    Sub(Range)
}

pub struct ErrorLabel {
    pub msg: String,
    pub variant: ErrorLabelVariants
}

/* These errors are used only for the sterr output

    Error labels must:
    - Be between the main error range
*/

pub struct BaseError {
    pub range: Range,
    pub msg: String,
    pub labels: Vec<ErrorLabel>
}

impl BaseError {

    pub fn new(msg: String, range: Range) -> Self {
        Self {
            msg,
            range: range,
            labels: vec![]
        }
    }

    pub fn new_with_labels(msg: String, range: Range, labels: Vec<ErrorLabel>) -> Self {
        Self {
            msg,
            range: range,
            labels
        }
    }

}

impl fmt::Debug for BaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
         .field("msg", &self.msg.to_string())
         .field("range", &self.range.to_string())
         .finish()
    }
}

impl fmt::Display for BaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Error: {}", self.msg.to_string())
    }
}

impl std::error::Error for BaseError {}

pub type LazyResult<T> = Result<T, BaseError>;
pub type LazyMultiResult<T> = Result<T, ErrorCollector>;

pub struct ErrorCollector {
    pub collected: Vec<BaseError>,
    pub filename: String 
} 

impl ErrorCollector {
    pub fn new(filename: &str) -> Self {
        ErrorCollector {
            collected: vec![],
            filename: filename.to_string()
        }
    }

    pub fn push(&mut self, err: BaseError) {
        self.collected.push(err);
    }

}

#[macro_export]
macro_rules! err {
    ($diagnostic: ident, $range: expr, $($vars: expr),*; $([$label_text: expr, $label_range: expr]),*) => {
            BaseError {
                msg: format_diagnostic(&Diagnostics::$diagnostic, vec![$($vars),*]),
                range: $range,
                labels: vec![$(
                    ErrorLabel {
                        msg: String::from($label_text),
                        variant: ErrorLabelVariants::Sub($label_range)
                    }
                ),*]
            }
    };
    ($diagnostic: ident, $range: expr, $($vars: expr),*; $([$label_text: expr]),*) => {
        BaseError {
            msg: format_diagnostic(&Diagnostics::$diagnostic, vec![$($vars),*]),
            range: $range,
            labels: vec![$(
                ErrorLabel {
                    msg: String::from($label_text),
                    variant: ErrorLabelVariants::Help,
                }
            ),*]
        }
    };
    ($diagnostic: ident, $range: expr, $($vars: expr),*) => {
        BaseError {
            msg: format_diagnostic(&Diagnostics::$diagnostic, vec![$($vars),*]),
            range: $range,
            labels: vec![]
        }
    };
    ($diagnostic: ident, $range: expr) => {
        BaseError {
            msg: Diagnostics::$diagnostic.message.to_string(),
            range: $range,
            labels: vec![]
        }
    };
    ($diagnostic: expr, $range: expr) => {
        BaseError {
            msg: $diagnostic,
            range: $range,
            labels: vec![]
        }
    }
}

#[macro_export]
macro_rules! dia {
    ($diagnostic: ident, $($vars: expr),*) => {
        format_diagnostic(&Diagnostics::$diagnostic, vec![$($vars),*])
    };
    ($diagnostic: ident) => {
        format_diagnostic(&Diagnostics::$diagnostic, vec![])
    }
}