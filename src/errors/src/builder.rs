
use colored::*;
use super::*;

fn get_digits_in_num(num: usize) -> usize {
    ((num as f32).log10() as usize) + 1
}

pub trait ErrorFormatter {
    fn get_file_contents(&self, file: &str) -> Option<&str>;

    fn format_err(&self, err: &Error, filename: &str) -> Option<String> {
        let space_to_border = get_digits_in_num(err.range.end.line) + 1; // + 1 because of the padding between the number and the border
        let mut res = format!("{}{} {} {}: {}\n", " ".repeat(space_to_border), "┎──".cyan(), filename, err.range, err.msg.to_string().red());
        let file_contents = self.get_file_contents(filename)?;
        let mut lines = file_contents.lines();
        let cyan_wall = "┃".cyan();
        for ind in err.range.start.line..=err.range.end.line {
            let line_text = lines.nth(ind as usize - 1)?;
            let mut line = format!("{}{}{}   {}\n", ind, " ".repeat(space_to_border - get_digits_in_num(ind)), cyan_wall, line_text);
            if err.highlighted {
            if ind == err.range.start.line {
                let mut cols = format!("{} {}   ", " ".repeat(space_to_border - 1), cyan_wall);
                if err.range.start.col == err.range.end.col && err.range.start.line == err.range.end.line {
                    cols.push_str(&" ".repeat(err.range.start.col));
                    cols.push_str(&format!("{}", "^".red()));
                } else {
                for col in 0..=line_text.len() {
                    if col >= err.range.start.col && col < err.range.end.col { cols.push_str(&format!("{}", "^".red())); }
                    else { cols.push(' '); }
                    }
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
            if let Some(lbl) = err.labels.iter().find(|i| i.range.start.line == ind) {
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
        res.push_str(&line);
    } 
        Some(res)
    }
}
