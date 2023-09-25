// 造型 数据

use bevy::{
    prelude::{
        Children, Component, Entity, Plugin, Quat, Query, Res, ResMut, Resource, Transform, Update,
        Vec3,
    },
    utils::HashMap,
};
use bevy_egui::{EguiContexts, EguiPlugin};

use crate::types::{LeftArm, LeftHand, RightArm, RightHand, RightLeg, LeftLeg};

#[derive(Debug, Resource)]
pub struct BoyEntity {
    pub boy_entity: Option<Entity>,
}

#[derive(Debug, Clone, Default)]
pub struct Pose {
    right_hand: (Quat, Vec3),
    left_hand: (Quat, Vec3),
    right_arm: (Quat, Vec3),
    left_arm: (Quat, Vec3),
    right_leg: (Quat, Vec3),
    left_leg: (Quat, Vec3),
}

// 找到子类中的某个数据
pub fn find_entity<T>(
    root: Entity,
    children_query: &Query<&Children>,
    t_query: &Query<&T>,
) -> Result<Entity, ()>
where
    T: Component,
{
    if let Ok(children) = children_query.get(root) {
        for child in children.iter() {
            if let Ok(_) = t_query.get(*child) {
                return Ok(*child);
            }
            if let Ok(entity) = find_entity::<T>(*child, children_query, t_query) {
                return Ok(entity);
            }
        }
    }

    Err(())
}

// 加载某个数据
// 获取当前的姿态数据

#[derive(Debug, Clone, Resource)]
pub struct PoseMap {
    pub map_data: HashMap<String, Pose>,
}

pub struct PoseEditPlugin;

impl Plugin for PoseEditPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(PoseMap {
            map_data: HashMap::new(),
        });
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        app.add_systems(Update, pose_edit_ui);
    }
}

// 操作造型的ui
fn pose_edit_ui(
    boy_entity: Res<BoyEntity>,
    mut contexts: EguiContexts,
    mut pose_map: ResMut<PoseMap>,
    children_query: Query<&Children>,
    right_arm_query: Query<&RightArm>,
    left_arm_query: Query<&LeftArm>,
    right_leg_query: Query<&RightLeg>,
    left_leg_query: Query<&LeftLeg>,
    right_hand_query: Query<&RightHand>,
    left_hand_query: Query<&LeftHand>,
    mut transforms_query: Query<&mut Transform>,
) {
    let ctx = contexts.ctx_mut();
    if let Some(root) = boy_entity.boy_entity {
        bevy_egui::egui::Window::new("Pose Edit")
            .resizable(true)
            .default_height(20.0)
            .default_width(5.0)
            .show(ctx, |ui| {
                // 这里的 循环的展示一下数据 姿势列表
                bevy_egui::egui::ScrollArea::both().show(ui, |ui| {
                    for (name, pose) in &pose_map.map_data {
                        if ui.button(name).clicked() {
                            load_pose_entity(
                                root,
                                pose.clone(),
                                &children_query,
                                &right_arm_query,
                                &left_arm_query,
                                &right_leg_query,
                                &left_leg_query,
                                &right_hand_query,
                                &left_hand_query,
                                &mut transforms_query,
                            );
                        }
                    }
                });
                // 这里有 记录当前的 pose 和 加载当前的pose
                if ui.button("Save Pose").clicked() {
                    let pose = get_pose_entity(
                        root,
                        &children_query,
                        &right_arm_query,
                        &left_arm_query,
                        &right_leg_query,
                        &left_leg_query,
                        &right_hand_query,
                        &left_hand_query,
                        &transforms_query,
                    );
                    // 这里暂时是默认值
                    let index = pose_map.map_data.len();
                    pose_map.map_data.insert(format!("pose{}", index), pose);
                }
            });
    }
}

// 加载数据
fn load_pose_entity(
    root: Entity,
    pose: Pose,
    children_query: &Query<&Children>,
    right_arm_query: &Query<&RightArm>,
    left_arm_query: &Query<&LeftArm>,
    right_leg_query: &Query<&RightLeg>,
    left_leg_query: &Query<&LeftLeg>,
    right_hand_query: &Query<&RightHand>,
    left_hand_query: &Query<&LeftHand>,
    transforms_query: &mut Query<&mut Transform>,
) {
    load_for_entity(
        root,
        children_query,
        right_arm_query,
        transforms_query,
        pose.right_arm,
    );
    load_for_entity(
        root,
        children_query,
        left_arm_query,
        transforms_query,
        pose.left_arm,
    );
    load_for_entity(
        root,
        children_query,
        right_leg_query,
        transforms_query,
        pose.right_leg,
    );
    load_for_entity(
        root,
        children_query,
        left_leg_query,
        transforms_query,
        pose.left_leg,
    );
    load_for_entity(
        root,
        children_query,
        right_hand_query,
        transforms_query,
        pose.right_hand,
    );
    load_for_entity(
        root,
        children_query,
        left_hand_query,
        transforms_query,
        pose.left_hand,
    );
}

fn load_for_entity<T>(
    root: Entity,
    children_query: &Query<&Children>,
    t_query: &Query<&T>,
    transforms_query: &mut Query<&mut Transform>,
    data: (Quat, Vec3),
) where
    T: Component,
{
    if let Ok(entity) = find_entity(root, children_query, t_query) {
        if let Ok(mut tfr) = transforms_query.get_mut(entity) {
            tfr.rotation = data.0;
            tfr.translation = data.1;
        }
    }
}

fn get_pose_entity(
    root: Entity,
    children_query: &Query<&Children>,
    right_arm_query: &Query<&RightArm>,
    left_arm_query: &Query<&LeftArm>,
    right_leg_query: &Query<&RightLeg>,
    left_leg_query: &Query<&LeftLeg>,
    right_hand_query: &Query<&RightHand>,
    left_hand_query: &Query<&LeftHand>,
    transforms_query: &Query<&mut Transform>,
) -> Pose {
    let mut pose = Pose::default();
    if let Some(data) = get_for_entity(root, children_query, right_arm_query, transforms_query) {
        pose.right_arm = data;
    }
    if let Some(data) = get_for_entity(root, children_query, left_arm_query, transforms_query) {
        pose.left_arm = data;
    }
    if let Some(data) = get_for_entity(root, children_query, right_leg_query, transforms_query) {
        pose.right_leg = data;
    }
    if let Some(data) = get_for_entity(root, children_query, left_leg_query, transforms_query) {
        pose.left_leg = data;
    }
    if let Some(data) = get_for_entity(root, children_query, right_hand_query, transforms_query) {
        pose.right_hand = data;
    }
    if let Some(data) = get_for_entity(root, children_query, left_hand_query, transforms_query) {
        pose.left_hand = data;
    }
    pose
}

// 获取单个值
fn get_for_entity<T>(
    root: Entity,
    children_query: &Query<&Children>,
    t_query: &Query<&T>,
    transforms_query: &Query<&mut Transform>,
) -> Option<(Quat, Vec3)>
where
    T: Component,
{
    if let Ok(entity) = find_entity(root, children_query, t_query) {
        if let Ok(tfr) = transforms_query.get(entity) {
            return Some((tfr.rotation, tfr.translation));
        }
    }
    None
}
