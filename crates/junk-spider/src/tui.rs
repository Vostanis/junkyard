use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

pub(crate) fn multi_progress(
    len: usize,
) -> anyhow::Result<(
    Option<MultiProgress>,
    Option<ProgressBar>,
    Option<ProgressBar>,
    Option<ProgressBar>,
)> {
    // overall multi progress bar
    let multi = MultiProgress::new();

    // total number of tickers to collect
    let total = multi.add(
        ProgressBar::new(len as u64).with_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.magenta}\n \
                        {msg:>9.white} |{bar:57.white/grey}| {pos:<2} / {human_len} \
                        ({percent_precise}%) [Time: {elapsed}, Rate: {per_sec}, ETA: {eta}]",
                )?
                .progress_chars("## "),
        ),
    );
    total.set_message("total");
    total.enable_steady_tick(Duration::from_millis(100));

    // total successful collections
    let success = multi.insert_after(
        &total,
        ProgressBar::new(len as u64).with_style(
            ProgressStyle::default_bar()
                .template(" {msg:>9.green} |{bar:57.green}| {pos:<2.green}")?
                .progress_chars("## "),
        ),
    );
    success.set_message("successes");

    // total failed collections
    let fails = multi.insert_after(
        &success,
        ProgressBar::new(len as u64).with_style(
            ProgressStyle::default_bar()
                .template(" {msg:>9.red} |{bar:57.red}| {pos:<2.red}")?
                .progress_chars("## "),
        ),
    );
    fails.set_message("failures");

    Ok((Some(multi), Some(total), Some(success), Some(fails)))
}

pub(crate) fn multi_progress_spinner(multi: Option<MultiProgress>, msg: String) -> ProgressBar {
    match multi {
        Some(m) => m.add(
            ProgressBar::new_spinner().with_message(msg).with_style(
                ProgressStyle::default_spinner()
                    .template("\t   > {msg}")
                    .expect("failed to set spinner style"),
            ),
        ),
        None => ProgressBar::hidden(),
    }
}
