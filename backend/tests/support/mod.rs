use std::collections::HashSet;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Runs `f` with environment variables temporarily modified.
///
/// This is panic-safe (restores variables on unwind) and also serializes access to
/// process-global env vars to avoid flaky tests when Rust runs tests in parallel.
///
/// `changes` is a list of `(key, value)` pairs:
/// - `Some(v)` sets the variable to `v`
/// - `None` removes the variable
pub fn with_scoped_env<F, R>(changes: &[(&str, Option<&str>)], f: F) -> R
where
    F: FnOnce() -> R,
{
    let _lock = ENV_LOCK.lock().expect("ENV_LOCK poisoned");
    let _guard = ScopedEnv::new(changes);
    f()
}

struct ScopedEnv {
    snapshot: Vec<(String, Option<String>)>,
}

impl ScopedEnv {
    fn new(changes: &[(&str, Option<&str>)]) -> Self {
        let keys: HashSet<&str> = changes.iter().map(|(k, _)| *k).collect();
        let snapshot = keys
            .into_iter()
            .map(|k| (k.to_string(), std::env::var(k).ok()))
            .collect::<Vec<_>>();

        for (k, v) in changes {
            match v {
                Some(val) => std::env::set_var(k, val),
                None => std::env::remove_var(k),
            }
        }

        Self { snapshot }
    }
}

impl Drop for ScopedEnv {
    fn drop(&mut self) {
        for (k, v) in self.snapshot.drain(..) {
            match v {
                Some(val) => std::env::set_var(&k, val),
                None => std::env::remove_var(&k),
            }
        }
    }
}
