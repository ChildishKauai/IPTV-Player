pub mod xtream;
pub mod omdb;
pub mod football;
pub mod scraper_integration;

pub use xtream::XtreamClient;
pub use omdb::{DiscoverCache, DiscoverItem, DiscoverCategory, DiscoverContentType};
pub use football::{FootballCache, FootballFixture, FootballCategory};
pub use scraper_integration::{ScraperManager, ScrapingStatus};
