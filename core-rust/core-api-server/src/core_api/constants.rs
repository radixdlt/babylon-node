pub(crate) const MAX_BATCH_COUNT_PER_REQUEST: u16 = 10000;
pub(crate) const CAP_BATCH_RESPONSE_WHEN_ABOVE_BYTES: usize = 15 * 1024 * 1024;

// TODO - Change this to be slightly larger than the double the max transaction payload size.
// (We double due to the hex encoding of the payload)
pub(crate) const LARGE_REQUEST_MAX_BYTES: usize = 3 * 1024 * 1024;
