// 造型 数据

use bevy::prelude::{Children, Component, Entity, Quat, Query, Vec3};

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
