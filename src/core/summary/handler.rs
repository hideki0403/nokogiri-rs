use crate::core::summary::def;

mod amazon;
mod branchio;
mod general;
mod reddit;
mod skeb;
mod spotify;
mod twitter;
mod wikipedia;
mod youtube;

pub static CUSTOM_HANDLERS: &[&dyn def::SummalyHandler] = &[
    &wikipedia::WikipediaHandler,
    &youtube::YoutubeHandler,
    &skeb::SkebHandler,
    &twitter::TwitterHandler,
    &spotify::SpotifyHandler,
    &branchio::BranchioHandler,
    &amazon::AmazonHandler,
    &reddit::RedditHandler,
];

pub static DEFAULT_HANDLER: &general::GeneralHandler = &general::GeneralHandler;
