pub mod fields;
pub mod detail;
pub mod download;

use anyhow::Result;
use crate::cli::{InspectArgs, DownloadArgs};
use crate::context::Context;

pub async fn inspect(ctx: &Context, args: &InspectArgs) -> Result<()> {
    detail::detail(ctx, args).await
}

pub async fn download(ctx: &Context, args: &DownloadArgs) -> Result<()> {
    let inspect_args = InspectArgs {
        query_id: args.query_id.clone(),
        output: args.output.clone(),
    };
    detail::detail(ctx, &inspect_args).await
} 