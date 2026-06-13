pub mod error;
pub mod models;
pub mod storage;

use error::Result;

fn main() -> Result<()> {
    println!("Gik initialized");
    Ok(())
}
