use crate::PropertyRecord;

/// Report containing information about the execution.
#[derive(Debug, Default)]
pub struct Report {
    /// Number of records already in the output before the execution.
    pub record_skipped_count: usize,
    /// Number of records that we successfully processed.
    pub record_processed_successful_count: usize,
    /// Number of records that have some information missing.
    pub record_processed_info_missing_count: usize,
    /// Errors for records that failed to process.
    pub records_processed_failed: Vec<(PropertyRecord, &'static str)>,
}
