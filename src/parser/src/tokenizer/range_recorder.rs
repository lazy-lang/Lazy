

use super::*;

pub struct RangeRecorder {
    pub start: LoC,
}

impl RangeRecorder {
    
    pub fn new(tok: &Tokenizer) -> Self {
        RangeRecorder {
            start: tok.input.loc(),
        }
    }

    #[inline]
    pub fn end(&self, tok: &Tokenizer) -> Range {
        Range {
            start: self.start,
            end: tok.last_loc
        }
    }

    #[inline]
    pub fn end_with(&self, loc: LoC) -> Range {
        Range {
            start: self.start,
            end: loc
        }
    }

    #[inline]
    pub fn err(&self, err: ParserErrorType, tok: &mut Tokenizer) {
        tok.error(err, Range { start: self.start, end: tok.last_loc})
    }

    #[inline]
    pub fn err_lbl(&self, e_type: ParserErrorType, labels: Vec<ErrorLabel>, tok: &mut Tokenizer) {
        tok.error_lbl(e_type, Range { start: self.start, end: tok.last_loc }, labels)
    }
    
}