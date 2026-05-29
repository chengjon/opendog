use super::*;
use std::cell::Cell;

struct StubDaemon {
    succeed: bool,
    called: Cell<bool>,
}

struct StubLocal {
    called: Cell<bool>,
}

impl ProjectLifecycle for StubDaemon {
    fn create_project(&self, _id: &str, _path: &str) -> Result<ProjectInfo> {
        self.called.set(true);
        if self.succeed {
            Ok(ProjectInfo {
                id: "test".into(),
                root_path: "/tmp".into(),
                db_path: "/tmp/test.db".into(),
                config: crate::config::ProjectConfigOverrides::default(),
                created_at: "0".into(),
                status: "active".into(),
            })
        } else {
            Err(OpenDogError::DaemonUnavailable)
        }
    }
    fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
        self.called.set(true);
        if self.succeed {
            Ok(vec![])
        } else {
            Err(OpenDogError::DaemonUnavailable)
        }
    }
    fn delete_project(&self, _id: &str) -> Result<bool> {
        self.called.set(true);
        if self.succeed {
            Ok(true)
        } else {
            Err(OpenDogError::DaemonUnavailable)
        }
    }
}

impl ProjectLifecycle for StubLocal {
    fn create_project(&self, _id: &str, _path: &str) -> Result<ProjectInfo> {
        self.called.set(true);
        Ok(ProjectInfo {
            id: "test".into(),
            root_path: "/tmp".into(),
            db_path: "/tmp/test.db".into(),
            config: crate::config::ProjectConfigOverrides::default(),
            created_at: "0".into(),
            status: "active".into(),
        })
    }
    fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
        self.called.set(true);
        Ok(vec![])
    }
    fn delete_project(&self, _id: &str) -> Result<bool> {
        self.called.set(true);
        Ok(true)
    }
}

#[test]
fn daemon_success_skips_local_fallback() {
    let daemon = StubDaemon {
        succeed: true,
        called: Cell::new(false),
    };
    let local = StubLocal {
        called: Cell::new(false),
    };
    let lifecycle = FallbackLifecycle::new(daemon, local);

    let result = lifecycle.create_project("p1", "/tmp/p1");
    assert!(result.is_ok());
    assert!(lifecycle.daemon.called.get());
    assert!(!lifecycle.local.called.get());
}

#[test]
fn daemon_unavailable_cascades_to_local() {
    let daemon = StubDaemon {
        succeed: false,
        called: Cell::new(false),
    };
    let local = StubLocal {
        called: Cell::new(false),
    };
    let lifecycle = FallbackLifecycle::new(daemon, local);

    let result = lifecycle.list_projects();
    assert!(result.is_ok());
    assert!(lifecycle.daemon.called.get());
    assert!(lifecycle.local.called.get());
}

#[test]
fn non_daemon_unavailable_error_propagates() {
    struct ErrorDaemon;
    impl ProjectLifecycle for ErrorDaemon {
        fn create_project(&self, _id: &str, _path: &str) -> Result<ProjectInfo> {
            Err(OpenDogError::ProjectNotFound("nope".into()))
        }
        fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
            Ok(vec![])
        }
        fn delete_project(&self, _id: &str) -> Result<bool> {
            Ok(true)
        }
    }
    struct NeverLocal;
    impl ProjectLifecycle for NeverLocal {
        fn create_project(&self, _id: &str, _path: &str) -> Result<ProjectInfo> {
            panic!("local should not be called for non-DaemonUnavailable errors");
        }
        fn list_projects(&self) -> Result<Vec<ProjectInfo>> {
            Ok(vec![])
        }
        fn delete_project(&self, _id: &str) -> Result<bool> {
            Ok(true)
        }
    }

    let lifecycle = FallbackLifecycle::new(ErrorDaemon, NeverLocal);
    let err = lifecycle.create_project("x", "/x").unwrap_err();
    assert!(matches!(err, OpenDogError::ProjectNotFound(_)));
}

#[test]
fn delete_project_cascades_on_daemon_unavailable() {
    let daemon = StubDaemon {
        succeed: false,
        called: Cell::new(false),
    };
    let local = StubLocal {
        called: Cell::new(false),
    };
    let lifecycle = FallbackLifecycle::new(daemon, local);

    let result = lifecycle.delete_project("p1");
    assert!(result.is_ok());
    assert!(lifecycle.daemon.called.get());
    assert!(lifecycle.local.called.get());
}
