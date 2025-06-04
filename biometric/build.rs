use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(target_vendor = "apple")]
    {
        build_apple::build_for_apple().await?;
    }
    Ok(())
}

#[cfg(target_vendor = "apple")]
mod build_apple;