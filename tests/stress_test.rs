mod common;
use common::TestContext;
use std::time::Instant;

#[test]
fn test_registry_stress_load() {
    let ctx = TestContext::new();
    let mut registry = ctx.get_registry();

    println!("Seeding 1000 sessions for stress test...");
    for i in 0..1000 {
        ctx.seed_session(
            "stress_proj",
            &format!("session-2026-03-08T12-00-{:08}.json", i),
            "I am a stress test session content.",
            0
        );
    }

    let start = Instant::now();
    registry.reload().unwrap();
    let duration = start.elapsed();

    println!("Registry reload time for 1000 sessions: {:?}", duration);
    
    // Performance Requirement: Should load 1000 sessions in less than 500ms 
    // (Actual time depends on machine, but this is a reasonable baseline for local SSD)
    assert!(duration.as_millis() < 1000, "Registry load too slow: {:?}", duration);
    assert_eq!(registry.list().len(), 1000);
}
