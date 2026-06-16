use super::*;

fn merge_text(base: &str, ours: &str, theirs: &str) -> MergeResult {
    let strategy = TextMergeStrategy;
    strategy.merge(Some(base.as_bytes()), ours.as_bytes(), theirs.as_bytes())
}

#[test]
fn test_merge_no_conflict_different_methods() {
    let base = "fn one() {}\nfn two() {}\nfn three() {}\n";
    let ours = "fn one() { println!(\"1\"); }\nfn two() {}\nfn three() {}\n";
    let theirs = "fn one() {}\nfn two() {}\nfn three() { println!(\"3\"); }\n";

    let expected = "fn one() { println!(\"1\"); }\nfn two() {}\nfn three() { println!(\"3\"); }\n";

    match merge_text(base, ours, theirs) {
        MergeResult::Resolved(content) => {
            assert_eq!(String::from_utf8(content).unwrap(), expected);
        }
        _ => panic!("Expected Resolved, got Conflict"),
    }
}

#[test]
fn test_merge_conflict_same_line() {
    let base = "fn one() {}\n";
    let ours = "fn one() { println!(\"ours\"); }\n";
    let theirs = "fn one() { println!(\"theirs\"); }\n";

    match merge_text(base, ours, theirs) {
        MergeResult::Conflict { base: b, ours: o, theirs: t } => {
            assert_eq!(b.unwrap(), base.as_bytes());
            assert_eq!(o, ours.as_bytes());
            assert_eq!(t, theirs.as_bytes());
        }
        _ => panic!("Expected Conflict"),
    }
}

#[test]
fn test_merge_identical_changes() {
    let base = "fn one() {}\n";
    let ours = "fn one() { println!(\"same\"); }\n";
    let theirs = "fn one() { println!(\"same\"); }\n";

    match merge_text(base, ours, theirs) {
        MergeResult::Resolved(content) => {
            assert_eq!(String::from_utf8(content).unwrap(), ours);
        }
        _ => panic!("Expected Resolved"),
    }
}

#[test]
fn test_merge_one_side_deleted_other_kept() {
    let base = "line1\nline2\nline3\n";
    let ours = "line1\nline3\n"; 
    let theirs = "line1\nline2\nline3\n"; 

    match merge_text(base, ours, theirs) {
        MergeResult::Resolved(content) => {
            assert_eq!(String::from_utf8(content).unwrap(), ours);
        }
        _ => panic!("Expected Resolved"),
    }
}

#[test]
fn test_merge_conflict_delete_vs_modify() {
    let base = "line1\nline2\nline3\n";
    let ours = "line1\nline3\n"; 
    let theirs = "line1\nline2_modified\nline3\n"; 

    match merge_text(base, ours, theirs) {
        MergeResult::Conflict { .. } => {}
        _ => panic!("Expected Conflict"),
    }
}
