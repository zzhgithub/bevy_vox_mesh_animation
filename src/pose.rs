// 造型 数据

use bevy::prelude::{Children, Component, Entity, Quat, Query, Vec3};

pub struct Pose {
    right_hand: (Quat, Vec3),
    left_hand: (Quat, Vec3),
    right_arm: Quat,
    left_arm: Quat,
    right_leg: Quat,
    left_leg: Quat,
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
