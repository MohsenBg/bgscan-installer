use owo_colors::{OwoColorize, Style};
use std::io::Write;

// Palette: a cooler, more "brand" cyan/violet pairing instead of flat bright_cyan
// everywhere, plus clearer semantic separation between info/warn/error/success.
static BRAND: Style = Style::new().bright_cyan();
static TITLE_STYLE: Style = Style::new().bold().bright_white();
static ACCENT: Style = Style::new().bright_magenta();
static GREEN: Style = Style::new().bright_green();
static RED: Style = Style::new().bright_red().bold();
static YELLOW: Style = Style::new().bright_yellow();
static DIM: Style = Style::new().dimmed();
static KEY: Style = Style::new().bright_blue();

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
    fn step(&mut self, n: usize, total: usize, message: &str);
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
            writeln!(self.out, "  {} {}", format!("{}", i + 1).style(ACCENT), opt).unwrap();
        }
        self.pad();
        writeln!(
            self.out,
            "  {} {}",
            format!("{}", options.len() + 1).style(DIM),
            "Cancel".style(DIM)
        )
        .unwrap();
        self.divider();
        self.pad();
        write!(self.out, "{} ", "❯".style(ACCENT)).unwrap();
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

        writeln!(self.out).unwrap();
        for line in BRAND_ART {
            self.pad();
            writeln!(self.out, "{}", line.style(BRAND)).unwrap();
        }
        self.pad();
    }

    fn divider(&mut self) {
        self.pad();
        writeln!(self.out, "{}", "─".repeat(DIVIDER_WIDTH)).unwrap();
    }

    fn title(&mut self, message: &str) {
        writeln!(self.out).unwrap();
        self.pad();
        writeln!(
            self.out,
            "{} {}",
            "▍".style(ACCENT),
            message.style(TITLE_STYLE)
        )
        .unwrap();
    }

    fn error(&mut self, message: &str) {
        self.pad();
        writeln!(self.out, "{} {}", "✗".style(RED), message).unwrap();
    }

    fn info(&mut self, message: &str) {
        self.pad();
        writeln!(self.out, "{} {}", "→".style(BRAND), message).unwrap();
    }

    fn warn(&mut self, message: &str) {
        self.pad();
        writeln!(self.out, "{} {}", "▲".style(YELLOW), message).unwrap();
    }

    fn success(&mut self, message: &str) {
        self.pad();
        writeln!(self.out, "{} {}", "✓".style(GREEN), message).unwrap();
    }

    fn muted(&mut self, message: &str) {
        self.pad();
        writeln!(self.out, "{}", message.style(DIM)).unwrap();
    }

    fn step(&mut self, n: usize, total: usize, message: &str) {
        self.pad();
        writeln!(
            self.out,
            "{} {}",
            format!("[{n}/{total}]").style(DIM),
            message
        )
        .unwrap();
    }

    fn table(&mut self, rows: &[(&str, &str)]) {
        for (key, value) in rows {
            self.pad();
            writeln!(
                self.out,
                "{:<TABLE_KEY_WIDTH$} {} {}",
                key.style(KEY),
                "│".style(DIM),
                value
            )
            .unwrap();
        }
    }

    fn raw(&mut self, message: &str) {
        self.pad();
        write!(self.out, "{}", message).unwrap();
    }
}
