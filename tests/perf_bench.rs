use castor::core::Registry;
use std::fs;
use std::path::Path;
use std::time::Instant;
use tempfile::tempdir;

fn seed_sessions(base: &Path, count: usize) {
    let chat_dir = base.join("stress_proj").join("chats");
    fs::create_dir_all(&chat_dir).expect("create chats dir");
    for i in 0..count {
        let file = chat_dir.join(format!("session-2026-03-08T12-00-{i:08}.json"));
        fs::write(
            file,
            r#"{"messages":[{"type":"user","content":"stress content"}]}"#,
        )
        .expect("write stress session");
    }
}

fn run_reload_bench(count: usize, threshold_ms: u128) {
    let tmp = tempdir().expect("create tempdir");
    seed_sessions(tmp.path(), count);

    let mut registry = Registry::new(tmp.path(), &tmp.path().join("cache.json"));
    let start = Instant::now();
    registry.reload().expect("registry reload");
    let elapsed = start.elapsed().as_millis();

    assert_eq!(registry.list().len(), count);
    assert!(
        elapsed <= threshold_ms,
        "reload benchmark failed: count={count}, elapsed={elapsed}ms, threshold={threshold_ms}ms"
    );
}

fn threshold(var: &str, default: u128) -> u128 {
    std::env::var(var)
        .ok()
        .and_then(|s| s.parse::<u128>().ok())
        .unwrap_or(default)
}

#[test]
#[ignore = "performance benchmark"]
fn bench_reload_1k() {
    run_reload_bench(1_000, threshold("CASTOR_BENCH_1K_MS", 400));
}

#[test]
#[ignore = "performance benchmark"]
fn bench_reload_5k() {
    run_reload_bench(5_000, threshold("CASTOR_BENCH_5K_MS", 2_500));
}

#[test]
#[ignore = "performance benchmark"]
fn bench_reload_10k() {
    run_reload_bench(10_000, threshold("CASTOR_BENCH_10K_MS", 6_000));
}
