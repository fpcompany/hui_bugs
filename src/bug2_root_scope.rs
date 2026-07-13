use bevy::prelude::*;
use bevy_hui::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, HuiPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, report_result.after(bevy_hui::HuiSystems::Style))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn((
        HtmlNode(asset_server.load("bug2_root_scope.html")),
        TemplateProperties::default().with("bg", "#e63946"),
    ));
    commands.spawn((
        HtmlNode(asset_server.load("bug2_root_text.html")),
        TemplateProperties::default().with("label", "ROOT TEXT SHOULD BE COMPILED"),
    ));
    info!(
        "BUG 2 repro: the upper outlined panel proves root/child style compilation; the lower panel proves root text compilation."
    );
}

fn report_result(mut printed: Local<bool>, nodes: Query<(&UiId, &BackgroundColor, Option<&Text>)>) {
    if *printed {
        return;
    }
    let mut style_root = None;
    let mut style_child = None;
    let mut text_root = None;
    for (id, color, text) in &nodes {
        match id.id().as_str() {
            "style-root" => style_root = Some(color.0),
            "style-child" => style_child = Some(color.0),
            "text-root" => text_root = Some((color.0, text.map(|text| text.0.as_str()))),
            _ => {}
        }
    }
    if let (Some(style_root_color), Some(style_child_color), Some((text_root_color, root_text))) =
        (style_root, style_child, text_root)
    {
        info!(
            ?style_root_color,
            ?style_child_color,
            ?text_root_color,
            ?root_text,
            "BUG 2 result: style-root retains default before the fix while style-child is red; text-root retains {{label}} before the fix"
        );
        *printed = true;
    }
}
