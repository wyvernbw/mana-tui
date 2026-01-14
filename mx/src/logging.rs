use flume::Sender;
use tracing_subscriber::Layer;

use crate::RenderMsg;

pub struct RatatuiLayer {
    sender: Sender<RenderMsg>,
}

impl RatatuiLayer {
    pub fn new(sender: Sender<RenderMsg>) -> Self {
        Self { sender }
    }
}

impl<S> Layer<S> for RatatuiLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();

        // Extract message
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let log_line = format!(
            "[{}] {}: {}",
            metadata.level(),
            metadata.target(),
            visitor.message
        );

        // Send to channel (ignore errors if receiver dropped)
        let _ = self.sender.send(RenderMsg::Log(log_line.into_boxed_str()));
    }
}

#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else {
        }
    }
}
