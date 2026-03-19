use chrono::Utc;
use uuid::Uuid;

use super::{Artifact, CreateTaskRequest, TaskEngine, TaskFilter, TaskStatus};
use crate::station::db;

fn make_request(description: &str) -> CreateTaskRequest {
    CreateTaskRequest {
        description: description.to_string(),
        priority: None,
    }
}

// ---------------------------------------------------------------------------
// Task lifecycle tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_create_task() {
    let engine = TaskEngine::new();
    let task = engine.create(make_request("Analyse user data")).await;

    assert!(!task.id.is_empty());
    // ID should be a valid UUID.
    Uuid::parse_str(&task.id).expect("id should be a valid UUID");
    assert_eq!(task.title, "Analyse user data");
    assert_eq!(task.status, TaskStatus::Pending);
    assert_eq!(task.priority, 0);
    assert!(task.agent_id.is_none());
    assert!(task.parent_id.is_none());
    assert!(task.input.is_none());
    assert!(task.output.is_none());
    assert!(task.sub_tasks.is_empty());
    assert!(task.started_at.is_none());
    assert!(task.completed_at.is_none());
}

#[tokio::test]
async fn test_create_task_with_priority() {
    let engine = TaskEngine::new();
    let task = engine
        .create(CreateTaskRequest {
            description: "High priority task".to_string(),
            priority: Some(10),
        })
        .await;

    assert_eq!(task.priority, 10);
}

#[tokio::test]
async fn test_start_task() {
    let engine = TaskEngine::new();
    let created = engine.create(make_request("Run analysis")).await;
    let started = engine.start(&created.id).await.unwrap();

    assert_eq!(started.status, TaskStatus::Running);
    assert!(started.started_at.is_some());

    // Verify via status() too.
    let fetched = engine.status(&created.id).await.unwrap();
    assert_eq!(fetched.status, TaskStatus::Running);
    assert!(fetched.started_at.is_some());
}

#[tokio::test]
async fn test_pause_task() {
    let engine = TaskEngine::new();
    let created = engine.create(make_request("Long running job")).await;
    engine.start(&created.id).await.unwrap();
    let paused = engine.pause(&created.id).await.unwrap();

    assert_eq!(paused.status, TaskStatus::Paused);
}

#[tokio::test]
async fn test_cancel_from_pending() {
    let engine = TaskEngine::new();
    let created = engine.create(make_request("Will be cancelled")).await;
    let cancelled = engine.cancel(&created.id).await.unwrap();

    assert_eq!(cancelled.status, TaskStatus::Cancelled);
    assert!(cancelled.completed_at.is_some());
}

#[tokio::test]
async fn test_cancel_from_running() {
    let engine = TaskEngine::new();
    let created = engine.create(make_request("Will be cancelled")).await;
    engine.start(&created.id).await.unwrap();
    let cancelled = engine.cancel(&created.id).await.unwrap();

    assert_eq!(cancelled.status, TaskStatus::Cancelled);
    assert!(cancelled.completed_at.is_some());
}

#[tokio::test]
async fn test_complete_task() {
    let engine = TaskEngine::new();
    let created = engine.create(make_request("Compute result")).await;
    engine.start(&created.id).await.unwrap();

    let output = serde_json::json!({ "result": 42, "summary": "done" });
    let completed = engine.complete(&created.id, output.clone()).await.unwrap();

    assert_eq!(completed.status, TaskStatus::Completed);
    assert_eq!(completed.output, Some(output));
    assert!(completed.completed_at.is_some());
}

#[tokio::test]
async fn test_fail_task() {
    let engine = TaskEngine::new();
    let created = engine.create(make_request("Doomed task")).await;
    engine.start(&created.id).await.unwrap();

    let failed = engine.fail(&created.id, "out of memory").await.unwrap();

    assert_eq!(failed.status, TaskStatus::Failed);
    assert!(failed.completed_at.is_some());
    let err_output = failed.output.unwrap();
    assert_eq!(err_output["error"], "out of memory");
}

#[tokio::test]
async fn test_fail_from_pending() {
    let engine = TaskEngine::new();
    let created = engine.create(make_request("Fail early")).await;
    let failed = engine.fail(&created.id, "invalid input").await.unwrap();

    assert_eq!(failed.status, TaskStatus::Failed);
    assert!(failed.completed_at.is_some());
}

// ---------------------------------------------------------------------------
// Invalid transition tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_invalid_start_already_running() {
    let engine = TaskEngine::new();
    let created = engine.create(make_request("Already running")).await;
    engine.start(&created.id).await.unwrap();

    let err = engine.start(&created.id).await.unwrap_err();
    assert!(err.contains("cannot start"));
    assert!(err.to_lowercase().contains("running"));
}

#[tokio::test]
async fn test_invalid_pause_pending() {
    let engine = TaskEngine::new();
    let created = engine.create(make_request("Not started")).await;

    let err = engine.pause(&created.id).await.unwrap_err();
    assert!(err.contains("cannot pause"));
    assert!(err.to_lowercase().contains("pending"));
}

#[tokio::test]
async fn test_invalid_complete_pending() {
    let engine = TaskEngine::new();
    let created = engine.create(make_request("Not started")).await;

    let err = engine
        .complete(&created.id, serde_json::json!({}))
        .await
        .unwrap_err();
    assert!(err.contains("cannot complete"));
}

#[tokio::test]
async fn test_invalid_cancel_completed() {
    let engine = TaskEngine::new();
    let created = engine.create(make_request("Already done")).await;
    engine.start(&created.id).await.unwrap();
    engine
        .complete(&created.id, serde_json::json!({}))
        .await
        .unwrap();

    let err = engine.cancel(&created.id).await.unwrap_err();
    assert!(err.contains("cannot cancel"));
}

#[tokio::test]
async fn test_invalid_fail_completed() {
    let engine = TaskEngine::new();
    let created = engine.create(make_request("Already done")).await;
    engine.start(&created.id).await.unwrap();
    engine
        .complete(&created.id, serde_json::json!({}))
        .await
        .unwrap();

    let err = engine.fail(&created.id, "too late").await.unwrap_err();
    assert!(err.contains("cannot fail"));
}

#[tokio::test]
async fn test_start_nonexistent_task() {
    let engine = TaskEngine::new();
    let err = engine.start("nonexistent").await.unwrap_err();
    assert!(err.contains("task not found"));
}

// ---------------------------------------------------------------------------
// List & filter tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_tasks() {
    let engine = TaskEngine::new();
    engine.create(make_request("Task A")).await;
    engine.create(make_request("Task B")).await;
    engine.create(make_request("Task C")).await;

    let all = engine.list(None).await;
    assert_eq!(all.len(), 3);
}

#[tokio::test]
async fn test_list_with_status_filter() {
    let engine = TaskEngine::new();
    let a = engine.create(make_request("Task A")).await;
    let b = engine.create(make_request("Task B")).await;
    engine.create(make_request("Task C")).await;

    engine.start(&a.id).await.unwrap();
    engine.start(&b.id).await.unwrap();

    let running = engine
        .list(Some(TaskFilter {
            status: Some(TaskStatus::Running),
            ..Default::default()
        }))
        .await;
    assert_eq!(running.len(), 2);
    assert!(running.iter().all(|t| t.status == TaskStatus::Running));

    let pending = engine
        .list(Some(TaskFilter {
            status: Some(TaskStatus::Pending),
            ..Default::default()
        }))
        .await;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].title, "Task C");
}

#[tokio::test]
async fn test_list_with_empty_filter() {
    let engine = TaskEngine::new();
    engine.create(make_request("Task A")).await;

    let all = engine.list(Some(TaskFilter::default())).await;
    assert_eq!(all.len(), 1);
}

// ---------------------------------------------------------------------------
// Sub-task tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_subtasks() {
    let engine = TaskEngine::new();
    let parent = engine.create(make_request("Parent task")).await;

    let child1 = engine.create(make_request("Sub-task 1")).await;
    let child2 = engine.create(make_request("Sub-task 2")).await;

    let id1 = engine
        .add_subtask(&parent.id, child1.clone())
        .await
        .unwrap();
    let id2 = engine
        .add_subtask(&parent.id, child2.clone())
        .await
        .unwrap();

    assert_eq!(id1, child1.id);
    assert_eq!(id2, child2.id);

    // Verify parent's sub_tasks list.
    let updated_parent = engine.status(&parent.id).await.unwrap();
    assert_eq!(updated_parent.sub_tasks.len(), 2);
    assert!(updated_parent.sub_tasks.contains(&child1.id));
    assert!(updated_parent.sub_tasks.contains(&child2.id));

    // Verify get_subtasks.
    let subtasks = engine.get_subtasks(&parent.id).await;
    assert_eq!(subtasks.len(), 2);

    // Verify children have parent_id set.
    let fetched_child = engine.status(&child1.id).await.unwrap();
    assert_eq!(fetched_child.parent_id, Some(parent.id.clone()));
}

#[tokio::test]
async fn test_add_subtask_nonexistent_parent() {
    let engine = TaskEngine::new();
    let child = engine.create(make_request("Orphan")).await;

    let err = engine.add_subtask("nonexistent", child).await.unwrap_err();
    assert!(err.contains("parent task not found"));
}

#[tokio::test]
async fn test_get_subtasks_nonexistent_parent() {
    let engine = TaskEngine::new();
    let subtasks = engine.get_subtasks("nonexistent").await;
    assert!(subtasks.is_empty());
}

// ---------------------------------------------------------------------------
// Artifact tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_artifacts() {
    let engine = TaskEngine::new();
    let task = engine.create(make_request("Generate report")).await;

    let artifact1 = Artifact {
        id: Uuid::new_v4().to_string(),
        task_id: task.id.clone(),
        agent_id: "agent-1".to_string(),
        artifact_type: "document".to_string(),
        name: "report.pdf".to_string(),
        path: "/tmp/report.pdf".to_string(),
        mime_type: Some("application/pdf".to_string()),
        size_bytes: Some(1024),
        created_at: Utc::now(),
    };

    let artifact2 = Artifact {
        id: Uuid::new_v4().to_string(),
        task_id: task.id.clone(),
        agent_id: "agent-1".to_string(),
        artifact_type: "log".to_string(),
        name: "execution.log".to_string(),
        path: "/tmp/execution.log".to_string(),
        mime_type: Some("text/plain".to_string()),
        size_bytes: Some(256),
        created_at: Utc::now(),
    };

    let id1 = engine.add_artifact(artifact1.clone()).await;
    let id2 = engine.add_artifact(artifact2.clone()).await;

    assert_eq!(id1, artifact1.id);
    assert_eq!(id2, artifact2.id);

    let artifacts = engine.get_artifacts(&task.id).await;
    assert_eq!(artifacts.len(), 2);
    assert_eq!(artifacts[0].name, "report.pdf");
    assert_eq!(artifacts[1].name, "execution.log");
}

#[tokio::test]
async fn test_get_artifacts_empty() {
    let engine = TaskEngine::new();
    let artifacts = engine.get_artifacts("nonexistent").await;
    assert!(artifacts.is_empty());
}

// ---------------------------------------------------------------------------
// Count-by-status tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_count_by_status() {
    let engine = TaskEngine::new();

    // Create 3 pending tasks.
    let a = engine.create(make_request("A")).await;
    let b = engine.create(make_request("B")).await;
    engine.create(make_request("C")).await;

    // Move some to different states.
    engine.start(&a.id).await.unwrap();
    engine.start(&b.id).await.unwrap();
    engine.complete(&a.id, serde_json::json!({})).await.unwrap();

    let counts = engine.count_by_status().await;
    assert_eq!(counts.get(&TaskStatus::Pending), Some(&1));
    assert_eq!(counts.get(&TaskStatus::Running), Some(&1));
    assert_eq!(counts.get(&TaskStatus::Completed), Some(&1));
    assert_eq!(counts.get(&TaskStatus::Paused), None);
    assert_eq!(counts.get(&TaskStatus::Failed), None);
    assert_eq!(counts.get(&TaskStatus::Cancelled), None);
}

// ---------------------------------------------------------------------------
// Serialization tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_task_status_serde() {
    let status = TaskStatus::Running;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"running\"");

    let back: TaskStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(back, TaskStatus::Running);
}

#[tokio::test]
async fn test_task_spec_serde_roundtrip() {
    let engine = TaskEngine::new();
    let task = engine.create(make_request("Serde test")).await;

    let json = serde_json::to_string(&task).unwrap();
    let back: super::TaskSpec = serde_json::from_str(&json).unwrap();

    assert_eq!(back.id, task.id);
    assert_eq!(back.title, task.title);
    assert_eq!(back.status, task.status);
    assert_eq!(back.priority, task.priority);
}

#[tokio::test]
async fn test_artifact_serde_roundtrip() {
    let artifact = Artifact {
        id: Uuid::new_v4().to_string(),
        task_id: "task-1".to_string(),
        agent_id: "agent-1".to_string(),
        artifact_type: "code".to_string(),
        name: "main.rs".to_string(),
        path: "/tmp/main.rs".to_string(),
        mime_type: Some("text/x-rust".to_string()),
        size_bytes: Some(512),
        created_at: Utc::now(),
    };

    let json = serde_json::to_string(&artifact).unwrap();
    let back: Artifact = serde_json::from_str(&json).unwrap();

    assert_eq!(back.id, artifact.id);
    assert_eq!(back.task_id, artifact.task_id);
    assert_eq!(back.artifact_type, "code");
    assert_eq!(back.mime_type, Some("text/x-rust".to_string()));
}

// ---------------------------------------------------------------------------
// SQLite persistence tests (D1D-267)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_task_persisted_to_sqlite() {
    let db_handle = db::init_memory().expect("init in-memory db");
    let engine = TaskEngine::with_db(db_handle.clone());

    let task = engine.create(make_request("Persist me")).await;

    // Verify the task was written to SQLite.
    let conn = db_handle.lock().unwrap();
    let title: String = conn
        .query_row(
            "SELECT title FROM tasks WHERE id = ?1",
            rusqlite::params![task.id],
            |row| row.get(0),
        )
        .expect("task should exist in db");
    assert_eq!(title, "Persist me");

    let status: String = conn
        .query_row(
            "SELECT status FROM tasks WHERE id = ?1",
            rusqlite::params![task.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(status, "pending");
}

#[tokio::test]
async fn test_task_status_updates_persisted() {
    let db_handle = db::init_memory().expect("init in-memory db");
    let engine = TaskEngine::with_db(db_handle.clone());

    let task = engine.create(make_request("Track status")).await;
    engine.start(&task.id).await.unwrap();

    // Verify running status persisted.
    {
        let conn = db_handle.lock().unwrap();
        let status: String = conn
            .query_row(
                "SELECT status FROM tasks WHERE id = ?1",
                rusqlite::params![task.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "running");
    }

    engine
        .complete(&task.id, serde_json::json!({"result": "ok"}))
        .await
        .unwrap();

    // Verify completed status persisted.
    {
        let conn = db_handle.lock().unwrap();
        let status: String = conn
            .query_row(
                "SELECT status FROM tasks WHERE id = ?1",
                rusqlite::params![task.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "completed");

        let output: Option<String> = conn
            .query_row(
                "SELECT output FROM tasks WHERE id = ?1",
                rusqlite::params![task.id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(output.unwrap().contains("ok"));
    }
}

#[tokio::test]
async fn test_task_roundtrip_save_load() {
    let db_handle = db::init_memory().expect("init in-memory db");

    // Engine 1: create tasks and transition them.
    let engine1 = TaskEngine::with_db(db_handle.clone());
    let t1 = engine1.create(make_request("Pending task")).await;
    let t2 = engine1.create(make_request("Running task")).await;
    let t3 = engine1.create(make_request("Completed task")).await;
    engine1.start(&t2.id).await.unwrap();
    engine1.start(&t3.id).await.unwrap();
    engine1
        .complete(&t3.id, serde_json::json!({}))
        .await
        .unwrap();

    // Engine 2: simulate app restart — load from same DB.
    let engine2 = TaskEngine::with_db(db_handle);
    let loaded = engine2.load_from_db().await.unwrap();

    // Should load pending + running (not completed).
    assert_eq!(loaded, 2);

    let pending = engine2.status(&t1.id).await;
    assert!(pending.is_some());
    assert_eq!(pending.unwrap().status, TaskStatus::Pending);

    let running = engine2.status(&t2.id).await;
    assert!(running.is_some());
    assert_eq!(running.unwrap().status, TaskStatus::Running);

    // Completed task should NOT be loaded.
    let completed = engine2.status(&t3.id).await;
    assert!(completed.is_none());
}

#[tokio::test]
async fn test_task_pause_cancel_persisted() {
    let db_handle = db::init_memory().expect("init in-memory db");
    let engine = TaskEngine::with_db(db_handle.clone());

    let task = engine.create(make_request("Pausable")).await;
    engine.start(&task.id).await.unwrap();
    engine.pause(&task.id).await.unwrap();

    {
        let conn = db_handle.lock().unwrap();
        let status: String = conn
            .query_row(
                "SELECT status FROM tasks WHERE id = ?1",
                rusqlite::params![task.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "paused");
    }

    // Create another task to cancel.
    let task2 = engine.create(make_request("Cancellable")).await;
    engine.cancel(&task2.id).await.unwrap();

    {
        let conn = db_handle.lock().unwrap();
        let status: String = conn
            .query_row(
                "SELECT status FROM tasks WHERE id = ?1",
                rusqlite::params![task2.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "cancelled");
    }
}

#[tokio::test]
async fn test_subtask_persisted_with_parent_id() {
    let db_handle = db::init_memory().expect("init in-memory db");
    let engine = TaskEngine::with_db(db_handle.clone());

    let parent = engine.create(make_request("Parent")).await;
    let child = engine.create(make_request("Child")).await;
    engine
        .add_subtask(&parent.id, child.clone())
        .await
        .unwrap();

    // Verify the child's parent_id is set in the database.
    let conn = db_handle.lock().unwrap();
    let parent_id: Option<String> = conn
        .query_row(
            "SELECT parent_id FROM tasks WHERE id = ?1",
            rusqlite::params![child.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(parent_id, Some(parent.id));
}

#[tokio::test]
async fn test_load_rebuilds_subtask_refs() {
    let db_handle = db::init_memory().expect("init in-memory db");

    let engine1 = TaskEngine::with_db(db_handle.clone());
    let parent = engine1.create(make_request("Parent")).await;
    let child = engine1.create(make_request("Child")).await;
    let child_id = child.id.clone();
    engine1.add_subtask(&parent.id, child).await.unwrap();

    // Simulate restart.
    let engine2 = TaskEngine::with_db(db_handle);
    engine2.load_from_db().await.unwrap();

    // Parent should have the child in sub_tasks.
    let loaded_parent = engine2.status(&parent.id).await.unwrap();
    assert!(loaded_parent.sub_tasks.contains(&child_id));
}
