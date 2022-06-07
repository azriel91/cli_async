use futures::{stream, StreamExt, TryStreamExt};
use structopt::{clap::AppSettings, StructOpt};
use tokio::sync::mpsc;

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
    use std::future::Future;
    use async_ctrlc::CtrlC;
    use tokio::sync::mpsc::{self, Receiver};
    use crate::{Credentials, PropertyRecord, Reporter};

    pub fn t00_setup_interrupt_handler() -> (impl Future<Output = ()>, Receiver<()>) {
        let (tx, rx) = mpsc::channel::<()>(2);

        let ctrl_c = CtrlC::new().expect("Error setting Ctrl-C handler");

        let ctrl_c_future = async move {
            ctrl_c.await;
            tx.send(()).await.expect("Failed to send interrupt message.");
        };

        (ctrl_c_future, rx)
    }
    pub fn t01_read_credentials() -> Credentials { Credentials }
    pub fn t02_stream_property_title_records(n: usize) -> Vec<PropertyRecord> { (0..n).map(PropertyRecord).collect() }
    pub fn t03_read_output_file(processed_count: usize) -> usize { processed_count }
    pub fn t04_start_progress_bar(reporter: &mut Reporter) { reporter.progress_bar_startup(); }
}

/// Looped tasks
#[rustfmt::skip]
mod looped {
    use std::{time::Duration};
    use tokio::time::sleep;
    use crate::{Credentials, PropertyRecord, PropertyInfoResult, PropertyRecordPopulated, Reporter};

    pub async fn t05_rate_limit_requests(delay: u64) { sleep(Duration::from_millis(delay)).await }
    pub async fn t06_authenticate_with_server(first_time: bool, _: Credentials, delay: u64) { if first_time { sleep(Duration::from_millis(delay)).await } }
    pub async fn t07_retrieve_information(n: usize, property_record: PropertyRecord, delay: u64) -> PropertyInfoResult {
        async {
            sleep(Duration::from_millis(delay)).await;
            if n % 11 == 0 && n % 3 == 0 { PropertyInfoResult::Error(property_record, "Could not find record information online.") }
            else if n % 3 == 0 { PropertyInfoResult::SuccessPartial }
            else { PropertyInfoResult::Success }
        }.await
    }
    pub fn t08_augment_record(record: PropertyRecord, info: PropertyInfoResult) -> PropertyRecordPopulated { PropertyRecordPopulated { record, info } }
    pub async fn t09_output_record_to_file(_: PropertyRecordPopulated) { sleep(Duration::from_millis(10)).await }
    pub async fn t10_update_progress_bar(reporter: &mut Reporter) { reporter.progress_bar_sync().await }
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

#[tokio::main]
async fn main() -> Result<(), ()> {
    let Opt {
        count: record_count,
        skip,
        delay_rate_limit,
        delay_auth,
        delay_retrieve,
    } = Opt::from_args();

    let (progress_tx, progress_rx) = mpsc::unbounded_channel::<PropertyInfoResult>();
    Reporter::print_logo().expect("Failed to print logo.");

    let (ctrl_c_future, interrupt_rx) = t00_setup_interrupt_handler();
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

    let reporter_future = async move {
        t10_update_progress_bar(&mut reporter).await;
        t11_output_execution_report(&reporter);
    };

    let processing_future = async move {
        // Hacks for futures:
        let progress_tx = &progress_tx;

        stream::iter(records.into_iter().enumerate().skip(records_precompleted))
            .then(move |(n, record)| async move {
                t05_rate_limit_requests(delay_rate_limit).await;
                t06_authenticate_with_server(n == 0, credentials, delay_auth).await;
                let info = t07_retrieve_information(n, record, delay_retrieve).await;
                progress_tx
                    .send(info)
                    .expect("Failed to send progress update.");
                Result::<_, ()>::Ok(t08_augment_record(record, info))
            })
            .try_for_each_concurrent(10, move |property_record_populated| async move {
                t09_output_record_to_file(property_record_populated).await;

                Ok(())
            })
            .await
    };

    let reporter_handle = tokio::spawn(reporter_future);

    let ctrl_c_handle = tokio::spawn(ctrl_c_future);
    let processing_handle = tokio::spawn(processing_future);

    let processed_or_interrupted = async {
        tokio::select! {
            _ = ctrl_c_handle => {}
            _ = processing_handle => {}
        }
    };

    let (_, _) = tokio::join!(reporter_handle, processed_or_interrupted);

    Ok(())
}
