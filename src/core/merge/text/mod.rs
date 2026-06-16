use crate::core::merge::strategy::{MergeResult, MergeStrategy};
use similar::{TextDiff, DiffOp};
use std::cmp;

pub struct TextMergeStrategy;

#[derive(Debug, Clone)]
struct Hunk<'a> {
    base_start: usize,
    base_end: usize,
    inserted_lines: Vec<&'a str>,
}

fn get_hunks<'a>(ops: &[DiffOp], new_lines: &[&'a str]) -> Vec<Hunk<'a>> {
    let mut hunks = Vec::new();
    for op in ops {
        match op {
            DiffOp::Insert { old_index, new_index, new_len } => {
                hunks.push(Hunk {
                    base_start: *old_index,
                    base_end: *old_index,
                    inserted_lines: new_lines[*new_index..*new_index + *new_len].to_vec(),
                });
            }
            DiffOp::Delete { old_index, old_len, .. } => {
                hunks.push(Hunk {
                    base_start: *old_index,
                    base_end: *old_index + *old_len,
                    inserted_lines: vec![],
                });
            }
            DiffOp::Replace { old_index, old_len, new_index, new_len } => {
                hunks.push(Hunk {
                    base_start: *old_index,
                    base_end: *old_index + *old_len,
                    inserted_lines: new_lines[*new_index..*new_index + *new_len].to_vec(),
                });
            }
            DiffOp::Equal { .. } => {}
        }
    }
    hunks
}

#[derive(Default)]
struct ConflictGroup<'a> {
    base_start: usize,
    base_end: usize,
    ours: Vec<Hunk<'a>>,
    theirs: Vec<Hunk<'a>>,
}

/// Groups overlapping hunks from both branches into "ConflictGroups".
/// Uses a Sweepline algorithm to merge intersecting intervals.
fn group_hunks<'a>(ours: Vec<Hunk<'a>>, theirs: Vec<Hunk<'a>>) -> Vec<ConflictGroup<'a>> {
    let mut groups = Vec::new();
    let mut i = 0;
    let mut j = 0;

    while i < ours.len() || j < theirs.len() {
        let mut group = ConflictGroup::default();

        // 1. Start a new group with the earliest available hunk
        let mut group_end = if i < ours.len() && (j == theirs.len() || ours[i].base_start <= theirs[j].base_start) {
            let end = ours[i].base_end;
            group.ours.push(ours[i].clone());
            i += 1;
            end
        } else {
            let end = theirs[j].base_end;
            group.theirs.push(theirs[j].clone());
            j += 1;
            end
        };

        // 2. Expand the group boundaries as long as we find overlapping hunks
        let mut expanded = true;
        while expanded {
            expanded = false;

            // Add all overlapping "ours" hunks
            while i < ours.len() && ours[i].base_start <= group_end {
                group_end = cmp::max(group_end, ours[i].base_end);
                group.ours.push(ours[i].clone());
                i += 1;
                expanded = true;
            }

            // Add all overlapping "theirs" hunks
            while j < theirs.len() && theirs[j].base_start <= group_end {
                group_end = cmp::max(group_end, theirs[j].base_end);
                group.theirs.push(theirs[j].clone());
                j += 1;
                expanded = true;
            }
        }

        // 3. Compute the overall boundaries for the accumulated group
        group.base_start = group.ours.first().map(|h| h.base_start)
            .unwrap_or(usize::MAX)
            .min(group.theirs.first().map(|h| h.base_start).unwrap_or(usize::MAX));
        group.base_end = group_end;

        groups.push(group);
    }

    groups
}

fn apply_groups(base_lines: &[&str], groups: Vec<ConflictGroup>) -> (String, bool) {
    let mut merged = String::new();
    let mut base_cursor = 0;
    let mut has_conflict = false;

    for group in groups {
        for line in &base_lines[base_cursor..group.base_start] {
            merged.push_str(line);
        }
        base_cursor = group.base_end;

        let net_ours: String = group.ours.iter().flat_map(|h| h.inserted_lines.clone()).collect();
        let net_theirs: String = group.theirs.iter().flat_map(|h| h.inserted_lines.clone()).collect();

        if group.ours.is_empty() {
            merged.push_str(&net_theirs);
        } else if group.theirs.is_empty() {
            merged.push_str(&net_ours);
        } else if net_ours == net_theirs {
            merged.push_str(&net_ours);
        } else {
            has_conflict = true;
            merged.push_str("<<<<<<< OURS\n");
            merged.push_str(&net_ours);
            if !net_ours.is_empty() && !net_ours.ends_with('\n') {
                merged.push('\n');
            }
            merged.push_str("=======\n");
            merged.push_str(&net_theirs);
            if !net_theirs.is_empty() && !net_theirs.ends_with('\n') {
                merged.push('\n');
            }
            merged.push_str(">>>>>>> THEIRS\n");
        }
    }

    for line in &base_lines[base_cursor..] {
        merged.push_str(line);
    }

    (merged, has_conflict)
}

impl MergeStrategy for TextMergeStrategy {
    fn merge(&self, base: Option<&[u8]>, ours: &[u8], theirs: &[u8]) -> MergeResult {
        let base_bytes = base.unwrap_or(b"");
        let ours_str = std::str::from_utf8(ours).unwrap_or("");
        let theirs_str = std::str::from_utf8(theirs).unwrap_or("");
        let base_str = std::str::from_utf8(base_bytes).unwrap_or("");

        if ours_str == theirs_str {
            return MergeResult::Resolved(ours.to_vec());
        }
        if ours_str == base_str {
            return MergeResult::Resolved(theirs.to_vec());
        }
        if theirs_str == base_str {
            return MergeResult::Resolved(ours.to_vec());
        }

        let diff_ours = TextDiff::from_lines(base_str, ours_str);
        let diff_theirs = TextDiff::from_lines(base_str, theirs_str);

        let base_lines: Vec<&str> = base_str.split_inclusive('\n').collect();
        let ours_lines: Vec<&str> = ours_str.split_inclusive('\n').collect();
        let theirs_lines: Vec<&str> = theirs_str.split_inclusive('\n').collect();

        let ours_hunks = get_hunks(diff_ours.ops(), &ours_lines);
        let theirs_hunks = get_hunks(diff_theirs.ops(), &theirs_lines);

        let groups = group_hunks(ours_hunks, theirs_hunks);
        let (merged, has_conflict) = apply_groups(&base_lines, groups);

        if has_conflict {
            MergeResult::Conflict {
                base: base.map(|b| b.to_vec()),
                ours: ours.to_vec(),
                theirs: theirs.to_vec(),
            }
        } else {
            MergeResult::Resolved(merged.into_bytes())
        }
    }
}

#[cfg(test)]
mod tests;
