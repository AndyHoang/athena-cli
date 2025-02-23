use aws_sdk_athena::Client;
use crate::cli::WorkgroupArgs;
use anyhow::Result;

pub async fn list(client: Client, args: WorkgroupArgs) -> Result<()> {
    println!("Listing workgroups (limit: {})", args.limit);
    
    let result = client.list_work_groups().max_results(args.limit).send().await?;
    
    for wg in result.work_groups() {
        if let Some(name) = wg.name() {
            println!("- {}", name);
        }
    }

    Ok(())
} 