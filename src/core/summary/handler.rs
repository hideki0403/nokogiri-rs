use crate::core::summary::def;

mod amazon;
mod branchio;
mod general;
mod skeb;
mod spotify;
mod twitter;
mod wikipedia;
mod youtube;

pub static HANDLERS: &[&dyn def::SummalyHandler] = &[
    &wikipedia::WikipediaHandler,
    &youtube::YoutubeHandler,
    &skeb::SkebHandler,
    &twitter::TwitterHandler,
    &spotify::SpotifyHandler,
    &branchio::BranchioHandler,
    &amazon::AmazonHandler,
    &general::GeneralHandler,
];
