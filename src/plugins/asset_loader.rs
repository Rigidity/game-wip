use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use indexmap::IndexMap;

use crate::GameState;

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LoadingAssets), load_assets)
            .add_systems(
                Update,
                check_assets.run_if(in_state(GameState::LoadingAssets)),
            );
    }
}

#[derive(Resource, Deref)]
pub struct BlockArray(Handle<Image>);

#[derive(Resource)]
pub struct Blocks(IndexMap<String, Handle<Image>>);

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut blocks = IndexMap::new();

    blocks.insert("dirt".into(), asset_server.load("blocks/dirt.png"));
    blocks.insert(
        "grass_side".into(),
        asset_server.load("blocks/grass_side.png"),
    );
    blocks.insert(
        "grass_top".into(),
        asset_server.load("blocks/grass_top.png"),
    );
    blocks.insert("rock".into(), asset_server.load("blocks/rock.png"));
    blocks.insert("sand".into(), asset_server.load("blocks/sand.png"));

    commands.insert_resource(Blocks(blocks));
}

fn check_assets(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut image_assets: ResMut<Assets<Image>>,
    blocks: Res<Blocks>,
    asset_server: Res<AssetServer>,
) {
    let Some(images) = blocks
        .0
        .values()
        .map(|handle| {
            if !asset_server.is_loaded_with_dependencies(handle) {
                None
            } else {
                image_assets.get(handle)
            }
        })
        .collect::<Option<Vec<&Image>>>()
    else {
        return;
    };

    let array_texture = Image::new(
        Extent3d {
            width: 16,
            height: 16,
            depth_or_array_layers: images.len() as u32,
        },
        TextureDimension::D2,
        images
            .into_iter()
            .flat_map(|image| image.data.clone())
            .collect(),
        TextureFormat::Rgba8UnormSrgb,
    );

    let handle = image_assets.add(array_texture);
    commands.insert_resource(BlockArray(handle));

    next_state.set(GameState::InGame);
}
