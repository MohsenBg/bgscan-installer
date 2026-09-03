use owo_colors::{OwoColorize, Style};
use std::io::Write;

static CYAN: Style = Style::new().bright_cyan();
static TITLE_STYLE: Style = Style::new().bright_cyan().bold();
static GREEN: Style = Style::new().bright_green();
static RED: Style = Style::new().bright_red();
static YELLOW: Style = Style::new().bright_yellow();
static DIM: Style = Style::new().dimmed();

const DIVIDER_WIDTH: usize = 52;
const TABLE_KEY_WIDTH: usize = 15;

pub trait UI {
    fn brand(&mut self);
    fn divider(&mut self);

    fn title(&mut self, message: &str);
    fn error(&mut self, message: &str);
    fn info(&mut self, message: &str);
    fn warn(&mut self, message: &str);
    fn success(&mut self, message: &str);
    fn muted(&mut self, message: &str);
    fn raw(&mut self, message: &str);

    fn table(&mut self, rows: &[(&str, &str)]);
}

pub struct TerminalUI<W: Write> {
    out: W,
}

impl<W: Write> TerminalUI<W> {
    pub fn new(out: W) -> Self {
        Self { out }
    }

    pub fn menu(
        &mut self,
        title: &str,
        options: &[&str],
    ) -> Result<usize, Box<dyn std::error::Error>> {
        writeln!(self.out, "\n{}", title).unwrap();
        for (i, opt) in options.iter().enumerate() {
            writeln!(self.out, "   [{}] {}", i + 1, opt).unwrap();
        }
        writeln!(self.out, "   [{}] Cancel", options.len() + 1).unwrap();
        write!(self.out, "   Choice: ").unwrap();
        self.out.flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let choice: usize = input
            .trim()
            .parse()
            .map_err(|_| format!("invalid choice: '{input}'", input = input.trim()))?;

        if choice == 0 || choice > options.len() + 1 {
            return Err(format!("choice out of range: {choice}").into());
        }

        Ok(choice)
    }
}

impl<W: Write> UI for TerminalUI<W> {
    fn brand(&mut self) {
        const BRAND_ART: [&str; 6] = [
            " ██████╗  ██████╗    ███████╗ ██████╗ █████╗ ███╗   ██╗",
            " ██╔══██╗██╔════╝    ██╔════╝██╔════╝██╔══██╗████╗  ██║",
            " ██████╔╝██║  ███╗   ███████╗██║     ███████║██╔██╗ ██║",
            " ██╔══██╗██║   ██║   ╚════██║██║     ██╔══██║██║╚██╗██║",
            " ██████╔╝╚██████╔╝   ███████║╚██████╗██║  ██║██║ ╚████║",
            " ╚═════╝  ╚═════╝    ╚══════╝╚═════╝╚═╝   ╚═╝╚═╝  ╚═══╝",
        ];

        for line in BRAND_ART {
            writeln!(self.out, "{}", line.style(CYAN)).unwrap();
        }
    }

    fn divider(&mut self) {
        writeln!(self.out, "{}", "─".repeat(DIVIDER_WIDTH)).unwrap();
    }

    fn title(&mut self, message: &str) {
        writeln!(self.out, "\n{}", message.style(TITLE_STYLE)).unwrap();
    }

    fn error(&mut self, message: &str) {
        writeln!(self.out, "{} {}", "✗".style(RED), message).unwrap();
    }

    fn info(&mut self, message: &str) {
        writeln!(self.out, "{} {}", "→".style(CYAN), message).unwrap();
    }

    fn warn(&mut self, message: &str) {
        writeln!(self.out, "{} {}", "⚠".style(YELLOW), message).unwrap();
    }

    fn success(&mut self, message: &str) {
        writeln!(self.out, "{} {}", "✓".style(GREEN), message).unwrap();
    }

    fn muted(&mut self, message: &str) {
        writeln!(self.out, "{}", message.style(DIM)).unwrap();
    }

    fn table(&mut self, rows: &[(&str, &str)]) {
        for (key, value) in rows {
            writeln!(self.out, "{:<TABLE_KEY_WIDTH$} {}", key.style(DIM), value).unwrap();
        }
    }

    fn raw(&mut self, message: &str) {
        write!(self.out, "{}", message).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output<F>(render: F) -> String
    where
        F: FnOnce(&mut TerminalUI<&mut Vec<u8>>),
    {
        let mut buffer = Vec::new();
        let mut ui = TerminalUI::new(&mut buffer);

        render(&mut ui);

        String::from_utf8(buffer).unwrap()
    }

    #[test]
    fn divider_prints_expected_width() {
        let output = output(|ui| ui.divider());

        assert_eq!(output, format!("{}\n", "─".repeat(DIVIDER_WIDTH)));
    }

    #[test]
    fn success_prints_icon_and_message() {
        let output = output(|ui| ui.success("Downloaded"));

        assert!(output.contains("✓"));
        assert!(output.contains("Downloaded"));
    }

    #[test]
    fn title_prints_blank_line_and_message() {
        let output = output(|ui| ui.title("Download"));

        assert!(output.starts_with('\n'));
        assert!(output.contains("Download"));
    }

    #[test]
    fn table_prints_key_value_rows() {
        let output = output(|ui| {
            ui.table(&[("os", "linux"), ("arch", "x86_64")]);
        });

        assert!(output.contains("os"));
        assert!(output.contains("linux"));
        assert!(output.contains("arch"));
        assert!(output.contains("x86_64"));
    }
}
