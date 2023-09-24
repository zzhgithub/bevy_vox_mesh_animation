use std::slice::Iter;

use bevy::{
    prelude::{BuildChildren, Handle, Mat4, Mesh, PbrBundle, Transform, Vec3, Vec4},
    render::{
        mesh::{
            skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
            VertexAttributeValues,
        },
        render_resource::PrimitiveTopology,
    },
    transform::TransformBundle,
};
use bevy_mod_mesh_tools::{mesh_len, mesh_normals, mesh_positions, mesh_uvs};

use crate::{
    types::{AnimatedJoint, Body, LeftArm, LeftHand, LeftLeg, RightArm, RightHand, RightLeg},
    DealWithJoints,
};

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
                    // 为了测试临时隐藏
                    // visibility: bevy::prelude::Visibility::Hidden,
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

#[derive(Debug, Clone)]
pub struct Body1Dealers;

impl DealWithJoints for Body1Dealers {
    fn deal(
        &self,
        handle: Handle<Mesh>,
        commands: &mut bevy::prelude::Commands,
        mesh_assets: &mut bevy::prelude::Assets<Mesh>,
        material_handle: Handle<bevy::prelude::StandardMaterial>,
        skinned_mesh_inverse_bindposes_assets: &mut bevy::prelude::Assets<
            bevy::render::mesh::skinning::SkinnedMeshInverseBindposes,
        >,
    ) -> Option<bevy::prelude::Entity> {
        // 获取mesh 然后添加
        if let Some(mesh) = mesh_assets.get(&handle) {
            let mut joints_mesh = Mesh::new(PrimitiveTopology::TriangleList);

            let positions_iter = mesh_positions(mesh);
            // 这要重重新转成数据
            let mesh_normals_iter = mesh_normals(mesh);
            let mesh_uvs_iter = mesh_uvs(mesh);
            let mesh_color_iter = mesh_color(mesh);
            // TODO 生成 joint index 并且配置权重
            joints_mesh.insert_attribute(
                Mesh::ATTRIBUTE_JOINT_INDEX,
                VertexAttributeValues::Uint16x4(vec![[0, 1, 2, 3]; mesh_len(mesh)]),
            );
            // 配置权限
            // 这使用 x = 0 和 y = 0 可以分出四个象限 不同象限的 对应的不一样
            //          |
            //     L.H  |  R.H
            //    ------0--------
            //     L.L  |  R.L
            //          |
            // left hand and right hand
            // left leg  and right leg
            let joint_weight: Vec<[f32; 4]> = positions_iter
                .clone()
                .map(|v3| {
                    let Vec3 { x, y, z: _ } = v3;
                    match (*x > 0.0, *y > -20.0) {
                        (true, true) => [1.0, 0.0, 0.0, 0.0],
                        (true, false) => [0.0, 0.0, 0.0, 1.0],
                        (false, true) => [0.0, 1.0, 0.0, 0.0],
                        (false, false) => [0.0, 0.0, 0.1, 0.0],
                    }
                })
                .collect();
            joints_mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, joint_weight);
            let position: Vec<[f32; 3]> = positions_iter.map(|x| [x.x, x.y, x.z]).collect();
            joints_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, position);
            let normals: Vec<[f32; 3]> = mesh_normals_iter.map(|p| [p.x, p.y, p.z]).collect();
            joints_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            let uvs: Vec<[f32; 2]> = mesh_uvs_iter.map(|p| [p.x, p.y]).collect();
            joints_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            let color: Vec<[f32; 4]> = mesh_color_iter.map(|p| [p.x, p.y, p.z, p.w]).collect();
            joints_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, color);
            let indices = mesh.indices().unwrap();
            joints_mesh.set_indices(Some(indices.clone()));

            let r_a = Vec3::new(-8.0, 6.0, -1.5);
            let l_a = Vec3::new(8.0, 6.0, -1.5);
            let l_l = Vec3::new(3.5, 24.5, -1.);
            let r_l = Vec3::new(-3.5, 24.5, -1.);

            // 这里绑定四个joint!
            let inverse_bindposes =
                skinned_mesh_inverse_bindposes_assets.add(SkinnedMeshInverseBindposes::from(vec![
                    // TODO 要修改这里的值
                    Mat4::from_translation(r_a.clone()),
                    Mat4::from_translation(l_a.clone()),
                    Mat4::from_translation(l_l.clone()),
                    Mat4::from_translation(r_l.clone()),
                ]));

            let entitiy1 = commands
                .spawn((
                    RightArm,
                    AnimatedJoint,
                    TransformBundle {
                        local: Transform::from_translation(Vec3::new(8.0, -6.0, 1.5)),
                        ..Default::default()
                    },
                ))
                .id();
            let entitiy2 = commands
                .spawn((
                    LeftArm,
                    AnimatedJoint,
                    TransformBundle {
                        local: Transform::from_translation(Vec3::new(-8.0, -6.0, 1.5)),
                        ..Default::default()
                    },
                ))
                .id();
            let entitiy3 = commands
                .spawn((
                    LeftLeg,
                    AnimatedJoint,
                    TransformBundle {
                        local: Transform::from_translation(Vec3::new(-3.5, -24.5, 1.)),
                        ..Default::default()
                    },
                ))
                .id();
            let entitiy4 = commands
                .spawn((
                    RightLeg,
                    AnimatedJoint,
                    TransformBundle {
                        local: Transform::from_translation(Vec3::new(3.5, -24.5, 1.)),
                        ..Default::default()
                    },
                ))
                .id();
            let joint_entities = vec![entitiy1, entitiy2, entitiy3, entitiy4];
            // 这里只能使用 Joint控制物体的大小 和 位置 那么这里怎么控制他们的位置和 大小呢？
            let ret = commands
                .spawn(PbrBundle {
                    mesh: mesh_assets.add(joints_mesh),
                    material: material_handle,
                    // 测试临时隐藏
                    // visibility: bevy::prelude::Visibility::Hidden,
                    ..Default::default()
                })
                .push_children(&joint_entities)
                .insert(SkinnedMesh {
                    inverse_bindposes: inverse_bindposes.clone(),
                    joints: joint_entities,
                })
                .id();
            return Some(ret);
        }
        None
    }
}

fn mesh_color(mesh: &Mesh) -> Iter<Vec4> {
    match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
        Some(VertexAttributeValues::Float32x4(v)) => unsafe {
            std::mem::transmute::<Iter<[f32; 4]>, Iter<Vec4>>(v.iter())
        },
        _ => [].iter(),
    }
}

#[derive(Debug, Clone)]
pub struct Body0Dealers;

impl DealWithJoints for Body0Dealers {
    fn deal(
        &self,
        handle: Handle<Mesh>,
        commands: &mut bevy::prelude::Commands,
        mesh_assets: &mut bevy::prelude::Assets<Mesh>,
        material_handle: Handle<bevy::prelude::StandardMaterial>,
        skinned_mesh_inverse_bindposes_assets: &mut bevy::prelude::Assets<
            SkinnedMeshInverseBindposes,
        >,
    ) -> Option<bevy::prelude::Entity> {
        // 这里记录 两个手 还有 一个身体的标记
        if let Some(mesh) = mesh_assets.get(&handle) {
            let mut joints_mesh = Mesh::new(PrimitiveTopology::TriangleList);

            let positions_iter = mesh_positions(mesh);
            // 这要重重新转成数据
            let mesh_normals_iter = mesh_normals(mesh);
            let mesh_uvs_iter = mesh_uvs(mesh);
            let mesh_color_iter = mesh_color(mesh);
            // 生成 joint index 并且配置权重
            joints_mesh.insert_attribute(
                Mesh::ATTRIBUTE_JOINT_INDEX,
                VertexAttributeValues::Uint16x4(vec![[0, 1, 2, 0]; mesh_len(mesh)]),
            );
            // 配置权限
            let joint_weight: Vec<[f32; 4]> = positions_iter
                .clone()
                .map(|v3| {
                    let Vec3 { x, y: _, z: _ } = v3;
                    return if *x < -15.0 {
                        [0.0, 1.0, 0.0, 0.0]
                    } else if *x < 15.0 {
                        [0.0, 0.0, 1.0, 0.0]
                    } else {
                        [1.0, 0.0, 0.0, 0.0]
                    };
                })
                .collect();
            joints_mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, joint_weight);
            let position: Vec<[f32; 3]> = positions_iter.map(|x| [x.x, x.y, x.z]).collect();
            joints_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, position);
            let normals: Vec<[f32; 3]> = mesh_normals_iter.map(|p| [p.x, p.y, p.z]).collect();
            joints_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            let uvs: Vec<[f32; 2]> = mesh_uvs_iter.map(|p| [p.x, p.y]).collect();
            joints_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            let color: Vec<[f32; 4]> = mesh_color_iter.map(|p| [p.x, p.y, p.z, p.w]).collect();
            joints_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, color);
            let indices = mesh.indices().unwrap();
            joints_mesh.set_indices(Some(indices.clone()));

            let r_h = Vec3::new(-24.0, 7.0, -2.5);
            let l_h = Vec3::new(24.0, 7.0, -2.5);
            let b = Vec3::new(0.0, 0.0, -0.0);

            // let r_h = Vec3::ZERO;
            // let l_h = Vec3::ZERO;
            // let b = Vec3::ZERO;

            let r_h_local = Vec3::new(24.0, -7.0, 2.5);
            let l_h_local = Vec3::new(-24.0, -7.0, 2.5);
            let b_local = Vec3::ZERO;

            // 这里绑定四个joint!
            let inverse_bindposes =
                skinned_mesh_inverse_bindposes_assets.add(SkinnedMeshInverseBindposes::from(vec![
                    Mat4::from_translation(r_h.clone()),
                    Mat4::from_translation(l_h.clone()),
                    Mat4::from_translation(b.clone()),
                ]));

            let entitiy1 = commands
                .spawn((
                    RightHand,
                    AnimatedJoint,
                    TransformBundle {
                        local: Transform::from_translation(r_h_local),
                        ..Default::default()
                    },
                ))
                .id();
            let entitiy2 = commands
                .spawn((
                    LeftHand,
                    AnimatedJoint,
                    TransformBundle {
                        local: Transform::from_translation(l_h_local),
                        ..Default::default()
                    },
                ))
                .id();
            let entitiy3 = commands
                .spawn((
                    Body,
                    AnimatedJoint,
                    TransformBundle {
                        local: Transform::from_translation(b_local),
                        ..Default::default()
                    },
                ))
                .id();
            let joint_entities = vec![entitiy1, entitiy2, entitiy3];
            let ret = commands
                .spawn(PbrBundle {
                    mesh: mesh_assets.add(joints_mesh),
                    material: material_handle,
                    ..Default::default()
                })
                .push_children(&joint_entities)
                .insert(SkinnedMesh {
                    inverse_bindposes: inverse_bindposes.clone(),
                    joints: joint_entities,
                })
                .id();
            return Some(ret);
        }
        None
    }
}
