use glam::Vec3;
// use rapier3d::prelude::*; // Commented out due to API compatibility issues
use crate::{DamageType, Stats};
use astraweave_physics::PhysicsWorld;

#[derive(Clone, Copy, Debug)]
pub struct IFrame {
    pub time_left: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Parry {
    pub window: f32,
    pub active: bool,
}

pub struct Combatant {
    pub body: u64, // PhysicsWorld BodyId
    pub stats: Stats,
    pub iframes: Option<IFrame>,
    pub parry: Option<Parry>,
}

pub struct HitResult {
    pub target: u64,
    pub damage: i32,
    pub parried: bool,
}

/// Sweep a capsule from `from` to `to`, apply damage to first hit collider body that isn't `self_id`.
/// TODO: Fix rapier3d API compatibility issues
pub fn perform_attack_sweep(
    _phys: &mut PhysicsWorld,
    _self_id: u64,
    _from: Vec3,
    _to: Vec3,
    _radius: f32,
    _base_damage: i32,
    _dtype: DamageType,
    _targets: &mut [Combatant],
) -> Option<HitResult> {
    // Temporarily disabled due to rapier3d API changes
    // TODO: Update to use new rapier3d cast_shape API
    unimplemented!("perform_attack_sweep is not yet implemented due to rapier3d API changes")
    /*
    let shape = SharedShape::capsule_y(radius * 0.5, radius);
    let dir = to - from;
    let len = dir.length();
    if len <= 1e-3 { return None; }
    let ray = Ray::new(point![from.x, from.y, from.z], vector![(dir/len).x, (dir/len).y, (dir/len).z]);
    let max_toi = len;

    let filter = QueryFilter::default();
    if let Some((hcoll, toi)) = phys.query_pipeline.cast_shape(
        &phys.bodies, &phys.colliders,
        &Isometry::translation(from.x, from.y, from.z),
        &shape, vector![dir.x, dir.y, dir.z], max_toi, filter, None
    ) {
        let hbody = phys.colliders.get(hcoll).and_then(|c| c.parent()).unwrap();
        if let Some(tgt_id) = phys.id_of(hbody) {
            if tgt_id == self_id { return None; }

            // apply to matching target
            if let Some(tgt) = targets.iter_mut().find(|t| t.body == tgt_id) {
                // parry check
                if let Some(p) = &mut tgt.parry {
                    if p.active && p.window > 0.0 {
                        p.window = 0.0;
                        return Some(HitResult { target: tgt_id, damage: 0, parried: true });
                    }
                }
                // iframe check
                if let Some(i) = &tgt.iframes { if i.time_left > 0.0 { return Some(HitResult { target: tgt_id, damage: 0, parried: false }); } }

                let dmg = tgt.stats.apply_damage(base_damage, dtype);
                return Some(HitResult { target: tgt_id, damage: dmg, parried: false });
            }
        }
    }
    None
    */
}
