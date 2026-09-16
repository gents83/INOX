use inox_resources::{swap_resource, ResourceHandle, ResourceTrait};
use inox_uid::generate_random_uid;

#[derive(Clone, Debug, PartialEq, Eq)]
struct FakeReloadable {
    active: Option<&'static str>,
    pending: Option<&'static str>,
    pending_valid: bool,
    diagnostic: Option<String>,
}

impl ResourceTrait for FakeReloadable {
    fn is_initialized(&self) -> bool {
        self.active.is_some()
    }

    fn invalidate(&mut self) -> &mut Self {
        self
    }

    fn replace_from(&mut self, other: &Self) {
        self.pending = other.active;
        self.pending_valid = other.pending_valid;
        self.diagnostic = other.diagnostic.clone();
    }
}

impl FakeReloadable {
    fn commit(&mut self) {
        if self.pending_valid {
            self.active = self.pending.take();
            self.diagnostic = None;
        }
    }
}

fn resource(
    value: &'static str,
    valid: bool,
    diagnostic: Option<&str>,
) -> std::sync::Arc<ResourceHandle<FakeReloadable>> {
    std::sync::Arc::new(ResourceHandle::new(
        generate_random_uid(),
        FakeReloadable {
            active: Some(value),
            pending: None,
            pending_valid: valid,
            diagnostic: diagnostic.map(str::to_string),
        },
    ))
}

#[test]
fn invalid_shader_candidate_keeps_active_resource_and_diagnostic() {
    let active = resource("valid-shader", true, None);
    let invalid = resource("invalid-shader", false, Some("WGSL syntax error"));

    swap_resource(&active, &invalid);

    let state = active.get();
    assert_eq!(state.active, Some("valid-shader"));
    assert_eq!(state.pending, Some("invalid-shader"));
    assert_eq!(state.diagnostic.as_deref(), Some("WGSL syntax error"));
    assert!(state.is_initialized());
}

#[test]
fn invalid_pipeline_then_valid_candidate_commits_later() {
    let active = resource("valid-pipeline", true, None);
    let invalid = resource("invalid-pipeline", false, Some("pipeline layout mismatch"));
    swap_resource(&active, &invalid);

    assert_eq!(active.get().active, Some("valid-pipeline"));
    assert_eq!(
        active.get().diagnostic.as_deref(),
        Some("pipeline layout mismatch")
    );

    let valid = resource("fixed-pipeline", true, None);
    swap_resource(&active, &valid);
    active.get_mut().commit();

    let state = active.get();
    assert_eq!(state.active, Some("fixed-pipeline"));
    assert_eq!(state.pending, None);
    assert_eq!(state.diagnostic, None);
    assert!(state.is_initialized());
}
