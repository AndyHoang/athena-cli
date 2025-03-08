pub mod fields;
pub mod detail;
pub mod download;

pub use detail::detail as inspect;

use anyhow::Result;
use crate::cli::{InspectArgs, DownloadArgs};
use crate::context::Context;

pub async fn download(ctx: &Context, args: &DownloadArgs) -> Result<()> {
    // Create inspect args with forced quiet mode
    detail::detail(ctx, &InspectArgs {
        query_id: args.query_id.clone(),
        output: args.output.clone(),
        quiet: true, // Always quiet for downloads
    }).await
} 