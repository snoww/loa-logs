use rsntp::SntpClient;

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
pub trait TimeSyncClient {
    fn synchronize(&self) -> Option<i64>;
}

pub struct SntpTimeSyncClient<'a>(SntpClient, &'a str);

impl<'a> TimeSyncClient for SntpTimeSyncClient<'a> {
    fn synchronize(&self) -> Option<i64> {

        self.0.synchronize(self.1).ok()
            .map(|pr| pr.datetime().into_chrono_datetime().unwrap_or_default().timestamp_millis())
    }
}

impl<'a> SntpTimeSyncClient<'a> {
    pub fn new(url: &'a str) -> Self {
        Self(SntpClient::new(), url)
    }
}