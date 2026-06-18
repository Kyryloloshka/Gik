pub enum MergeResult {
    /// Conflict resolved automatically (or no changes detected).
    Resolved(Vec<u8>),

    /// Unresolvable conflict detected.
    Conflict {
        base: Option<Vec<u8>>,
        ours: Vec<u8>,
        theirs: Vec<u8>,
    },
}

pub trait MergeStrategy {
    fn merge(&self, base: Option<&[u8]>, ours: &[u8], theirs: &[u8]) -> MergeResult;
}
