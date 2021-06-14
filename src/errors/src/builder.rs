
use colored::*;
use super::*;
use std::collections::HashMap;


fn get_digits_in_num(num: usize) -> usize {
    ((num as f32).log10() as usize) + 1
}

pub struct ErrorFormatter<'a> {
    pub files: HashMap<String, Vec<&'a str>>
}


impl<'a> ErrorFormatter<'a> {
    
    pub fn new() -> Self {
        ErrorFormatter { 
            files: HashMap::new()
        }
    }

    pub fn add(&mut self, source: String, content: &'a String) {
        self.files.insert(source, content.lines().collect::<Vec<&str>>());
    }

    pub fn print_err<T: Into<String>, E: fmt::Display>(&self, source: T, err: &Error<E>) -> Option<String> {
        let space_to_border = get_digits_in_num(err.range.end.line) + 1; // + 1 because of the padding between the number and the border
        let real_source = source.into();
        let mut res = format!("{}{} {} {}: {}\n", " ".repeat(space_to_border), "┎──".cyan(), real_source, err.range, err.msg.to_string().red());
        let lines = self.files.get(&real_source)?;
        let cyan_wall = "┃".cyan();
        for ind in err.range.start.line..=err.range.end.line {
            let line_text = lines[ind as usize - 1];
            let mut line = format!("{}{}{}   {}\n", ind, " ".repeat(space_to_border - get_digits_in_num(ind)), cyan_wall, line_text);
            if err.highlighted {
            if ind == err.range.start.line {
                let mut cols = format!("{} {}   ", " ".repeat(space_to_border - 1), cyan_wall);
                for col in 0..=line_text.len() {
                    if col >= err.range.start.col && col < err.range.end.col { cols.push_str(&format!("{}", "^".red())); }
                    else { cols.push(' '); }
                }
                cols.push('\n');
                line.push_str(&cols);
            }
            if ind == err.range.end.line && err.range.start.line != err.range.end.line {
                let mut cols = format!("{} {}   ", " ".repeat(space_to_border - 1), cyan_wall);
                for col in 0..=line_text.len() {
                    if col >= err.range.end.col { cols.push_str(&format!("{}", "^".red())); }
                    else { cols.push(' '); }
                }
                cols.push('\n');
                line.push_str(&cols);
            }
        }
            if let Some(labels) = &err.labels {
                if let Some(lbl) = labels.iter().find(|i| i.range.start.line == ind) {
                    let mut col = format!("{} {}   ", " ".repeat(space_to_border - 1), cyan_wall);
                    let text_variant = match lbl.variant {
                        ErrorLabelVariants::Primary => ("^".red(), lbl.msg.red()),
                        ErrorLabelVariants::Secondary => ("─".bright_black(), lbl.msg.bright_black())
                    };
                    for col_ind in 0..=line_text.len() {
                        if col_ind >= lbl.range.start.col && col_ind <= lbl.range.end.col { col.push_str(&format!("{}", text_variant.0)); }
                        else { col.push(' '); }
                    }
                    col.push_str(&format!("{}\n", text_variant.1));
                    line.push_str(&col);
                }
            };
            res.push_str(&line);
        } 
        Some(res)
    }

}




