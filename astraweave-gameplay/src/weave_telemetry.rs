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
