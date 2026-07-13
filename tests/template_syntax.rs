use bevy::asset::{Asset, AssetPath, Handle};
use bevy_hui::prelude::{AssetLoadAdaptor, VerboseHtmlError, parse_template};

struct NoopAssetLoader;

impl AssetLoadAdaptor for NoopAssetLoader {
    fn load<'a, A: Asset>(&mut self, _: impl Into<AssetPath<'a>>) -> Handle<A> {
        Handle::default()
    }
}

fn parses(path: &[u8]) {
    parse_template::<VerboseHtmlError>(path, &mut NoopAssetLoader)
        .expect("repro template must be accepted by bevy_hui's asset parser");
}

#[test]
fn bug2_templates_parse() {
    parses(include_bytes!("../assets/bug2_root_scope.html"));
    parses(include_bytes!("../assets/bug2_root_text.html"));
}
