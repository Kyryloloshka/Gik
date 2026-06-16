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
            let mut existing_parents = Vec::new();
            let mut parents_to_add = Vec::new();
            for p in parents {
                if let Some(idx) = self.columns.iter().position(|h| h == p) {
                    existing_parents.push(idx);
                } else {
                    parents_to_add.push(*p);
                }
            }

            let mut added = 0;
            let mut removed = false;

            if parents_to_add.is_empty() {
                self.columns.remove(pos);
                removed = true;
            } else {
                self.columns[pos] = parents_to_add[0];
                for p in parents_to_add.iter().skip(1) {
                    self.columns.push(*p);
                    added += 1;
                }
            }

            let is_merge = parents.len() > 1;
            let merged_left = existing_parents.iter().any(|&idx| idx < pos);
            
            if is_merge || added > 0 || removed {
                let mut trans = String::new();
                for i in 0..old_len {
                    if i == pos {
                        if removed {
                            if merged_left { trans.push_str("|/"); } else { trans.push_str("|\\"); }
                        } else if added > 0 {
                            trans.push_str("|\\");
                        } else if is_merge {
                            if merged_left { trans.push_str("|/"); } else { trans.push_str("|\\"); }
                        } else {
                            trans.push_str("| ");
                        }
                    } else {
                        trans.push_str("| ");
                    }
                }
                for _ in 0..added {
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
