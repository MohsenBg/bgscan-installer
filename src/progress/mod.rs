use indicatif::{ProgressBar, ProgressStyle};

pub struct Progress {
    bar: ProgressBar,
}

impl Progress {
    pub fn new(total: u64) -> Self {
        let bar = ProgressBar::new(total);

        bar.set_style(
            ProgressStyle::with_template(
                "{msg}\
         {bar:40.cyan/blue} {percent:>3}%\n\
         Downloaded: {bytes}/{total_bytes}\n\
         Speed:      {bytes_per_sec}\n\
         Elapsed:    {elapsed_precise}\n\
         Remaining:  {eta_precise}",
            )
            .unwrap(),
        );

        Self { bar }
    }

    pub fn inc(&self, amount: u64) {
        self.bar.inc(amount);
    }

    pub fn finish(&self) {
        self.bar.finish();
    }
}
