use pelican_ui::events::Event;
use pelican_ui::Context;

/// Event triggered when the [`QRScanner`] component detects a QR code.
#[derive(Debug, Clone)]
pub struct QRCodeScannedEvent(pub String);

impl Event for QRCodeScannedEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}