use glam::Vec3;

#[derive(Clone, Debug)]
pub struct NpcWorldView {
    pub time_of_day: f32,         // 0..24
    pub self_pos: Vec3,
    pub player_pos: Option<Vec3>,
    pub player_dist: Option<f32>,
    pub nearby_threat: bool,
    pub location_tag: Option<String>, // e.g., "market", "gate"
}
