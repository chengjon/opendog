use super::*;

#[test]
fn storage_maintenance_layer_empty_projects() {
    let result = storage_maintenance_layer(&[]);
    assert_eq!(result["status"], "available");
    assert_eq!(result["projects_with_candidates"], 0);
    assert_eq!(result["projects_with_vacuum_candidates"], 0);
    assert_eq!(result["total_approx_db_size_bytes"], 0);
    assert_eq!(result["total_approx_reclaimable_bytes"], 0);
    assert!(result["priority_projects"].as_array().unwrap().is_empty());
}

#[test]
fn storage_maintenance_layer_aggregates_sizes() {
    let projects = vec![
        json!({
            "project_id": "proj_a",
            "status": "available",
            "storage_maintenance": {
                "maintenance_candidate": false,
                "vacuum_candidate": false,
                "cleanup_review_candidate": false,
                "approx_db_size_bytes": 1_000_000,
                "approx_reclaimable_bytes": 100_000,
                "reclaim_ratio": 0.1,
                "suggested_mode": "none",
                "summary": "small",
            }
        }),
        json!({
            "project_id": "proj_b",
            "status": "available",
            "storage_maintenance": {
                "maintenance_candidate": false,
                "vacuum_candidate": false,
                "cleanup_review_candidate": false,
                "approx_db_size_bytes": 2_000_000,
                "approx_reclaimable_bytes": 200_000,
                "reclaim_ratio": 0.1,
                "suggested_mode": "none",
                "summary": "small",
            }
        }),
    ];
    let result = storage_maintenance_layer(&projects);
    assert_eq!(result["total_approx_db_size_bytes"], 3_000_000);
    assert_eq!(result["total_approx_reclaimable_bytes"], 300_000);
    assert_eq!(result["projects_with_candidates"], 0);
}

#[test]
fn storage_maintenance_layer_filters_candidates() {
    let projects = vec![
        json!({
            "project_id": "clean_project",
            "status": "available",
            "storage_maintenance": {
                "maintenance_candidate": false,
                "vacuum_candidate": false,
                "cleanup_review_candidate": false,
                "approx_db_size_bytes": 500_000,
                "approx_reclaimable_bytes": 0,
                "reclaim_ratio": 0.0,
                "suggested_mode": "none",
                "summary": "no issues",
            }
        }),
        json!({
            "project_id": "dirty_project",
            "status": "available",
            "storage_maintenance": {
                "maintenance_candidate": true,
                "vacuum_candidate": true,
                "cleanup_review_candidate": true,
                "approx_db_size_bytes": 50_000_000,
                "approx_reclaimable_bytes": 25_000_000,
                "reclaim_ratio": 0.5,
                "suggested_mode": "review_cleanup_then_vacuum",
                "summary": "needs cleanup",
            }
        }),
    ];
    let result = storage_maintenance_layer(&projects);
    assert_eq!(result["projects_with_candidates"], 1);
    assert_eq!(result["projects_with_vacuum_candidates"], 1);
    let priority = result["priority_projects"].as_array().unwrap();
    assert_eq!(priority.len(), 1);
    assert_eq!(priority[0]["project_id"], "dirty_project");
}

#[test]
fn storage_maintenance_layer_truncates_to_five() {
    let mut projects = Vec::new();
    for i in 0..8 {
        projects.push(json!({
            "project_id": format!("proj_{}", i),
            "status": "available",
            "storage_maintenance": {
                "maintenance_candidate": true,
                "vacuum_candidate": i % 2 == 0,
                "cleanup_review_candidate": true,
                "approx_db_size_bytes": 30_000_000 + i as i64 * 1_000_000,
                "approx_reclaimable_bytes": 15_000_000 + i as i64 * 500_000,
                "reclaim_ratio": 0.5,
                "suggested_mode": "review_cleanup_then_vacuum",
                "summary": "needs attention",
            }
        }));
    }
    let result = storage_maintenance_layer(&projects);
    let priority = result["priority_projects"].as_array().unwrap();
    assert_eq!(priority.len(), 5);
}

#[test]
fn storage_maintenance_layer_sorts_vacuum_candidates_first() {
    let projects = vec![
        json!({
            "project_id": "no_vacuum",
            "status": "available",
            "storage_maintenance": {
                "maintenance_candidate": true,
                "vacuum_candidate": false,
                "cleanup_review_candidate": true,
                "approx_db_size_bytes": 30_000_000,
                "approx_reclaimable_bytes": 2_000_000,
                "reclaim_ratio": 0.07,
                "suggested_mode": "review_cleanup",
                "summary": "cleanup only",
            }
        }),
        json!({
            "project_id": "with_vacuum",
            "status": "available",
            "storage_maintenance": {
                "maintenance_candidate": true,
                "vacuum_candidate": true,
                "cleanup_review_candidate": true,
                "approx_db_size_bytes": 30_000_000,
                "approx_reclaimable_bytes": 2_000_000,
                "reclaim_ratio": 0.07,
                "suggested_mode": "review_cleanup_then_vacuum",
                "summary": "vacuum needed",
            }
        }),
    ];
    let result = storage_maintenance_layer(&projects);
    let priority = result["priority_projects"].as_array().unwrap();
    assert_eq!(priority.len(), 2);
    // Vacuum candidate should be first
    assert_eq!(priority[0]["project_id"], "with_vacuum");
    assert_eq!(priority[1]["project_id"], "no_vacuum");
}

// --- storage_maintenance_execution_templates tests ---
