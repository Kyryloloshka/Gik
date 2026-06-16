use crate::core::hash::Hash;

pub struct GraphRenderer {
    columns: Vec<Hash>,
}

impl Default for GraphRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphRenderer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
        }
    }

    pub fn process_commit(&mut self, current: &Hash, parents: &[Hash]) -> (String, String, Vec<String>) {
        let mut pos = match self.columns.iter().position(|h| h == current) {
            Some(p) => p,
            None => {
                self.columns.push(*current);
                self.columns.len() - 1
            }
        };

        // Collapse any duplicate pointers to the current commit
        let mut to_remove = Vec::new();
        for (i, col) in self.columns.iter().enumerate() {
            if i != pos && col == current {
                to_remove.push(i);
            }
        }
        for i in to_remove.into_iter().rev() {
            self.columns.remove(i);
            if i < pos { pos -= 1; }
        }

        let mut commit_prefix = String::new();
        for i in 0..self.columns.len() {
            if i == pos {
                commit_prefix.push_str("* ");
            } else {
                commit_prefix.push_str("| ");
            }
        }

        let mut transitions = Vec::new();
        let old_len = self.columns.len();

        if parents.is_empty() {
            self.columns.remove(pos);
        } else {
            let p0 = parents[0];
            if let Some(existing_pos) = self.columns.iter().position(|h| h == &p0) {
                if existing_pos != pos {
                    // Parent is already tracked elsewhere, so this branch merges into it
                    self.columns.remove(pos);
                } else {
                    self.columns[pos] = p0;
                }
            } else {
                self.columns[pos] = p0;
            }

            let mut added = 0;
            for parent in parents.iter().skip(1) {
                if !self.columns.contains(parent) {
                    self.columns.push(*parent);
                    added += 1;
                }
            }

            if added > 0 {
                let mut trans = String::new();
                for i in 0..old_len {
                    if i == pos {
                        trans.push_str("|\\");
                    } else {
                        trans.push_str("| ");
                    }
                }
                for _ in 1..added {
                    trans.push_str("\\ ");
                }
                transitions.push(trans);
            }
        }

        let mut msg_prefix = String::new();
        for _ in 0..self.columns.len() {
            msg_prefix.push_str("| ");
        }

        (commit_prefix, msg_prefix, transitions)
    }
}
