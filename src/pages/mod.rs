use std::time::Duration;

pub mod network;
pub mod monitor;

pub trait Page {
    fn display(&self) -> String;
    fn refresh_rate(&self) -> Duration;
    fn update(&mut self);
}