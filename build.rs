use anyhow::{Ok, Result};
use substreams_ethereum::Abigen;

fn main() -> Result<(), anyhow::Error> {
    Abigen::new(
        "ENSRegistryWithFallback.json",
        "abi/ENSRegistryWithFallback.json",
    )?
    .generate()?
    .write_to_file("src/abi/ENSRegistryWithFallback.rs")?;

    Abigen::new(
        "DefaultReverseResolver.json",
        "abi/DefaultReverseResolver.json",
    )?
    .generate()?
    .write_to_file("src/abi/DefaultReverseResolver.rs")?;

    Abigen::new("ReverseRegistrar.json", "abi/ReverseRegistrar.json")?
        .generate()?
        .write_to_file("src/abi/ReverseRegistrar.rs")?;

    Ok(())
}
