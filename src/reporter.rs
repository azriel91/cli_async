use std::{fmt, fmt::Write as _, io, io::Write as _};

use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::mpsc::{Receiver, UnboundedReceiver};

use crate::{Colours, PropertyInfoResult, Report};

#[derive(Debug)]
pub struct Reporter {
    /// `ProgressBar` for the overall progress.
    progress_overall: ProgressBar,
    /// Receiver to receive updates when a record is processed.
    progress_receiver: UnboundedReceiver<PropertyInfoResult>,
    /// Process report of records.
    report: Report,
    /// Interrupt handler.
    interrupt_rx: Option<Receiver<()>>,
    /// Whether this reporter has been interrupted.
    interrupted: bool,
}

impl Reporter {
    pub fn new(
        record_count: u64,
        record_count_processed: u64,
        progress_receiver: UnboundedReceiver<PropertyInfoResult>,
        show_progress: bool,
        interrupt_rx: Option<Receiver<()>>,
    ) -> Self {
        // Can't support `MultiProgress`: <https://github.com/mitsuhiko/indicatif/issues/125>

        let progress_overall = if show_progress {
            ProgressBar::new(record_count)
        } else {
            ProgressBar::hidden()
        };
        progress_overall.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .progress_chars("█▒░"),
        );
        progress_overall.set_position(record_count_processed);

        let report = Report {
            record_skipped_count: record_count_processed as usize,
            ..Default::default()
        };

        Self {
            progress_overall,
            progress_receiver,
            report,
            interrupt_rx,
            interrupted: false,
        }
    }

    /// Writes the logo to stderr.
    ///
    /// The logo should be a stylized:
    ///
    /// ```text
    ///              _   _ _   _
    ///  ___ ___| |_|_| |_| |___
    /// | . |_ -|  _| |  _| | -_|
    /// |  _|___|_| |_|_| |_|___|
    /// |_|
    /// ```
    pub fn print_logo() -> crossterm::Result<()> {
        let logo_left = ["    ", " ___ ___", "| . |_ -", "|  _|___", "|_|     ", ""];
        let logo_right = [
            "     _   _ _   _",
            "| |_|_| |_| |___",
            "|  _| |  _| | -_|",
            "|_| |_|_| |_|___|",
            "",
            "",
        ];
        let prompt = logo_left
            .iter()
            .zip(logo_right.iter())
            .try_fold(String::with_capacity(384), |mut buffer, (left, right)| {
                let left = Colours::LOGO_LEFT.apply(left);
                let right = Colours::LOGO_RIGHT.apply(right);

                write!(&mut buffer, "{}", left)?;
                writeln!(&mut buffer, "{}", right)?;

                Result::<String, fmt::Error>::Ok(buffer)
            })
            .expect("Failed to construct logo.");
        let mut stderr = io::stderr();
        stderr.write_all(prompt.as_bytes())?;
        stderr.flush()?;

        Ok(())
    }

    pub fn progress_bar_startup(&mut self) {}

    /// Synchronizes the progress bar with the state of processing.
    pub async fn progress_bar_sync(&mut self) {
        if let Some(mut interrupt_rx) = self.interrupt_rx.take() {
            tokio::select! {
                () = self.progress_bar_sync_internal() => {
                    self.progress_overall.finish();
                },
                _ = interrupt_rx.recv() => {},
            }

        // Empty remaining queue.
        // self.progress_bar_sync_internal().await;
        } else {
            self.progress_bar_sync_internal().await;
            self.progress_overall.finish();
        }
    }

    async fn progress_bar_sync_internal(&mut self) {
        while let Some(process_result) = self.progress_receiver.recv().await {
            match process_result {
                PropertyInfoResult::Success => {
                    self.report.record_processed_successful_count += 1;
                }
                PropertyInfoResult::SuccessPartial => {
                    self.report.record_processed_info_missing_count += 1;
                }
                PropertyInfoResult::Error(record, error) => {
                    self.report.records_processed_failed.push((record, error));
                }
            }
            self.progress_overall.inc(1);
        }
    }

    /// Writes the report to stderr.
    pub fn print_report(&self) -> crossterm::Result<()> {
        let self_report = &self.report;
        let failed_count = self_report.records_processed_failed.len();

        let mut report = String::with_capacity(1024);
        writeln!(&mut report)?;
        writeln!(
            &mut report,
            "{}",
            Colours::REPORT_BORDER
                .apply("------------------------------------------------------------")
        )?;

        writeln!(&mut report, "{}", Colours::REPORT_TITLE.apply("# Report"))?;
        writeln!(&mut report)?;

        writeln!(&mut report, "{}", Colours::REPORT_TITLE.apply("## Summary"))?;
        writeln!(&mut report)?;

        // Processed item count
        write!(
            &mut report,
            "{:<35} ",
            Colours::REPORT_LABEL.apply("* Records processed:"),
        )?;
        if self_report.record_processed_successful_count > 0 {
            writeln!(
                &mut report,
                "{:>7}",
                Colours::REPORT_ITEM_SUCCESS
                    .apply(self_report.record_processed_successful_count.to_string())
            )?;
        } else {
            writeln!(
                &mut report,
                "{:>7}",
                self_report.record_processed_successful_count
            )?;
        }

        // Missing info item count
        write!(
            &mut report,
            "{:<35} ",
            Colours::REPORT_LABEL.apply("* Records processed (missing info):"),
        )?;
        if self_report.record_processed_info_missing_count > 0 {
            writeln!(
                &mut report,
                "{:>7}",
                Colours::REPORT_ITEM_PARTIAL_SUCCESS
                    .apply(self_report.record_processed_info_missing_count.to_string())
            )?;
        } else {
            writeln!(
                &mut report,
                "{:>7}",
                self_report.record_processed_info_missing_count
            )?;
        }

        // Failed item count
        write!(
            &mut report,
            "{:<35} ",
            Colours::REPORT_LABEL.apply("* Records with errors:"),
        )?;
        if failed_count > 0 {
            writeln!(
                &mut report,
                "{:>7}",
                Colours::REPORT_ITEM_FAILURE.apply(failed_count.to_string())
            )?;
        } else {
            writeln!(&mut report, "{:>7}", failed_count)?;
        }

        // Skipped item count
        writeln!(
            &mut report,
            "{:<35} {:>7}",
            Colours::REPORT_LABEL.apply("* Records skipped (pre-existing):"),
            self_report.record_skipped_count
        )?;

        if failed_count > 0 {
            writeln!(&mut report)?;
            writeln!(
                &mut report,
                "{}",
                Colours::REPORT_TITLE_ERROR.apply("## Errors"),
            )?;
            writeln!(&mut report)?;

            // Error table headings
            writeln!(
                &mut report,
                "{row_index:>5} | {title_number:<13} | {error:30}",
                row_index = Colours::REPORT_LABEL.apply("#"),
                title_number = Colours::REPORT_LABEL.apply("title_number"),
                error = Colours::REPORT_LABEL.apply("error")
            )?;
            writeln!(
                &mut report,
                "----- | ------------- | ------------------------------"
            )?;
            self_report.records_processed_failed.iter().try_for_each(
                |(property_record_meta, error)| {
                    writeln!(
                        &mut report,
                        "{row_index:5} | {title_number:<13} | {error:30}",
                        row_index = property_record_meta.0,
                        title_number = Colours::REPORT_ERROR_ITEM
                            .apply(&format!("ABC123/{:02}", property_record_meta.0)),
                        error = Colours::REPORT_ERROR_MESSAGE.apply(error.to_string().as_str())
                    )
                },
            )?;
        }

        writeln!(
            &mut report,
            "{}",
            Colours::REPORT_BORDER
                .apply("------------------------------------------------------------")
        )?;

        let mut stderr = io::stderr();
        stderr.write_all(report.as_bytes())?;
        stderr.flush()?;

        Ok(())
    }
}
