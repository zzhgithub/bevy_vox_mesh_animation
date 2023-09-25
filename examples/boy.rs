use bevy::{
    prelude::*, render::mesh::skinning::SkinnedMeshInverseBindposes, utils::HashMap,
    window::PresentMode,
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::{prelude::PickingInteraction, DefaultPickingPlugins};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_transform_gizmo::{GizmoTransformable, TransformGizmo, TransformGizmoPlugin};
use bevy_vox_mesh::{vox_scene_info::VoxSceneInfo, VoxMeshPlugin};
use bevy_vox_mesh_animation::{
    dealers::{Body0Dealers, Body1Dealers, CommonDealers},
    perpare_player_data,
    pose::{BoyEntity, PoseEditPlugin},
    types::{AnimatedJoint, LeftArm, LeftLeg, RightArm, RightHand, RightLeg},
    DealWithJoints,
};
use std::f32::consts::PI;

fn main() {
    App::default()
        .add_plugins((
            DefaultPlugins,
            DefaultPickingPlugins,
            TransformGizmoPlugin::new(
                Quat::from_rotation_y(-0.2), // Align the gizmo to a different coordinate system.
                                             // Use TransformGizmoPlugin::default() to align to the
                                             // scene's coordinate system.
            ),
        ))
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(EguiPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(VoxMeshPlugin::default())
        .add_plugins(PoseEditPlugin)
        .insert_resource(BoyMate {
            handle: None,
            mate: None,
        })
        .insert_resource(BoyEntity { boy_entity: None })
        .insert_resource(FaceNow::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                load_mate,
                load_boy,
                toggle_faces,
                show_pick,
                toggle_visible_animated_joint,
                toggle_camera_controls_system,
                auto_toggle_camera_controls_system,
            ),
        )
        // .add_systems(Update, load_ik)
        .run();
}

#[derive(Debug, Resource, Clone)]
pub struct BoyMate {
    pub handle: Option<Handle<VoxSceneInfo>>,
    pub mate: Option<VoxSceneInfo>,
}

#[derive(Debug, Resource, Clone)]
pub struct FaceNow {
    pub now_face: &'static str,
}

impl Default for FaceNow {
    fn default() -> Self {
        Self { now_face: "face0" }
    }
}

#[derive(Debug, Component)]
pub struct ReadyEntity;

// 自动切换镜头的转动
fn auto_toggle_camera_controls_system(
    mut pan_orbit_query: Query<&mut PanOrbitCamera>,
    mut gizmo_query: Query<(
        &mut TransformGizmo,
        &mut PickingInteraction,
        &GlobalTransform,
    )>,
) {
    for (_gizmo, interaction, _transform) in gizmo_query.iter_mut() {
        if *interaction == PickingInteraction::Pressed {
            // 如果存在任何一个 是按下去的情况下
            for mut pan_orbit in pan_orbit_query.iter_mut() {
                pan_orbit.enabled = false;
            }
            return;
        }
    }
    for mut pan_orbit in pan_orbit_query.iter_mut() {
        pan_orbit.enabled = true;
    }
}

fn toggle_camera_controls_system(
    key_input: Res<Input<KeyCode>>,
    mut pan_orbit_query: Query<&mut PanOrbitCamera>,
) {
    if key_input.just_pressed(KeyCode::T) {
        for mut pan_orbit in pan_orbit_query.iter_mut() {
            pan_orbit.enabled = !pan_orbit.enabled;
        }
    }
}

fn toggle_faces(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(Entity, &Name, &mut Visibility)>,
    mut face_now: ResMut<FaceNow>,
) {
    let faces = vec!["face0", "face1", "face2", "face3"];
    if keyboard_input.just_pressed(KeyCode::Tab) {
        if let Some(index) = faces.iter().position(|&x| x == face_now.now_face) {
            let next_index = if index == faces.len() - 1 {
                0
            } else {
                index + 1
            };
            let next_face = faces[next_index];
            for (_, name, mut visibility) in query.iter_mut() {
                if faces.contains(&name.as_str()) {
                    if name.as_str() == next_face {
                        *visibility.as_mut() = Visibility::Inherited;
                    } else {
                        *visibility.as_mut() = Visibility::Hidden;
                    }
                }
            }
            face_now.now_face = next_face;
        }
    }
}

// 显示配置的关节
fn show_joints(
    time: Res<Time>,
    mut gizmos: Gizmos,
    mut query: Query<(&mut Transform, &RightArm)>,
    boy_entity: Res<BoyEntity>,
    transform_query: Query<&Transform, Without<RightArm>>,
) {
    if let Some(entity) = boy_entity.boy_entity {
        if let Ok(trf) = transform_query.get(entity) {
            // 这里可以进行其他的处理?
            {
                for (mut transform, _) in &mut query {
                    transform.rotation =
                        Quat::from_axis_angle(Vec3::Z, 0.5 * PI * time.elapsed_seconds().sin());
                    let new_trf = trf.clone().mul_transform(transform.clone());
                    gizmos.sphere(new_trf.translation, new_trf.rotation, 0.2, Color::RED);
                }
            }
        }
    }
}

// 获取可以配置的 控制器
fn show_pick(
    mut commands: Commands,
    query: Query<(Entity, &AnimatedJoint), Without<GizmoTransformable>>,
) {
    for (entity, _) in query.iter() {
        commands.entity(entity).insert((
            bevy_mod_picking::PickableBundle::default(),
            bevy_mod_picking::backends::raycast::RaycastPickTarget::default(),
            bevy_transform_gizmo::GizmoTransformable,
        ));
    }
}

fn toggle_visible_animated_joint(
    mut query: Query<(&mut Visibility, &AnimatedJoint)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::V) {
        for (mut visibility, _) in query.iter_mut() {
            match *visibility {
                Visibility::Inherited => *visibility = Visibility::Hidden,
                Visibility::Hidden => *visibility = Visibility::Inherited,
                Visibility::Visible => *visibility = Visibility::Hidden,
            }
        }
    }
}

fn load_boy(
    mut commands: Commands,
    boy_mate: Res<BoyMate>,
    mut boy_entity: ResMut<BoyEntity>,
    assets: Res<AssetServer>,
    mut stdmats: ResMut<Assets<StandardMaterial>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut skinned_mesh_inverse_bindposes_assets: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    if let Some(_entity) = boy_entity.boy_entity {
        // 这里可以进行其他的处理?
    } else {
        if let Some(mate_data) = boy_mate.mate.clone() {
            if mate_data.all_loaded("boy.vox", mesh_assets.as_ref(), assets.as_ref()) {
                let mut config_map: HashMap<String, Box<dyn DealWithJoints>> = HashMap::new();
                let dealer = CommonDealers {};
                let body1_dealer = Body1Dealers {};
                let body0_dealer = Body0Dealers {};
                config_map.insert(String::from("face0"), Box::new(dealer.clone()));
                config_map.insert(String::from("face1"), Box::new(dealer.clone()));
                config_map.insert(String::from("face2"), Box::new(dealer.clone()));
                config_map.insert(String::from("face3"), Box::new(dealer.clone()));
                config_map.insert(String::from("body0"), Box::new(body0_dealer.clone()));
                config_map.insert(String::from("body1"), Box::new(body1_dealer.clone()));

                let entitiys = perpare_player_data(
                    "boy.vox",
                    mate_data.clone(),
                    &mut commands,
                    assets.as_ref(),
                    stdmats.add(Color::rgb(1., 1., 1.).into()),
                    mesh_assets.as_mut(),
                    skinned_mesh_inverse_bindposes_assets.as_mut(),
                    config_map,
                    stdmats.as_mut(),
                );
                // TODO 这里使用其他方法准备数据!
                let boy = commands
                    .spawn((
                        ReadyEntity,
                        Visibility::Inherited,
                        ComputedVisibility::HIDDEN,
                        GlobalTransform::IDENTITY,
                        Transform {
                            translation: Vec3 {
                                x: 0.0,
                                y: 1.0 / 40. * 40., // height is 80 so the button is scale*80/2
                                z: 0.0,
                            },
                            scale: Vec3 {
                                x: 1.0 / 40.,
                                y: 1.0 / 40.,
                                z: 1.0 / 40.,
                            },
                            ..Default::default()
                        } * Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, PI)),
                    ))
                    .id();
                for s in vec![
                    &String::from("face0"),
                    &String::from("face1"),
                    &String::from("face2"),
                    &String::from("face3"),
                    &String::from("body0"),
                    &String::from("body1"),
                ] {
                    if let Some(entity) = entitiys.get(s) {
                        commands.entity(boy).add_child(*entity);
                    }
                }
                boy_entity.boy_entity = Some(boy);
            }
        }
    }
}

fn load_mate(mate_assets: Res<Assets<VoxSceneInfo>>, mut boy_mate: ResMut<BoyMate>) {
    if let Some(handle) = boy_mate.handle.clone() {
        match boy_mate.mate {
            Some(_) => {}
            None => {
                if let Some(mate) = mate_assets.get(&handle) {
                    boy_mate.mate = Some(mate.clone());
                }
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut stdmats: ResMut<Assets<StandardMaterial>>,
    mut boy_mate: ResMut<BoyMate>,
    assets: Res<AssetServer>,
) {
    let mate_data_handle: Handle<VoxSceneInfo> = assets.load("boy.vox#scene");
    boy_mate.handle = Some(mate_data_handle);

    // 添加环境光照
    commands.insert_resource(AmbientLight {
        brightness: 1.06,
        ..Default::default()
    });

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert((
            bevy_mod_picking::backends::raycast::RaycastPickCamera::default(),
            bevy_transform_gizmo::GizmoPickSource::default(),
            PanOrbitCamera::default(),
        ));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            subdivisions: 2,
            size: 5.0,
        })),
        material: stdmats.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
}
