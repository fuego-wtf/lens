use async_trait::async_trait;
use std::pin::Pin;
use tokio_stream::Stream;

use crate::{Lens, LensContext, LensEvent, LensResult, Result};

/// Event stream type
pub type LensEventStream = Pin<Box<dyn Stream<Item = LensEvent> + Send>>;

/// Lens trait with streaming support for observable execution.
///
/// Implement this trait for lenses that emit progress events during execution.
/// The desktop client subscribes to the event stream to show real-time progress.
///
/// # Example
///
/// ```rust
/// use async_trait::async_trait;
/// use lens::{
///     Lens, LensContext, LensEvent, LensResult, Result,
///     StreamingLens, LensEventStream,
/// };
/// use tokio::sync::mpsc;
/// use tokio_stream::wrappers::ReceiverStream;
/// use std::time::{Duration, Instant};
///
/// struct CounterLens;
///
/// #[async_trait]
/// impl Lens for CounterLens {
///     fn id(&self) -> &str { "counter" }
///     fn name(&self) -> &str { "Counter" }
///     fn version(&self) -> &str { "1.0.0" }
///
///     async fn execute(&self, ctx: LensContext) -> Result<LensResult> {
///         Ok(LensResult::success(serde_json::json!({"count": 10})))
///     }
/// }
///
/// #[async_trait]
/// impl StreamingLens for CounterLens {
///     async fn execute_streaming(
///         &self,
///         ctx: LensContext,
///     ) -> Result<(LensResult, LensEventStream)> {
///         let (tx, rx) = mpsc::channel(100);
///         let lens_id = self.id().to_string();
///         let start = Instant::now();
///
///         tokio::spawn(async move {
///             let _ = tx.send(LensEvent::started(&lens_id, "counting")).await;
///
///             for i in 1..=10 {
///                 let _ = tx.send(LensEvent::progress_with_percent(
///                     &lens_id,
///                     format!("Count: {}", i),
///                     i as f32 * 10.0,
///                 )).await;
///             }
///
///             let _ = tx.send(LensEvent::completed(&lens_id, start.elapsed())).await;
///         });
///
///         Ok((
///             LensResult::success(serde_json::json!({"count": 10})),
///             Box::pin(ReceiverStream::new(rx)),
///         ))
///     }
/// }
/// ```
#[async_trait]
pub trait StreamingLens: Lens {
    /// Execute the lens task with event streaming
    ///
    /// Returns both the final result and a stream of events that
    /// occurred during execution. Desktop can subscribe to this
    /// stream to show progress in the UI.
    async fn execute_streaming(
        &self,
        ctx: LensContext,
    ) -> Result<(LensResult, LensEventStream)>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LensError;
    use serde_json::json;
    use std::path::PathBuf;
    use std::time::Instant;
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;
    use tokio_stream::StreamExt;

    struct TestStreamingLens;

    #[async_trait]
    impl Lens for TestStreamingLens {
        fn id(&self) -> &str {
            "test_streaming"
        }

        fn name(&self) -> &str {
            "Test Streaming Lens"
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        async fn execute(&self, _ctx: LensContext) -> Result<LensResult> {
            Ok(LensResult::success(json!({"status": "ok"})))
        }
    }

    #[async_trait]
    impl StreamingLens for TestStreamingLens {
        async fn execute_streaming(
            &self,
            _ctx: LensContext,
        ) -> Result<(LensResult, LensEventStream)> {
            let (tx, rx) = mpsc::channel(100);
            let lens_id = self.id().to_string();
            let start = Instant::now();

            tokio::spawn(async move {
                let _ = tx.send(LensEvent::started(&lens_id, "test")).await;

                for i in 1..=3 {
                    let _ = tx
                        .send(LensEvent::progress_with_percent(
                            &lens_id,
                            format!("Step {}", i),
                            i as f32 * 33.3,
                        ))
                        .await;
                }

                let _ = tx
                    .send(LensEvent::completed(&lens_id, start.elapsed()))
                    .await;
            });

            Ok((
                LensResult::success(json!({"total_steps": 3})),
                Box::pin(ReceiverStream::new(rx)),
            ))
        }
    }

    struct FailingStreamingLens;

    #[async_trait]
    impl Lens for FailingStreamingLens {
        fn id(&self) -> &str {
            "failing"
        }
        fn name(&self) -> &str {
            "Failing Lens"
        }
        fn version(&self) -> &str {
            "1.0.0"
        }

        async fn execute(&self, _ctx: LensContext) -> Result<LensResult> {
            Err(LensError::ExecutionFailed("test failure".to_string()))
        }
    }

    #[async_trait]
    impl StreamingLens for FailingStreamingLens {
        async fn execute_streaming(
            &self,
            _ctx: LensContext,
        ) -> Result<(LensResult, LensEventStream)> {
            let (tx, rx) = mpsc::channel(100);
            let lens_id = self.id().to_string();

            tokio::spawn(async move {
                let _ = tx.send(LensEvent::started(&lens_id, "failing")).await;
                let _ = tx
                    .send(LensEvent::failed(&lens_id, "Something went wrong", false))
                    .await;
            });

            Ok((
                LensResult::failure("Task failed".to_string()),
                Box::pin(ReceiverStream::new(rx)),
            ))
        }
    }

    #[tokio::test]
    async fn test_streaming_task_emits_events() {
        let lens = TestStreamingLens;
        let ctx = LensContext::new(PathBuf::from("/tmp"), json!({}));

        let (result, mut stream) = lens.execute_streaming(ctx).await.unwrap();

        assert!(result.success);
        assert_eq!(result.output["total_steps"], 3);

        let mut events = Vec::new();
        while let Some(event) = stream.next().await {
            events.push(event);
        }

        // Should have: Started, 3 Progress, Completed = 5 events
        assert_eq!(events.len(), 5);

        // First event should be Started
        match &events[0] {
            LensEvent::Started { lens, task, .. } => {
                assert_eq!(lens, "test_streaming");
                assert_eq!(task, "test");
            }
            _ => panic!("Expected Started event"),
        }

        // Last event should be Completed
        match &events[4] {
            LensEvent::Completed { lens, .. } => {
                assert_eq!(lens, "test_streaming");
            }
            _ => panic!("Expected Completed event"),
        }
    }

    #[tokio::test]
    async fn test_streaming_task_progress_events() {
        let lens = TestStreamingLens;
        let ctx = LensContext::new(PathBuf::from("/tmp"), json!({}));

        let (_result, mut stream) = lens.execute_streaming(ctx).await.unwrap();

        let mut progress_events = Vec::new();
        while let Some(event) = stream.next().await {
            if let LensEvent::Progress { message, percent, .. } = event {
                progress_events.push((message, percent));
            }
        }

        assert_eq!(progress_events.len(), 3);
        assert_eq!(progress_events[0].0, "Step 1");
        assert_eq!(progress_events[1].0, "Step 2");
        assert_eq!(progress_events[2].0, "Step 3");
    }

    #[tokio::test]
    async fn test_streaming_task_failure() {
        let lens = FailingStreamingLens;
        let ctx = LensContext::new(PathBuf::from("/tmp"), json!({}));

        let (result, mut stream) = lens.execute_streaming(ctx).await.unwrap();

        assert!(!result.success);

        let mut events = Vec::new();
        while let Some(event) = stream.next().await {
            events.push(event);
        }

        // Should have: Started, Failed = 2 events
        assert_eq!(events.len(), 2);

        match &events[1] {
            LensEvent::Failed {
                error, recoverable, ..
            } => {
                assert_eq!(error, "Something went wrong");
                assert!(!recoverable);
            }
            _ => panic!("Expected Failed event"),
        }
    }

    #[test]
    fn test_streaming_task_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TestStreamingLens>();
    }
}
