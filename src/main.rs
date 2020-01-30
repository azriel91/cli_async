use crossbeam::channel;
use structopt::{clap::AppSettings, StructOpt};

mod colours;
mod report;
mod reporter;

mod types {
    #[derive(Clone, Copy, Debug)]
    pub struct Credentials;

    #[derive(Clone, Copy, Debug)]
    pub struct PropertyRecord(pub usize);

    #[derive(Clone, Copy, Debug)]
    pub struct PropertyRecordPopulated {
        pub record: PropertyRecord,
        pub info: PropertyInfoResult,
    }

    #[derive(Clone, Copy, Debug)]
    pub enum PropertyInfoResult {
        Success,
        SuccessPartial,
        Error(PropertyRecord, &'static str),
    }
}

/// Startup tasks
#[rustfmt::skip]
mod startup {
    use crossbeam::channel::{self, Receiver};
    use crate::{Credentials, PropertyRecord, Reporter};

    pub fn t00_setup_interrupt_handler() -> Receiver<()> {
        let (tx, rx) = channel::bounded::<()>(2);

        ctrlc::set_handler(move || {
            tx.send(()).expect("Failed to send interrupt message.");
        }).expect("Error setting Ctrl-C handler");

        rx
    }
    pub fn t01_read_credentials() -> Credentials { Credentials }
    pub fn t02_stream_property_title_records(n: usize) -> Vec<PropertyRecord> { (0..n).map(PropertyRecord).collect() }
    pub fn t03_read_output_file(processed_count: usize) -> usize { processed_count }
    pub fn t04_start_progress_bar(reporter: &mut Reporter) { reporter.progress_bar_startup(); }
}

/// Looped tasks
#[rustfmt::skip]
mod looped {
    use std::{thread, time::Duration};
    use crate::{Credentials, PropertyRecord, PropertyInfoResult, PropertyRecordPopulated, Reporter};

    pub fn t05_rate_limit_requests(delay: u64) { thread::sleep(Duration::from_millis(delay)) }
    pub fn t06_authenticate_with_server(first_time: bool, _: Credentials, delay: u64) { if first_time { thread::sleep(Duration::from_millis(delay)) } }
    pub fn t07_retrieve_information(n: usize, property_record: PropertyRecord, delay: u64) -> PropertyInfoResult {
        thread::sleep(Duration::from_millis(delay));
        if n % 11 == 0 && n % 3 == 0 { PropertyInfoResult::Error(property_record, "Could not find record information online.") }
        else if n % 3 == 0 { PropertyInfoResult::SuccessPartial }
        else { PropertyInfoResult::Success }
    }
    pub fn t08_augment_record(record: PropertyRecord, info: PropertyInfoResult) -> PropertyRecordPopulated { PropertyRecordPopulated { record, info } }
    pub fn t09_output_record_to_file(_: PropertyRecordPopulated) { thread::sleep(Duration::from_millis(10)) }
    pub fn t10_update_progress_bar(reporter: &mut Reporter) { reporter.progress_bar_sync(); }
}

// Final task
mod last {
    use crate::Reporter;

    pub fn t11_output_execution_report(reporter: &Reporter) {
        reporter
            .print_report()
            .expect("Failed to print execution report.")
    }
}

use crate::{
    colours::Colours, last::*, looped::*, report::Report, reporter::Reporter, startup::*, types::*,
};

#[derive(Debug, StructOpt)]
#[structopt(
    global_setting = AppSettings::ColoredHelp,
    about = "Simulates online information lookup for records.",
)]
struct Opt {
    /// Total number of records.
    #[structopt(short, long, default_value = "50")]
    count: usize,
    /// Number of records already processed.
    #[structopt(short, long, default_value = "0")]
    skip: usize,
    /// Number of milliseconds to sleep per record.
    #[structopt(long, default_value = "50")]
    delay_rate_limit: u64,
    /// Number of milliseconds authentication takes.
    #[structopt(long, default_value = "20")]
    delay_auth: u64,
    /// Number of milliseconds information retrieval takes.
    #[structopt(long, default_value = "50")]
    delay_retrieve: u64,
}

fn main() {
    let Opt {
        count: record_count,
        skip,
        delay_rate_limit,
        delay_auth,
        delay_retrieve,
    } = Opt::from_args();

    let (progress_tx, progress_rx) = channel::unbounded::<PropertyInfoResult>();
    Reporter::print_logo().expect("Failed to print logo.");

    let interrupt_rx = t00_setup_interrupt_handler();
    let credentials = t01_read_credentials();
    let records = t02_stream_property_title_records(record_count);
    let records_precompleted = t03_read_output_file(skip);
    let mut reporter = Reporter::new(
        record_count as u64,
        records_precompleted as u64,
        progress_rx,
        true,
        Some(interrupt_rx),
    );
    t04_start_progress_bar(&mut reporter);

    let _result = records
        .into_iter()
        .enumerate()
        .skip(records_precompleted)
        .map(|(n, record)| {
            t05_rate_limit_requests(delay_rate_limit);
            t06_authenticate_with_server(n == 0, credentials, delay_auth);
            let info = t07_retrieve_information(n, record, delay_retrieve);
            progress_tx
                .send(info)
                .expect("Failed to send progress update.");
            t08_augment_record(record, info)
        })
        .try_for_each(|property_record_populated| {
            t09_output_record_to_file(property_record_populated);
            t10_update_progress_bar(&mut reporter);

            if reporter.is_interrupted() {
                Err(()) // Quick exit
            } else {
                Ok(())
            }
        });

    t11_output_execution_report(&reporter);
}
