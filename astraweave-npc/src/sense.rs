use glam::Vec3;

#[derive(Clone, Debug)]
pub struct NpcWorldView {
    pub time_of_day: f32, // 0..24
    pub self_pos: Vec3,
    pub player_pos: Option<Vec3>,
    pub player_dist: Option<f32>,
    pub nearby_threat: bool,
    pub location_tag: Option<String>, // e.g., "market", "gate"
}

impl Default for NpcWorldView {
    fn default() -> Self {
        Self {
            time_of_day: 12.0,
            self_pos: Vec3::ZERO,
            player_pos: None,
            player_dist: None,
            nearby_threat: false,
            location_tag: None,
        }
    }
}

impl NpcWorldView {
    pub fn new(self_pos: Vec3, time_of_day: f32) -> Self {
        Self {
            time_of_day,
            self_pos,
            ..Default::default()
        }
    }

    pub fn with_player(mut self, player_pos: Vec3) -> Self {
        let dist = self.self_pos.distance(player_pos);
        self.player_pos = Some(player_pos);
        self.player_dist = Some(dist);
        self
    }

    pub fn with_threat(mut self, threat: bool) -> Self {
        self.nearby_threat = threat;
        self
    }

    pub fn with_location(mut self, tag: impl Into<String>) -> Self {
        self.location_tag = Some(tag.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_world_view() {
        let view = NpcWorldView::default();
        assert_eq!(view.time_of_day, 12.0);
        assert_eq!(view.self_pos, Vec3::ZERO);
        assert!(view.player_pos.is_none());
        assert!(view.player_dist.is_none());
        assert!(!view.nearby_threat);
        assert!(view.location_tag.is_none());
    }

    #[test]
    fn test_new_world_view() {
        let view = NpcWorldView::new(Vec3::new(10.0, 0.0, 20.0), 8.0);
        assert_eq!(view.time_of_day, 8.0);
        assert_eq!(view.self_pos, Vec3::new(10.0, 0.0, 20.0));
    }

    #[test]
    fn test_with_player() {
        let view = NpcWorldView::new(Vec3::new(0.0, 0.0, 0.0), 12.0)
            .with_player(Vec3::new(3.0, 0.0, 4.0));
        
        assert!(view.player_pos.is_some());
        assert_eq!(view.player_pos.unwrap(), Vec3::new(3.0, 0.0, 4.0));
        assert!(view.player_dist.is_some());
        assert!((view.player_dist.unwrap() - 5.0).abs() < 0.001); // 3-4-5 triangle
    }

    #[test]
    fn test_with_threat() {
        let view = NpcWorldView::default().with_threat(true);
        assert!(view.nearby_threat);
        
        let view = NpcWorldView::default().with_threat(false);
        assert!(!view.nearby_threat);
    }

    #[test]
    fn test_with_location() {
        let view = NpcWorldView::default().with_location("market");
        assert_eq!(view.location_tag, Some("market".to_string()));
    }

    #[test]
    fn test_builder_chain() {
        let view = NpcWorldView::new(Vec3::new(5.0, 0.0, 5.0), 18.0)
            .with_player(Vec3::new(10.0, 0.0, 5.0))
            .with_threat(true)
            .with_location("gate");
        
        assert_eq!(view.time_of_day, 18.0);
        assert_eq!(view.self_pos, Vec3::new(5.0, 0.0, 5.0));
        assert!(view.player_pos.is_some());
        assert!(view.player_dist.is_some());
        assert!((view.player_dist.unwrap() - 5.0).abs() < 0.001);
        assert!(view.nearby_threat);
        assert_eq!(view.location_tag, Some("gate".to_string()));
    }

    #[test]
    fn test_clone() {
        let view = NpcWorldView::new(Vec3::ONE, 6.0)
            .with_player(Vec3::new(2.0, 2.0, 2.0))
            .with_threat(true)
            .with_location("tavern");
        
        let cloned = view.clone();
        assert_eq!(cloned.time_of_day, view.time_of_day);
        assert_eq!(cloned.self_pos, view.self_pos);
        assert_eq!(cloned.player_pos, view.player_pos);
        assert_eq!(cloned.nearby_threat, view.nearby_threat);
        assert_eq!(cloned.location_tag, view.location_tag);
    }
}

