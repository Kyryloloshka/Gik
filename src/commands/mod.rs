pub mod init;
pub mod stage;
pub mod commit;
pub mod log;
pub mod undo;
pub mod update;

pub use init::init;
pub use stage::stage;
pub use commit::commit;
pub use log::log;
pub use undo::undo;
pub use update::update;


#[cfg(test)]
mod tests;
