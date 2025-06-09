use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Use Cargo's env var to check the TARGET vendor at runtime
    let target_vendor = std::env::var("CARGO_CFG_TARGET_VENDOR").unwrap_or_default();
    #[cfg(target_vendor = "apple")]
    {
        // Only call the Apple-specific build if both:
        // - The code is being compiled for Apple (host)
        // - The target vendor is Apple (target)
        if target_vendor == "apple" {
            build_apple::build_for_apple().await?;
        }
    }
    Ok(())
}

// Only compile this module if the host is Apple
#[cfg(target_vendor = "apple")]
mod build_apple;
