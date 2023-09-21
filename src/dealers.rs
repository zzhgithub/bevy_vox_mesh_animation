use bevy::prelude::{Handle, Mesh, PbrBundle, Transform};

use crate::DealWithJoints;

#[derive(Debug, Clone)]
pub struct CommonDealers;

impl DealWithJoints for CommonDealers {
    fn deal(
        &self,
        handle: Handle<Mesh>,
        commands: &mut bevy::prelude::Commands,
        mesh_assets: &mut bevy::prelude::Assets<bevy::prelude::Mesh>,
        material_handle: bevy::prelude::Handle<bevy::prelude::StandardMaterial>,
        _skinned_mesh_inverse_bindposes_assets: &mut bevy::prelude::Assets<
            bevy::render::mesh::skinning::SkinnedMeshInverseBindposes,
        >,
    ) -> Option<bevy::prelude::Entity> {
        if let Some(mesh) = mesh_assets.get(&handle) {
            let entity = commands
                .spawn(PbrBundle {
                    transform: Transform::IDENTITY,
                    mesh: mesh_assets.add(mesh.clone()),
                    material: material_handle.clone(),
                    ..Default::default()
                })
                .id();
            return Some(entity);
        }
        None
    }
}
