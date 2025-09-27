use crate::core::summary::def;

mod general;
mod amazon;
mod branchio;
mod spotify;
mod twitter;
mod skeb;

pub static HANDLERS: &[&dyn def::SummalyHandler] = &[
    &skeb::SkebHandler,
    &twitter::TwitterHandler,
    &spotify::SpotifyHandler,
    &branchio::BranchioHandler,
    &amazon::AmazonHandler,
    &general::GeneralHandler,
];
