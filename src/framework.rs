use std::sync::Arc;
use twilight_http::Client;
use twilight_model::id::Id;
use vesper::prelude::*;

pub fn build_framework(http_client: Arc<Client>, application_id: u64) -> Arc<Framework<()>> {
    Arc::new(
        Framework::builder(http_client, Id::new(application_id), ())
            .command(crate::commands::promoters)
            .build(),
    )
}
