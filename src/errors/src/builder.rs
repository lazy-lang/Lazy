
use colored::*;
use super::*;

fn get_digits_in_num(num: usize) -> usize {
    ((num as f32).log10() as usize) + 1
}

pub trait ErrorFormatter {
    fn get_file_contents(&self, file: &str) -> Option<&str>;

    fn format_err(&self, err: &BaseError, filename: &str) -> Option<String> {
        let space_to_border = get_digits_in_num(err.range.end.line) + 1; // + 1 because of the padding between the number and the border
        let mut res = format!("{}{} {} {}: {}\n", " ".repeat(space_to_border), "┎──".cyan(), filename, err.range, err.msg.to_string().red());
        let file_contents = self.get_file_contents(filename)?;
        let lines = file_contents.lines().collect::<Vec<&str>>();
        let cyan_wall = "┃".cyan();
        let red_arrow = &format!("{}", "^".red());
        for ind in err.range.start.line..=err.range.end.line {
            let line_text = lines[ind as usize - 1];
            let mut line = format!("{}{}{}   {}\n", ind, " ".repeat(space_to_border - get_digits_in_num(ind)), cyan_wall, line_text);
            if ind == err.range.start.line {
                let mut cols = format!("{} {}   ", " ".repeat(space_to_border - 1), cyan_wall);
                if err.range.start.col == err.range.end.col && err.range.start.line == err.range.end.line {
                    cols.push_str(&format!("{}{}", &" ".repeat(err.range.start.col), "^".red()));
                } else {
                for col in 0..=line_text.len() {
                    if col >= err.range.start.col && col < err.range.end.col { cols.push_str(&red_arrow); }
                    else { cols.push(' '); }
                    }
                }
                cols.push('\n');
                line.push_str(&cols);
            }
            if ind == err.range.end.line && err.range.start.line != err.range.end.line {
                let mut cols = format!("{} {}   ", " ".repeat(space_to_border - 1), cyan_wall);
                for col in 0..=line_text.len() {
                    if col >= err.range.end.col { cols.push_str(&red_arrow); }
                    else { cols.push(' '); }
                }
                cols.push('\n');
                line.push_str(&cols);
            }
        res.push_str(&line);
        }
        for label in &err.labels {
            match label.variant {
                ErrorLabelVariants::Help => {
                    res.push_str(&format!("{}", format!("{} {}   Help: {}", " ".repeat(space_to_border - 1), cyan_wall, label.msg).bright_black()))
                }
                ErrorLabelVariants::Sub(range) => {
                    let line_in_question = lines[range.start.line - 1];
                    let mut cols = format!("\n{} {}   ", " ".repeat(space_to_border - 1), cyan_wall);
                    for col in 0..=line_in_question.len() {
                        if col >= range.start.col && col < range.end.col { cols.push_str(&red_arrow); }
                        else { cols.push(' '); }
                    }
                    res.push_str(&format!("{}\n{} {}   {}{}", cyan_wall, cyan_wall, " ".repeat(space_to_border - 1), line_in_question, cols));
                }
            }
        }
        Some(res)
    }
}
