#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub enum Phases {
    StartFrame = 0,
    PlatformUpdate,
    PreUpdate,
    Update,
    PostUpdate,
    PreRender,
    Render,
    PostRender,
    EndFrame,
}

impl Phases {
    pub fn iterator() -> impl Iterator<Item = Phases> {
        [
            Phases::StartFrame,
            Phases::PlatformUpdate,
            Phases::PreUpdate,
            Phases::Update,
            Phases::PostUpdate,
            Phases::PreRender,
            Phases::Render,
            Phases::PostRender,
            Phases::EndFrame,
        ]
        .iter()
        .copied()
    }
}
