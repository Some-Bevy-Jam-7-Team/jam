//! Handles the animated expressions for the intro CRT

use bevy::{platform::collections::HashMap, prelude::*};
use bevy_yarnspinner::prelude::DialogueRunner;
use crate::{props::generic::Crt, third_party::bevy_trenchbroom::GetTrenchbroomModelPath as _};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CrtModel>()
        .init_resource::<CrtScreenTextures>()
        .add_systems(Update, (
            poll_crt_model_load.run_if(not(resource_exists::<CrtScreenMaterial>)),
            clear_intro_crt_emote,
        ));
}

/// Call this system from yarnspinner to change the emote
pub(crate) fn set_intro_crt_emote(
    emote_name: In<String>,
    handle: Res<CrtScreenMaterial>,
    textures: Res<CrtScreenTextures>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Some(material) = materials.get_mut(&**handle) {
        let texture = textures.get(emote_name.as_str());
        // Paste the texture or clear it
        material.base_color_texture = texture.cloned();
    } else {
        warn!("Could not get glass material from asset repository");
    }
}

/// Repository for the images that can be used as the intro CRT emote
#[derive(Resource, Deref)]
pub(crate) struct CrtScreenTextures(HashMap<String, Handle<Image>>);

impl FromWorld for CrtScreenTextures {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self(HashMap::from_iter([
            ("boot".into(), assets.load("models/office/crt/boot.png")),
            ("smile".into(), assets.load("models/office/crt/smile.png")),
            ("smile2".into(), assets.load("models/office/crt/smile2.png")),
            ("stonks".into(), assets.load("models/office/crt/stonks.png")),
            ("sideways".into(), assets.load("models/office/crt/sideways.png")),
            ("point".into(), assets.load("models/office/crt/point.png")),
            ("glitch".into(), assets.load("models/office/crt/glitch.png")),
            ("shocked".into(), assets.load("models/office/crt/shocked.png")),
            ("nod".into(), assets.load("models/office/crt/nod.png")),
            ("shake".into(), assets.load("models/office/crt/shake.png")),
            ("blank".into(), assets.load("models/office/crt/blank.png")),
            ("x".into(), assets.load("models/office/crt/x.png")),
            ("annoyed".into(), assets.load("models/office/crt/annoyed.png")),
            ("away".into(), assets.load("models/office/crt/away.png")),
            ("upside".into(), assets.load("models/office/crt/upside.png")),
            ("leftside".into(), assets.load("models/office/crt/leftside.png")),
            ("rightside".into(), assets.load("models/office/crt/rightside.png")),
        ]))
    }
}

#[derive(Resource, Deref)]
pub(crate) struct CrtScreenMaterial(Handle<StandardMaterial>);

#[derive(Resource, Deref)]
struct CrtModel(Handle<Gltf>);

impl FromWorld for CrtModel {
    fn from_world(world: &mut World) -> Self {
        Self(world.load_asset(Crt::model_path()))
    }
}

fn poll_crt_model_load(
    handle: Res<CrtModel>,
    models: Res<Assets<Gltf>>,
    mut commands: Commands,
) {
    if let Some(model) = models.get(&**handle) {
        if let Some(material) = model.named_materials.get("glass") {
            commands.insert_resource(CrtScreenMaterial(material.clone()));
            commands.remove_resource::<CrtModel>();
        } else {
            warn!("Could not get glass material from monitor model");
        }
    }
}

/// Clears the emote when dialog stops
/// so the face does not show up on every screen
/// in a 5 mile radius
fn clear_intro_crt_emote(
    dialog: Single<&DialogueRunner>,
    handle: Res<CrtScreenMaterial>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !dialog.is_running() && let Some(material) = materials.get_mut(&**handle) {
        material.base_color_texture = None;
    }
}
