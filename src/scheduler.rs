use std::{future::Future, pin::Pin};

use crate::Walle;
use tokio_cron_scheduler::{Job, JobScheduler};

pub trait ScheduledJob {
    fn cron(&self) -> &'static str;
    fn call(&self, walle: Walle) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>;
}

pub struct Scheduler {
    inner: JobScheduler,
    walle: Walle,
}

impl Scheduler {
    pub fn new(walle: Walle) -> Self {
        Self {
            inner: JobScheduler::new().unwrap(),
            walle,
        }
    }

    pub fn add(&mut self, job: impl ScheduledJob + Send + Sync + 'static) {
        let walle = self.walle.clone();
        let job = Job::new_async(job.cron(), move |_, _| job.call(walle.clone())).unwrap();
        self.inner.add(job).unwrap();
    }

    pub fn start(&self) {
        self.inner.start().unwrap();
    }
}

/// for test
pub struct OneMinutePassed;

impl ScheduledJob for OneMinutePassed {
    fn cron(&self) -> &'static str {
        "0 * * * * *"
    }
    fn call(&self, walle: Walle) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>> {
        Box::pin(async move {
            for (bot_id, _bot) in walle.bots.read().await.iter() {
                println!("One minute passed with bot: {}", bot_id);
            }
        })
    }
}
