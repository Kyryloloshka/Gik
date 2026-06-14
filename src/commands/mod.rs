pub mod init;
pub mod stage;
pub mod commit;
pub mod log;
pub mod status;
pub mod undo;
pub mod update;
pub mod diff;

pub use init::init;
pub use stage::stage;
pub use commit::commit;
pub use log::log;
pub use status::status;
pub use undo::undo;
pub use update::update;
pub use diff::diff;

#[cfg(test)]
mod tests;
