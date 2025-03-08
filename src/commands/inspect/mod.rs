pub mod detail;
pub mod download;
pub mod fields;

pub use detail::detail as inspect;

use crate::cli::{DownloadArgs, InspectArgs};
use crate::context::Context;
use anyhow::Result;

pub async fn download(ctx: &Context, args: &DownloadArgs) -> Result<()> {
    // Create inspect args with forced quiet mode
    detail::detail(
        ctx,
        &InspectArgs {
            query_id: args.query_id.clone(),
            output: args.output.clone(),
            quiet: true, // Always quiet for downloads
        },
    )
    .await
}
