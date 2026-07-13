use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_hui::prelude::*;

#[derive(Resource)]
struct HuiRoots(Vec<Entity>);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
            HuiPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (controls, report_dirty)
                .chain()
                .after(bevy_hui::HuiSystems::Style),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    let template = asset_server.load("bug1_rows.html");
    let roots = (0..40)
        .map(|_| commands.spawn(HtmlNode(template.clone())).id())
        .collect();
    commands.insert_resource(HuiRoots(roots));
    info!(
        "BUG 1 repro: 40 static template instances x 20 text rows. SPACE despawns them; H hides their roots."
    );
}

fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    roots: Res<HuiRoots>,
    existing: Query<(), With<HtmlNode>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        for &root in &roots.0 {
            if existing.get(root).is_ok() {
                commands.entity(root).despawn();
            }
        }
        info!("Despawning the hui subtrees (SPACE).");
    }
    if keys.just_pressed(KeyCode::KeyH) {
        for &root in &roots.0 {
            if existing.get(root).is_ok() {
                commands.entity(root).insert(Visibility::Hidden);
            }
        }
        info!(
            "Hiding hui roots with Visibility::Hidden (H); styles still run on every descendant."
        );
    }
}

fn report_dirty(
    time: Res<Time<Real>>,
    mut elapsed: Local<f32>,
    changed_nodes: Query<(), Changed<Node>>,
    changed_fonts: Query<(), Changed<TextFont>>,
    changed_hovers: Query<(), Changed<HoverTimer>>,
    changed_pressed: Query<(), Changed<PressedTimer>>,
    all_nodes: Query<(), With<Node>>,
    text_nodes: Query<(), With<TextFont>>,
) {
    *elapsed += time.delta_secs();
    if *elapsed < 1.0 {
        return;
    }
    *elapsed = 0.0;
    info!(
        total_nodes = all_nodes.iter().count(),
        total_text_nodes = text_nodes.iter().count(),
        changed_node = changed_nodes.iter().count(),
        changed_text_font = changed_fonts.iter().count(),
        changed_hover_timer = changed_hovers.iter().count(),
        changed_pressed_timer = changed_pressed.iter().count(),
        "static hui dirty-counts (these should become zero after startup)"
    );
}
