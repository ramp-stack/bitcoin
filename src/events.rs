use pelican_ui::events::Event;
use pelican_ui::Context;

#[derive(Debug, Clone)]
pub struct UpdateWalletEvent;

impl Event for UpdateWalletEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}