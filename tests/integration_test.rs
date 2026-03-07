mod common;
use castor::ops::{export, prune, stats::StorageStats};
use common::TestContext;
use std::fs;

#[test]
fn test_full_prune_and_restore_cycle() {
    let ctx = TestContext::new();
    let executor = ctx.get_executor();
    let mut registry = ctx.get_registry();

    ctx.seed_session(
        "proj_a",
        "session-2026-03-08T10-00-fresh111.json",
        "I am fresh",
        0,
    );
    ctx.seed_session(
        "proj_a",
        "session-2026-01-01T10-00-oldold22.json",
        "I am old",
        60,
    );

    registry.reload().unwrap();
    assert_eq!(registry.list().len(), 2);

    let to_prune = prune::find_sessions_to_prune(registry.list(), 30);
    assert_eq!(to_prune.len(), 1);
    assert_eq!(to_prune[0].id, "session-2026-01-01T10-00-oldold22.json");

    for s in &to_prune {
        executor.delete_soft(s, false).unwrap();
    }

    registry.reload().unwrap();
    assert_eq!(registry.list().len(), 1);

    executor
        .restore("session-2026-01-01T10-00-oldold22.json", false)
        .unwrap();
    registry.reload().unwrap();
    assert_eq!(registry.list().len(), 2);
}

#[test]
fn test_stats_and_grep() {
    let ctx = TestContext::new();
    let mut registry = ctx.get_registry();

    ctx.seed_session(
        "p1",
        "session-2026-03-08T10-00-aaaa1111.json",
        "Find me: KeywordX",
        0,
    );
    registry.reload().unwrap();

    // Grep
    let matches: Vec<_> = registry
        .list()
        .iter()
        .filter(|s| fs::read_to_string(&s.path).unwrap().contains("KeywordX"))
        .collect();
    assert_eq!(matches.len(), 1);

    // Stats
    let stats = StorageStats::calculate(registry.list(), &ctx.config);
    assert_eq!(stats.total_sessions, 1);
}

#[test]
fn test_export_logic() {
    let ctx = TestContext::new();
    let mut registry = ctx.get_registry();

    ctx.seed_session(
        "p1",
        "session-2026-03-08T10-00-aaaa1111.json",
        "Markdown test",
        0,
    );
    registry.reload().unwrap();

    let session = &registry.list()[0];
    let md = export::session_to_markdown(session).unwrap();

    assert!(md.contains("# Session:"));
    assert!(md.contains("Markdown test"));
}
