use super::*;

#[test]
fn build_empty_buckets_generates_correct_count() {
    let buckets = build_empty_buckets(1000, 5, 60);
    assert_eq!(buckets.len(), 5);
}

#[test]
fn build_empty_buckets_starts_at_start_ts() {
    let buckets = build_empty_buckets(1000, 3, 60);
    assert_eq!(buckets[0].bucket_start, 1000);
}

#[test]
fn build_empty_buckets_spaces_by_bucket_size() {
    let buckets = build_empty_buckets(0, 4, 100);
    let starts: Vec<i64> = buckets.iter().map(|b| b.bucket_start).collect();
    assert_eq!(starts, vec![0, 100, 200, 300]);
}

#[test]
fn build_empty_buckets_all_counts_zero() {
    let buckets = build_empty_buckets(0, 10, 30);
    for bucket in &buckets {
        assert_eq!(bucket.access_count, 0);
        assert_eq!(bucket.modification_count, 0);
    }
}

#[test]
fn build_empty_buckets_single_bucket() {
    let buckets = build_empty_buckets(500, 1, 200);
    assert_eq!(buckets.len(), 1);
    assert_eq!(buckets[0].bucket_start, 500);
}

#[test]
fn build_empty_buckets_zero_buckets() {
    let buckets = build_empty_buckets(0, 0, 60);
    assert!(buckets.is_empty());
}

#[test]
fn bucket_access_count_returns_matching_count() {
    let buckets = vec![
        TrendBucket {
            bucket_start: 0,
            access_count: 5,
            modification_count: 0,
        },
        TrendBucket {
            bucket_start: 60,
            access_count: 10,
            modification_count: 0,
        },
        TrendBucket {
            bucket_start: 120,
            access_count: 3,
            modification_count: 0,
        },
    ];
    assert_eq!(bucket_access_count(&buckets, 60), 10);
}

#[test]
fn bucket_access_count_returns_zero_for_missing() {
    let buckets = vec![TrendBucket {
        bucket_start: 0,
        access_count: 5,
        modification_count: 0,
    }];
    assert_eq!(bucket_access_count(&buckets, 999), 0);
}

#[test]
fn bucket_access_count_empty_slice() {
    let buckets: Vec<TrendBucket> = vec![];
    assert_eq!(bucket_access_count(&buckets, 0), 0);
}

#[test]
fn bucket_access_count_first_bucket() {
    let buckets = vec![
        TrendBucket {
            bucket_start: 100,
            access_count: 7,
            modification_count: 0,
        },
        TrendBucket {
            bucket_start: 200,
            access_count: 8,
            modification_count: 0,
        },
    ];
    assert_eq!(bucket_access_count(&buckets, 100), 7);
}
