use crate::minecraft_launcher::app::App;
use std::error::Error;

pub enum Event<I> {
    Input(I),
    Tick,
}

/// Crossterm demo
#[derive(Debug)]
pub struct Cli {
    /// time in ms between two ticks.
    pub tick_rate: u64,
    /// whether unicode symbols are used to improve the overall look of the app
    pub enhanced_graphics: bool,
}

pub fn main(app: App) -> Result<(), Box<dyn Error>> {
    app.run()?;

    Ok(())
}
