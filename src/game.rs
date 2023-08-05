use pagurus::{event::Event, Result, System};

#[derive(Debug, Default)]
pub struct Game {}

impl<S: System> pagurus::Game<S> for Game {
    fn initialize(&mut self, system: &mut S) -> Result<()> {
        todo!()
    }

    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool> {
        todo!()
    }
}
