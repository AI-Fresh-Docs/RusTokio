use async_trait::async_trait;
use rustok_core::{MigrationSource, RusToKModule};
use sea_orm_migration::MigrationTrait;

pub mod dto;
pub mod error;
pub mod services;

pub use dto::{CreatePageInput, PageResponse};
pub use error::{PageError, PageResult};
pub use services::PageService;

pub struct PagesModule;

#[async_trait]
impl RusToKModule for PagesModule {
    fn slug(&self) -> &'static str {
        "pages"
    }

    fn name(&self) -> &'static str {
        "Pages"
    }

    fn description(&self) -> &'static str {
        "Static pages module (About, Contacts, Privacy Policy, Landing Pages)"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

impl MigrationSource for PagesModule {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Vec::new()
    }
}
