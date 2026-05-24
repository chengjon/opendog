use crate::error::Result;
use crate::storage::database::Database;
use rusqlite::params;

#[derive(Debug, Clone)]
pub struct DataRiskCache {
    pub mock_candidate_count: usize,
    pub hardcoded_candidate_count: usize,
    pub mixed_review_file_count: usize,
    pub updated_at: String,
}

pub fn upsert_data_risk_cache(
    db: &Database,
    mock_count: usize,
    hardcoded_count: usize,
    mixed_count: usize,
    now: &str,
) -> Result<()> {
    db.execute(
        "INSERT INTO data_risk_cache (id, mock_candidate_count, hardcoded_candidate_count, mixed_review_file_count, updated_at)
         VALUES (1, ?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET
            mock_candidate_count = excluded.mock_candidate_count,
            hardcoded_candidate_count = excluded.hardcoded_candidate_count,
            mixed_review_file_count = excluded.mixed_review_file_count,
            updated_at = excluded.updated_at",
        params![mock_count as i64, hardcoded_count as i64, mixed_count as i64, now],
    )?;
    Ok(())
}

pub fn get_data_risk_cache(db: &Database) -> Result<Option<DataRiskCache>> {
    match db.query_row(
        "SELECT mock_candidate_count, hardcoded_candidate_count, mixed_review_file_count, updated_at
         FROM data_risk_cache WHERE id = 1",
        params![],
        |row| {
            Ok(DataRiskCache {
                mock_candidate_count: row.get::<_, i64>(0)? as usize,
                hardcoded_candidate_count: row.get::<_, i64>(1)? as usize,
                mixed_review_file_count: row.get::<_, i64>(2)? as usize,
                updated_at: row.get(3)?,
            })
        },
    ) {
        Ok(cache) => Ok(Some(cache)),
        Err(crate::error::OpenDogError::Database(rusqlite::Error::QueryReturnedNoRows)) => Ok(None),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Database {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("data_risk_cache_test.db");
        let db = Database::open_project(&db_path).unwrap();
        Box::leak(Box::new(dir));
        db
    }

    #[test]
    fn upsert_and_read_cache() {
        let db = test_db();

        // Empty initially
        assert!(get_data_risk_cache(&db).unwrap().is_none());

        // Insert
        upsert_data_risk_cache(&db, 5, 3, 2, "2026-05-24T12:00:00Z").unwrap();
        let cache = get_data_risk_cache(&db).unwrap().unwrap();
        assert_eq!(cache.mock_candidate_count, 5);
        assert_eq!(cache.hardcoded_candidate_count, 3);
        assert_eq!(cache.mixed_review_file_count, 2);

        // Update
        upsert_data_risk_cache(&db, 10, 7, 4, "2026-05-24T13:00:00Z").unwrap();
        let cache = get_data_risk_cache(&db).unwrap().unwrap();
        assert_eq!(cache.mock_candidate_count, 10);
        assert_eq!(cache.hardcoded_candidate_count, 7);
        assert_eq!(cache.mixed_review_file_count, 4);
        assert_eq!(cache.updated_at, "2026-05-24T13:00:00Z");
    }
}
