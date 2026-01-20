use serde::Serialize;

#[derive(Default, Clone, Debug, Serialize)]
pub struct WeaveTelemetry {
    pub ops_applied: usize,
    pub terrain_cost: i32,
    pub weather_cost: i32,
    pub est_time_saved_sec: f32,
    pub risk_score: f32,   // e.g. faction hostility / spawn risk
    pub reward_score: f32, // e.g. resource multiplier delta
}

impl WeaveTelemetry {
    pub fn add_terrain(&mut self, cost: i32) {
        self.ops_applied += 1;
        self.terrain_cost += cost;
    }
    pub fn add_weather(&mut self, cost: i32) {
        self.ops_applied += 1;
        self.weather_cost += cost;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weave_telemetry_default() {
        let telemetry = WeaveTelemetry::default();

        assert_eq!(telemetry.ops_applied, 0);
        assert_eq!(telemetry.terrain_cost, 0);
        assert_eq!(telemetry.weather_cost, 0);
        assert_eq!(telemetry.est_time_saved_sec, 0.0);
        assert_eq!(telemetry.risk_score, 0.0);
        assert_eq!(telemetry.reward_score, 0.0);
    }

    #[test]
    fn test_add_terrain() {
        let mut telemetry = WeaveTelemetry::default();

        telemetry.add_terrain(10);

        assert_eq!(telemetry.ops_applied, 1, "Should increment ops count");
        assert_eq!(telemetry.terrain_cost, 10, "Should add terrain cost");
        assert_eq!(telemetry.weather_cost, 0, "Weather cost should remain 0");
    }

    #[test]
    fn test_add_weather() {
        let mut telemetry = WeaveTelemetry::default();

        telemetry.add_weather(5);

        assert_eq!(telemetry.ops_applied, 1, "Should increment ops count");
        assert_eq!(telemetry.weather_cost, 5, "Should add weather cost");
        assert_eq!(telemetry.terrain_cost, 0, "Terrain cost should remain 0");
    }

    #[test]
    fn test_multiple_operations() {
        let mut telemetry = WeaveTelemetry::default();

        telemetry.add_terrain(10);
        telemetry.add_terrain(15);
        telemetry.add_weather(5);
        telemetry.add_weather(8);

        assert_eq!(telemetry.ops_applied, 4, "Should count all operations");
        assert_eq!(
            telemetry.terrain_cost, 25,
            "Should sum terrain costs (10+15)"
        );
        assert_eq!(telemetry.weather_cost, 13, "Should sum weather costs (5+8)");
    }

    #[test]
    fn test_manual_field_assignment() {
        let telemetry = WeaveTelemetry {
            est_time_saved_sec: 120.5,
            risk_score: 0.75,
            reward_score: 1.5,
            ..Default::default()
        };

        assert_eq!(telemetry.est_time_saved_sec, 120.5);
        assert_eq!(telemetry.risk_score, 0.75);
        assert_eq!(telemetry.reward_score, 1.5);
    }

    #[test]
    fn test_clone() {
        let mut original = WeaveTelemetry::default();
        original.add_terrain(10);
        original.add_weather(5);

        let cloned = original.clone();

        assert_eq!(cloned.ops_applied, original.ops_applied);
        assert_eq!(cloned.terrain_cost, original.terrain_cost);
        assert_eq!(cloned.weather_cost, original.weather_cost);
    }
}
