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
    padding: usize,
}

impl<W: Write> TerminalUI<W> {
    pub fn new(out: W) -> Self {
        Self { out, padding: 2 }
    }

    /// Left padding applied to every output line.
    pub fn set_padding(&mut self, padding: usize) {
        self.padding = padding;
    }

    fn pad(&mut self) {
        write!(self.out, "{:width$}", "", width = self.padding).unwrap();
    }

    pub fn menu(
        &mut self,
        title: &str,
        options: &[&str],
    ) -> Result<usize, Box<dyn std::error::Error>> {
        self.title(title);
        for (i, opt) in options.iter().enumerate() {
            self.pad();
            writeln!(self.out, "[{}] {}", i + 1, opt).unwrap();
        }
        self.pad();
        writeln!(self.out, "[{}] Cancel", options.len() + 1).unwrap();
        self.pad();
        write!(self.out, "Choice: ").unwrap();
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
            self.pad();
            writeln!(self.out, "{}", line.style(CYAN)).unwrap();
        }
    }

    fn divider(&mut self) {
        self.pad();
        writeln!(self.out, "{}", "─".repeat(DIVIDER_WIDTH)).unwrap();
    }

    fn title(&mut self, message: &str) {
        writeln!(self.out).unwrap();
        self.pad();
        writeln!(self.out, "{}", message.style(TITLE_STYLE)).unwrap();
    }

    fn error(&mut self, message: &str) {
        self.pad();
        writeln!(self.out, "{} {}", "✗".style(RED), message).unwrap();
    }

    fn info(&mut self, message: &str) {
        self.pad();
        writeln!(self.out, "{} {}", "→".style(CYAN), message).unwrap();
    }

    fn warn(&mut self, message: &str) {
        self.pad();
        writeln!(self.out, "{} {}", "⚠".style(YELLOW), message).unwrap();
    }

    fn success(&mut self, message: &str) {
        self.pad();
        writeln!(self.out, "{} {}", "✓".style(GREEN), message).unwrap();
    }

    fn muted(&mut self, message: &str) {
        self.pad();
        writeln!(self.out, "{}", message.style(DIM)).unwrap();
    }

    fn table(&mut self, rows: &[(&str, &str)]) {
        for (key, value) in rows {
            self.pad();
            writeln!(self.out, "{:<TABLE_KEY_WIDTH$} {}", key.style(DIM), value).unwrap();
        }
    }

    fn raw(&mut self, message: &str) {
        self.pad();
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

        assert_eq!(output, format!("  {}\n", "─".repeat(DIVIDER_WIDTH)));
    }

    #[test]
    fn padding_can_be_changed() {
        let mut buffer = Vec::new();
        {
            let mut ui = TerminalUI::new(&mut buffer);
            ui.set_padding(0);
            ui.success("Downloaded");
            ui.set_padding(4);
            ui.success("again");
        }

        let output = String::from_utf8(buffer).unwrap();
        // padding is plain spaces before the (possibly ANSI-styled) line
        let lead = |line: &str| line.len() - line.trim_start().len();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lead(lines[0]), 0);
        assert!(lines[0].contains("Downloaded"));
        assert_eq!(lead(lines[1]), 4);
        assert!(lines[1].contains("again"));
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
