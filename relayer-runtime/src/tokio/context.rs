use alloc::sync::Arc;
use async_trait::async_trait;
use core::future::Future;
use core::marker::PhantomData;
use core::time::Duration;
use std::time::Instant;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, oneshot};
use tokio::time::sleep;
use tracing;

use ibc_relayer_framework::core::traits::core::Async;
use ibc_relayer_framework::one_for_all::traits::batch::OfaBatch;
use ibc_relayer_framework::one_for_all::traits::chain::OfaChain;
use ibc_relayer_framework::one_for_all::traits::error::OfaError;
use ibc_relayer_framework::one_for_all::traits::runtime::{LogLevel, OfaRuntime};

use super::error::Error as TokioError;

pub struct TokioRuntimeContext<Error> {
    pub runtime: Arc<Runtime>,
    pub phantom: PhantomData<Error>,
}

impl<Error> TokioRuntimeContext<Error> {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self {
            runtime,
            phantom: PhantomData,
        }
    }
}

impl<Error> Clone for TokioRuntimeContext<Error> {
    fn clone(&self) -> Self {
        Self::new(self.runtime.clone())
    }
}

#[async_trait]
impl<Error> OfaRuntime for TokioRuntimeContext<Error>
where
    Error: OfaError + From<TokioError>,
{
    type Error = Error;

    type Time = Instant;

    async fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Error => tracing::error!(message),
            LogLevel::Warn => tracing::warn!(message),
            LogLevel::Info => tracing::info!(message),
            LogLevel::Debug => tracing::debug!(message),
            LogLevel::Trace => tracing::trace!(message),
        }
    }

    async fn sleep(&self, duration: Duration) {
        sleep(duration).await;
    }

    fn now(&self) -> Instant {
        Instant::now()
    }

    fn duration_since(time: &Instant, other: &Instant) -> Duration {
        time.duration_since(other.clone())
    }

    fn spawn<F>(&self, task: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(task);
    }
}

#[async_trait]
impl<Chain, Error> OfaBatch<Chain> for TokioRuntimeContext<Error>
where
    Chain: OfaChain<Error = Error>,
    Error: From<TokioError> + Clone + Async,
{
    type MessagesSender = mpsc::Sender<(Vec<Chain::Message>, Self::ResultSender)>;
    type MessagesReceiver = mpsc::Receiver<(Vec<Chain::Message>, Self::ResultSender)>;

    type ResultSender = oneshot::Sender<Result<Vec<Vec<Chain::Event>>, Chain::Error>>;
    type ResultReceiver = oneshot::Receiver<Result<Vec<Vec<Chain::Event>>, Chain::Error>>;

    fn new_result_channel() -> (Self::ResultSender, Self::ResultReceiver) {
        oneshot::channel()
    }

    async fn send_messages(
        sender: &Self::MessagesSender,
        messages: Vec<Chain::Message>,
        result_sender: Self::ResultSender,
    ) -> Result<(), Chain::Error> {
        sender
            .send((messages, result_sender))
            .await
            .map_err(|_| TokioError::channel_closed().into())
    }

    async fn try_receive_messages(
        receiver: &mut Self::MessagesReceiver,
    ) -> Result<Option<(Vec<Chain::Message>, Self::ResultSender)>, Chain::Error> {
        match receiver.try_recv() {
            Ok(batch) => Ok(Some(batch)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => {
                Err(TokioError::channel_closed().into())
            }
        }
    }

    async fn receive_result(
        result_receiver: Self::ResultReceiver,
    ) -> Result<Result<Vec<Vec<Chain::Event>>, Chain::Error>, Chain::Error> {
        result_receiver
            .await
            .map_err(|_| TokioError::channel_closed().into())
    }

    fn send_result(
        result_sender: Self::ResultSender,
        events: Result<Vec<Vec<Chain::Event>>, Chain::Error>,
    ) -> Result<(), Chain::Error> {
        result_sender
            .send(events)
            .map_err(|_| TokioError::channel_closed().into())
    }
}