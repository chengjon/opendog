use crate::error::Result;
use crate::storage::database::Database;
use rusqlite::params;

pub fn count_file_sightings_before(db: &Database, cutoff_ts: i64) -> Result<i64> {
    db.query_row(
        "SELECT COUNT(*) FROM file_sightings WHERE CAST(seen_at AS INTEGER) < ?1",
        params![cutoff_ts],
        |row| row.get(0),
    )
}

pub fn delete_file_sightings_before(db: &Database, cutoff_ts: i64) -> Result<usize> {
    db.execute(
        "DELETE FROM file_sightings WHERE CAST(seen_at AS INTEGER) < ?1",
        params![cutoff_ts],
    )
}

pub fn count_file_events_before(db: &Database, cutoff_ts: i64) -> Result<i64> {
    db.query_row(
        "SELECT COUNT(*) FROM file_events WHERE CAST(event_time AS INTEGER) < ?1",
        params![cutoff_ts],
        |row| row.get(0),
    )
}

pub fn delete_file_events_before(db: &Database, cutoff_ts: i64) -> Result<usize> {
    db.execute(
        "DELETE FROM file_events WHERE CAST(event_time AS INTEGER) < ?1",
        params![cutoff_ts],
    )
}

pub fn count_verification_runs_before(db: &Database, cutoff_ts: i64) -> Result<i64> {
    db.query_row(
        "SELECT COUNT(*) FROM verification_runs WHERE CAST(finished_at AS INTEGER) < ?1",
        params![cutoff_ts],
        |row| row.get(0),
    )
}

pub fn delete_verification_runs_before(db: &Database, cutoff_ts: i64) -> Result<usize> {
    db.execute(
        "DELETE FROM verification_runs WHERE CAST(finished_at AS INTEGER) < ?1",
        params![cutoff_ts],
    )
}

pub fn list_snapshot_run_ids_to_prune(db: &Database, keep_latest: usize) -> Result<Vec<i64>> {
    db.prepare_and_query(
        "SELECT id
         FROM snapshot_runs
         ORDER BY CAST(captured_at AS INTEGER) DESC, id DESC
         LIMIT -1 OFFSET ?1",
        params![keep_latest as i64],
        |row| row.get(0),
    )
}

pub fn count_snapshot_history_for_runs(db: &Database, run_ids: &[i64]) -> Result<i64> {
    if run_ids.is_empty() {
        return Ok(0);
    }
    let (sql, params_vec) = in_clause_sql(
        "SELECT COUNT(*) FROM snapshot_history WHERE run_id IN ({})",
        run_ids,
    );
    db.query_row(&sql, rusqlite::params_from_iter(params_vec), |row| {
        row.get(0)
    })
}

pub fn delete_snapshot_history_for_runs(db: &Database, run_ids: &[i64]) -> Result<usize> {
    if run_ids.is_empty() {
        return Ok(0);
    }
    let (sql, params_vec) =
        in_clause_sql("DELETE FROM snapshot_history WHERE run_id IN ({})", run_ids);
    Ok(db
        .conn()
        .execute(&sql, rusqlite::params_from_iter(params_vec))?)
}

pub fn delete_snapshot_runs_by_ids(db: &Database, run_ids: &[i64]) -> Result<usize> {
    if run_ids.is_empty() {
        return Ok(0);
    }
    let (sql, params_vec) = in_clause_sql("DELETE FROM snapshot_runs WHERE id IN ({})", run_ids);
    Ok(db
        .conn()
        .execute(&sql, rusqlite::params_from_iter(params_vec))?)
}

fn in_clause_sql(prefix: &str, ids: &[i64]) -> (String, Vec<i64>) {
    let placeholders: Vec<String> = (1..=ids.len()).map(|index| format!("?{}", index)).collect();
    (prefix.replace("{}", &placeholders.join(", ")), ids.to_vec())
}

pub fn count_snapshot_runs(db: &Database) -> Result<i64> {
    db.query_row("SELECT COUNT(*) FROM snapshot_runs", rusqlite::params![], |row| {
        row.get(0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::Database;
    use rusqlite::params;

    fn test_db() -> Database {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("retention_test.db");
        let db = Database::open_project(&db_path).unwrap();
        Box::leak(Box::new(dir));
        db
    }

    #[test]
    fn count_and_delete_file_sightings_by_cutoff() {
        let db = test_db();
        let cutoff = 5000i64;
        db.execute(
            "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES ('a.rs', 'codex', 1, '4000')",
            params![],
        )
        .unwrap();
        db.execute(
            "INSERT INTO file_sightings (file_path, process_name, pid, seen_at) VALUES ('b.rs', 'codex', 2, '6000')",
            params![],
        )
        .unwrap();
        assert_eq!(count_file_sightings_before(&db, cutoff).unwrap(), 1);
        assert_eq!(delete_file_sightings_before(&db, cutoff).unwrap(), 1);
        assert_eq!(count_file_sightings_before(&db, cutoff).unwrap(), 0);
    }

    #[test]
    fn count_and_delete_file_events_by_cutoff() {
        let db = test_db();
        db.execute(
            "INSERT INTO file_events (file_path, event_type, event_time) VALUES ('a.rs', 'modify', '100')",
            params![],
        )
        .unwrap();
        db.execute(
            "INSERT INTO file_events (file_path, event_type, event_time) VALUES ('b.rs', 'modify', '900')",
            params![],
        )
        .unwrap();
        assert_eq!(count_file_events_before(&db, 500).unwrap(), 1);
        assert_eq!(delete_file_events_before(&db, 500).unwrap(), 1);
    }

    #[test]
    fn count_and_delete_verification_runs_by_cutoff() {
        let db = test_db();
        db.execute(
            "INSERT INTO verification_runs (kind, status, command, source, finished_at) VALUES ('test', 'passed', 'c', 't', '100')",
            params![],
        )
        .unwrap();
        db.execute(
            "INSERT INTO verification_runs (kind, status, command, source, finished_at) VALUES ('lint', 'passed', 'c', 't', '900')",
            params![],
        )
        .unwrap();
        assert_eq!(count_verification_runs_before(&db, 500).unwrap(), 1);
        assert_eq!(delete_verification_runs_before(&db, 500).unwrap(), 1);
    }

    #[test]
    fn snapshot_run_pruning_keeps_latest() {
        let db = test_db();
        for ts in ["100", "200", "300"] {
            db.execute(
                "INSERT INTO snapshot_runs (captured_at, file_count) VALUES (?1, 1)",
                params![ts],
            )
            .unwrap();
        }
        // Keep latest 1, prune the rest
        let to_prune = list_snapshot_run_ids_to_prune(&db, 1).unwrap();
        assert_eq!(to_prune.len(), 2);

        // Count and delete history
        let history_count = count_snapshot_history_for_runs(&db, &to_prune).unwrap();
        assert_eq!(history_count, 0); // no history rows seeded

        let deleted_runs = delete_snapshot_runs_by_ids(&db, &to_prune).unwrap();
        assert_eq!(deleted_runs, 2);
    }

    #[test]
    fn empty_run_ids_is_noop() {
        let db = test_db();
        assert_eq!(count_snapshot_history_for_runs(&db, &[]).unwrap(), 0);
        assert_eq!(delete_snapshot_history_for_runs(&db, &[]).unwrap(), 0);
        assert_eq!(delete_snapshot_runs_by_ids(&db, &[]).unwrap(), 0);
    }

    #[test]
    fn in_clause_sql_generates_placeholders() {
        let (sql, ids) = in_clause_sql("SELECT * FROM t WHERE id IN ({})", &[10, 20, 30]);
        assert_eq!(sql, "SELECT * FROM t WHERE id IN (?1, ?2, ?3)");
        assert_eq!(ids, vec![10, 20, 30]);
    }

    #[test]
    fn count_snapshot_runs_returns_total_count() {
        let db = test_db();
        assert_eq!(count_snapshot_runs(&db).unwrap(), 0);
        for ts in ["100", "200", "300"] {
            db.execute(
                "INSERT INTO snapshot_runs (captured_at, file_count) VALUES (?1, 1)",
                params![ts],
            )
            .unwrap();
        }
        assert_eq!(count_snapshot_runs(&db).unwrap(), 3);
    }
}
