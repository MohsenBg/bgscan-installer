use indicatif::{ProgressBar, ProgressStyle};

pub struct Progress {
    bar: ProgressBar,
}

impl Progress {
    pub fn new(total: u64) -> Self {
        let bar = ProgressBar::new(total);

        bar.set_style(
            ProgressStyle::with_template(concat!(
                "\n",
                "  {spinner:.green} {msg:.bold}\n",
                "  {bar:40.green/black} {percent:>3}%\n",
                "  {bytes:>10.yellow} / {total_bytes:<10.yellow}",
                "   {bytes_per_sec:.cyan}\n",
                "  ⏱  {elapsed_precise:.dim}   ⏳ {eta_precise:.dim}\n",
            ))
            .unwrap()
            .progress_chars("█▓░")
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );

        bar.enable_steady_tick(std::time::Duration::from_millis(80));

        Self { bar }
    }

    pub fn inc(&self, amount: u64) {
        self.bar.inc(amount);
    }

    pub fn finish(&self) {
        self.bar.finish_with_message("done ✔");
    }
}
