use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::sprite::Mesh2dHandle;

use crate::common::resources::ColorPalette;
use crate::game_rules::resources::PieceGenerator;
use crate::game_rules::resources::Score;
use crate::game_rules::resources::XP;
use crate::ui_grid::resources::MaterialCollection as GridMaterialCollection;
use crate::ui_grid::resources::MeshCollection as GridMeshCollection;

use super::components::*;
use super::resources::*;
use super::UI_SIDE_VIRTUAL_WIDTH;

pub(crate) fn setup_background(
    mut commands: Commands,
    meshes: Res<MeshCollection>,
    palette: Res<ColorPalette>,
    root: Res<UiSideRoot>,
) {
    commands
        .spawn((
            Name::new("Side Background"),
            ColorMesh2dBundle {
                mesh: meshes.background.clone().into(),
                material: palette.background_2.material.clone(),
                transform: Transform::from_translation([0.0, 0.0, -100.0].into()),
                ..Default::default()
            },
        ))
        .set_parent(**root);
}

pub(crate) fn setup_preview(
    mut commands: Commands,
    meshes: Res<MeshCollection>,
    palette: Res<ColorPalette>,
    root: Res<UiSideRoot>,
) {
    let preview = commands
        .spawn((
            Name::new("Next Piece Frame"),
            ColorMesh2dBundle {
                mesh: meshes.preview_box.clone().into(),
                material: palette.background_1.material.clone(),
                transform: Transform::from_translation([0.0, 0.0, -100.0].into())
                    .with_scale(Vec3::new(0.8, 0.8, 1.0)),
                ..Default::default()
            },
        ))
        .set_parent(**root)
        .id();

    commands
        .spawn((
            Name::new("Next Piece"),
            NextPiece,
            ColorMesh2dBundle {
                transform: Transform::from_translation([0.0, 0.0, 50.0].into()),
                ..Default::default()
            },
        ))
        .set_parent(preview);
}

pub(crate) fn setup_score_pannel(
    mut commands: Commands,
    root: Res<UiSideRoot>,
    fonts: Res<FontsCollection>,
    palette: Res<ColorPalette>,
) {
    commands
        .spawn((
            Name::new("Score Label"),
            Text2dBundle {
                text: Text::from_section(
                    "Score",
                    TextStyle {
                        font_size: 32.0,
                        color: palette.text_title.color,
                        font: fonts.title.clone(),
                    },
                )
                .with_no_wrap(),
                transform: Transform::from_translation(
                    [10.0 - UI_SIDE_VIRTUAL_WIDTH / 2.0, -320.0, 0.0].into(),
                ),
                text_anchor: Anchor::TopLeft,
                ..Default::default()
            },
        ))
        .set_parent(**root);

    commands
        .spawn((
            Name::new("Score Display"),
            ScoreDisplay,
            Text2dBundle {
                text: Text::from_section(
                    "0",
                    TextStyle {
                        font_size: 32.0,
                        color: palette.text_default.color,
                        font: fonts.default.clone(),
                    },
                ),
                transform: Transform::from_translation(
                    [10.0 - UI_SIDE_VIRTUAL_WIDTH / 2.0, -350.0, 0.].into(),
                ),
                text_anchor: Anchor::TopLeft,
                ..Default::default()
            },
        ))
        .set_parent(**root);

    commands
        .spawn((
            Name::new("Level Label"),
            Text2dBundle {
                text: Text::from_section(
                    "Level",
                    TextStyle {
                        font_size: 32.0,
                        color: palette.text_title.color,
                        font: fonts.title.clone(),
                    },
                )
                .with_no_wrap(),
                transform: Transform::from_translation(
                    [10.0 - UI_SIDE_VIRTUAL_WIDTH / 2.0, -240.0, 0.0].into(),
                ),
                text_anchor: Anchor::TopLeft,
                ..Default::default()
            },
        ))
        .set_parent(**root);

    commands
        .spawn((
            Name::new("Level Display"),
            LevelDisplay,
            Text2dBundle {
                text: Text::from_section(
                    "1",
                    TextStyle {
                        font_size: 32.0,
                        color: palette.text_default.color,
                        font: fonts.default.clone(),
                    },
                ),
                transform: Transform::from_translation(
                    [10.0 - UI_SIDE_VIRTUAL_WIDTH / 2.0, -270.0, 0.].into(),
                ),
                text_anchor: Anchor::TopLeft,
                ..Default::default()
            },
        ))
        .set_parent(**root);

    commands
        .spawn((
            Name::new("Time Label"),
            Text2dBundle {
                text: Text::from_section(
                    "Time",
                    TextStyle {
                        font_size: 32.0,
                        color: palette.text_title.color,
                        font: fonts.title.clone(),
                    },
                )
                .with_no_wrap(),
                transform: Transform::from_translation(
                    [10.0 - UI_SIDE_VIRTUAL_WIDTH / 2.0, -160.0, 0.0].into(),
                ),
                text_anchor: Anchor::TopLeft,
                ..Default::default()
            },
        ))
        .set_parent(**root);

    commands
        .spawn((
            Name::new("Time Display"),
            Text2dBundle {
                text: Text::from_section(
                    "00:00:00",
                    TextStyle {
                        font_size: 32.0,
                        color: palette.text_default.color,
                        font: fonts.default.clone(),
                    },
                ),
                transform: Transform::from_translation(
                    [10.0 - UI_SIDE_VIRTUAL_WIDTH / 2.0, -190.0, 0.].into(),
                ),
                text_anchor: Anchor::TopLeft,
                ..Default::default()
            },
        ))
        .set_parent(**root);
}

pub(crate) fn udpate_score_display(mut text: Query<Mut<Text>, With<ScoreDisplay>>, score: Res<Score>) {
    if !score.is_changed() {
        return;
    }

    for mut text in &mut text {
        text.sections[0].value = score.to_string();
    }
}

pub(crate) fn udpate_level_display(mut text: Query<Mut<Text>, With<LevelDisplay>>, xp: Res<XP>) {
    if !xp.is_changed() {
        return;
    }

    for mut text in &mut text {
        text.sections[0].value = xp.level().to_string();
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn update_next_piece(
    mut rng: ResMut<PieceGenerator>,
    mut previews: Query<(Mut<Mesh2dHandle>, Mut<Handle<ColorMaterial>>), With<NextPiece>>,
    grid_meshes: Res<GridMeshCollection>,
    grid_materials: Res<GridMaterialCollection>,
) {
    let next_piece = rng.peek();

    for (mut mesh, mut material) in &mut previews {
        *mesh = grid_meshes.pieces_small_blocks[next_piece].clone().into();
        *material = grid_materials.pieces[next_piece].clone();
    }
}
