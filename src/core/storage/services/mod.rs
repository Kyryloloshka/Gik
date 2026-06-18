pub mod index;
pub mod commit;
pub mod undo;
pub mod object;
pub mod refs;
pub mod config;
pub mod session;

pub use index::IndexService;
pub use commit::CommitService;
pub use undo::UndoService;
pub use object::ObjectService;
pub use refs::RefService;
pub use config::ConfigService;
pub use session::SessionService;


