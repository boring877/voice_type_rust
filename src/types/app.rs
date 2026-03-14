//! Application state types
//!
//! Contains the shared state used to coordinate between different components
//! of the application.

use crate::types::Config;

/// Application state shared between components
#[derive(Debug)]
pub struct SharedState {
    /// User configuration
    pub config: Config,
    /// Whether app should quit
    pub should_quit: bool,
}

impl SharedState {
    /// Create a new shared state with the given configuration
    pub fn new(config: Config) -> Self {
        Self {
            config,
            should_quit: false,
        }
    }
}
