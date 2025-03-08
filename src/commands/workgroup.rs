use anyhow::Result;
use crate::cli::WorkgroupArgs;
use crate::context::Context;

pub async fn list(ctx: &Context, args: &WorkgroupArgs) -> Result<()> {
    let client = ctx.create_athena_client();
    
    println!("Listing workgroups (limit: {})", args.limit);
    
    let result = client
        .list_work_groups()
        .max_results(args.limit)
        .send()
        .await?;

    // work_groups() returns a slice reference, not an Option
    for workgroup in result.work_groups() {
        if let Some(name) = workgroup.name() {
            println!("- {}", name);
        }
    }

    Ok(())
} 