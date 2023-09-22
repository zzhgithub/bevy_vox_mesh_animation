use bevy::{
    prelude::{
        AssetServer, Assets, BuildChildren, Commands, ComputedVisibility, Entity, GlobalTransform,
        Handle, Mesh, Name, StandardMaterial, Transform, Visibility,
    },
    render::mesh::skinning::SkinnedMeshInverseBindposes,
    utils::HashMap,
};
use bevy_vox_mesh::mate_data::{LayerData, VoxMateData};
use dot_vox::SceneNode;

// 制作和使用 vox 作为动画的工具
pub mod dealers;
pub mod mesh_helper;
pub mod types;
pub trait DealWithJoints: Send + Sync + 'static {
    fn deal(
        &self,
        handle: Handle<Mesh>,
        commands: &mut Commands,
        mesh_assets: &mut Assets<Mesh>,
        material_handle: Handle<StandardMaterial>,
        skinned_mesh_inverse_bindposes_assets: &mut Assets<SkinnedMeshInverseBindposes>,
    ) -> Option<Entity>;
}

pub struct DealerHolder {}

pub fn perpare_player_data(
    base_id: &'static str,
    vox_mate_data: VoxMateData,
    commands: &mut Commands,
    asset_server: &AssetServer,
    material_handle: Handle<StandardMaterial>,
    mesh_assets: &mut Assets<Mesh>,
    skinned_mesh_inverse_bindposes_assets: &mut Assets<SkinnedMeshInverseBindposes>,
    perpare_map: HashMap<String, Box<dyn DealWithJoints>>,
) -> HashMap<String, Entity> {
    let mut result: HashMap<String, Entity> = HashMap::new();
    for scene_node in vox_mate_data.scenes.iter() {
        match scene_node {
            SceneNode::Transform {
                attributes,
                frames: _,
                child: _,
                layer_id: _,
            } => {
                if let Some(name) = attributes.get("_name") {
                    if let Some(dealer) = perpare_map.get(name) {
                        let ret = deal_scene_node(
                            base_id,
                            commands,
                            asset_server,
                            scene_node,
                            &vox_mate_data.scenes,
                            material_handle.clone(),
                            mesh_assets,
                            &vox_mate_data.layer_map,
                            skinned_mesh_inverse_bindposes_assets,
                            dealer,
                        );
                        result.insert(name.clone(), ret[0]);
                    }
                }
            }
            _ => {}
        }
    }
    result
}

fn deal_scene_node(
    base_id: &'static str,
    commands: &mut Commands,
    asset_server: &AssetServer,
    scene_node: &SceneNode,
    scenes_tree: &Vec<SceneNode>,
    material_handle: Handle<StandardMaterial>,
    mesh_assets: &mut Assets<Mesh>,
    layer_map: &HashMap<u32, bool>,
    skinned_mesh_inverse_bindposes_assets: &mut Assets<SkinnedMeshInverseBindposes>,
    deal_with_joints: &Box<dyn DealWithJoints>,
) -> Vec<Entity> {
    let mut result: Vec<Entity> = Vec::new();
    match scene_node {
        SceneNode::Transform {
            attributes,
            frames,
            child,
            layer_id,
        } => {
            // 标记一下当前数据？
            let mut node = commands.spawn(LayerData(layer_id.clone()));
            if let Some(name) = attributes.get("_name") {
                node.insert(Name::new(name.to_owned()));
            }
            for frame in frames.iter() {
                // TODO: Support Other Types
                if let Some(pos) = frame.position() {
                    node.insert(Transform::from_xyz(
                        pos.x as f32,
                        pos.y as f32,
                        pos.z as f32,
                    ));
                }
            }

            let children = deal_scene_node(
                base_id,
                node.commands(),
                asset_server,
                &scenes_tree[child.clone() as usize],
                scenes_tree,
                material_handle.clone(),
                mesh_assets,
                layer_map,
                skinned_mesh_inverse_bindposes_assets,
                deal_with_joints,
            );
            node.push_children(&children);

            let visibilty = if let Some(hidden) = layer_map.get(layer_id) {
                if *hidden {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                }
            } else {
                Visibility::Inherited
            };
            node.insert((
                visibilty,
                ComputedVisibility::HIDDEN,
                Transform::IDENTITY,
                GlobalTransform::IDENTITY,
            ));
            result.push(node.id());
        }
        SceneNode::Group {
            attributes: _,
            children,
        } => {
            // 获取一组数据
            for ch_key in children {
                let children = deal_scene_node(
                    base_id,
                    commands,
                    asset_server,
                    &scenes_tree[ch_key.clone() as usize],
                    scenes_tree,
                    material_handle.clone(),
                    mesh_assets,
                    layer_map,
                    skinned_mesh_inverse_bindposes_assets,
                    deal_with_joints,
                );
                result.extend(children);
            }
        }
        SceneNode::Shape {
            attributes: _,
            models,
        } => {
            // 这里生成单个的entity
            for shape in models {
                let key = if shape.model_id == 0 {
                    format!("{}", base_id,)
                } else {
                    format!("{}#model{}", base_id, shape.model_id)
                };
                let handle: Handle<Mesh> = asset_server.get_handle(key.clone());
                println!("{}-{:?}", key, asset_server.get_load_state(handle.clone()));

                if let Some(entity) = deal_with_joints.deal(
                    handle.clone(),
                    commands,
                    mesh_assets,
                    material_handle.clone(),
                    skinned_mesh_inverse_bindposes_assets,
                ) {
                    result.push(entity);
                }
            }
        }
    }
    result
}
