use crate::cli::{AwsArgs, DisplayArgs};
use crate::config::Config;
use anyhow::Result;
use std::env;
use std::sync::Arc;

/// Holds all runtime context including config, CLI args, and AWS clients
pub struct Context {
    config: Config,
    aws_args: AwsArgs,
    display_args: DisplayArgs,
    aws_config: Arc<aws_config::SdkConfig>,
}

impl Context {
    pub async fn new(config: Config, aws_args: AwsArgs, display_args: DisplayArgs) -> Result<Self> {
        let ctx = Self {
            config,
            aws_args,
            display_args,
            aws_config: Arc::new(aws_config::SdkConfig::builder().build()),
        };

        let aws_config = Arc::new(crate::aws::build_aws_config(ctx.profile(), ctx.region()).await?);

        Ok(Self { aws_config, ..ctx })
    }

    pub fn profile(&self) -> Option<String> {
        self.aws_args
            .profile
            .clone()
            .or_else(|| env::var("AWS_PROFILE").ok())
            .or_else(|| env::var("AWS_DEFAULT_PROFILE").ok())
            .or_else(|| self.config.aws.profile.clone())
    }

    pub fn region(&self) -> String {
        let region = self
            .aws_args
            .region
            .as_ref()
            .cloned()
            .or_else(|| env::var("AWS_REGION").ok())
            .or_else(|| self.config.aws.region.clone())
            .unwrap_or_else(|| "eu-west-1".to_string());

        region
    }

    pub fn database(&self) -> Option<String> {
        self.aws_args
            .database
            .clone()
            .or_else(|| env::var("AWS_ATHENA_DATABASE").ok())
            .or_else(|| self.config.aws.database.clone())
    }

    pub fn workgroup(&self) -> String {
        self.aws_args
            .workgroup
            .as_ref()
            .cloned()
            .or_else(|| env::var("AWS_ATHENA_WORKGROUP").ok())
            .or_else(|| self.config.aws.workgroup.clone())
            .unwrap_or_else(|| "primary".to_string())
    }

    pub fn catalog(&self) -> String {
        self.aws_args
            .catalog
            .as_ref()
            .cloned()
            .or_else(|| env::var("AWS_ATHENA_CATALOG").ok())
            .or_else(|| self.config.aws.catalog.clone())
            .unwrap_or_else(|| "AwsDataCatalog".to_string())
    }

    pub fn output_location(&self) -> Option<String> {
        self.aws_args
            .output_location
            .clone()
            .or_else(|| env::var("AWS_ATHENA_OUTPUT_LOCATION").ok())
            .or(Some(self.config.aws.output_location.clone()))
    }

    pub fn aws_config(&self) -> &aws_config::SdkConfig {
        &self.aws_config
    }

    pub fn create_athena_client(&self) -> aws_sdk_athena::Client {
        aws_sdk_athena::Client::new(&self.aws_config)
    }

    pub fn quiet(&self) -> bool {
        self.display_args.quiet
    }

    pub fn history_size(&self) -> i32 {
        self.config.app.history_size
    }
}
