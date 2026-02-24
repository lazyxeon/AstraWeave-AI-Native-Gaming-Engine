//! Wave 2 Mutation Remediation — Environment Presets: per-variant exhaustive tests
//!
//! Targets: environment_preset_panel.rs (1,349 lines)
//! - TimeOfDay (12 variants × sun_elevation/sun_temperature/hour/is_daytime)
//! - WeatherCondition (14 variants × cloud_coverage/fog_multiplier/sun_attenuation/has_precipitation)
//! - SkyType (4 variants × supports_time_of_day)
//! - FogType (6 variants × performance_cost)
//! - Tonemapper (8 variants × is_color_preserving/is_filmic)
//! - MoodPreset (14 variants × recommended_tonemapper/contrast/saturation)
//! - EnvironmentSettings: from_mood, apply_time, apply_weather

use aw_editor_lib::panels::environment_preset_panel::{
    EnvironmentPresetPanel, EnvironmentSettings, FogType, MoodPreset, SkyType, TimeOfDay,
    Tonemapper, WeatherCondition,
};

// ============================================================================
// TIME OF DAY — SUN ELEVATION (12 variants)
// ============================================================================

#[test]
fn tod_dawn_sun_elevation() {
    assert!((TimeOfDay::Dawn.sun_elevation() - 5.0).abs() < 0.01);
}

#[test]
fn tod_early_morning_sun_elevation() {
    assert!((TimeOfDay::EarlyMorning.sun_elevation() - 15.0).abs() < 0.01);
}

#[test]
fn tod_morning_sun_elevation() {
    assert!((TimeOfDay::Morning.sun_elevation() - 35.0).abs() < 0.01);
}

#[test]
fn tod_noon_sun_elevation() {
    assert!((TimeOfDay::Noon.sun_elevation() - 75.0).abs() < 0.01);
}

#[test]
fn tod_afternoon_sun_elevation() {
    assert!((TimeOfDay::Afternoon.sun_elevation() - 55.0).abs() < 0.01);
}

#[test]
fn tod_golden_hour_sun_elevation() {
    assert!((TimeOfDay::GoldenHour.sun_elevation() - 15.0).abs() < 0.01);
}

#[test]
fn tod_sunset_sun_elevation() {
    assert!((TimeOfDay::Sunset.sun_elevation() - 3.0).abs() < 0.01);
}

#[test]
fn tod_dusk_sun_elevation() {
    assert!((TimeOfDay::Dusk.sun_elevation() - (-5.0)).abs() < 0.01);
}

#[test]
fn tod_blue_hour_sun_elevation() {
    assert!((TimeOfDay::BlueHour.sun_elevation() - (-10.0)).abs() < 0.01);
}

#[test]
fn tod_night_sun_elevation() {
    assert!((TimeOfDay::Night.sun_elevation() - (-30.0)).abs() < 0.01);
}

#[test]
fn tod_midnight_sun_elevation() {
    assert!((TimeOfDay::Midnight.sun_elevation() - (-45.0)).abs() < 0.01);
}

#[test]
fn tod_late_night_sun_elevation() {
    assert!((TimeOfDay::LateNight.sun_elevation() - (-35.0)).abs() < 0.01);
}

// ============================================================================
// TIME OF DAY — SUN TEMPERATURE (12 variants)
// ============================================================================

#[test]
fn tod_dawn_sun_temperature() {
    assert_eq!(TimeOfDay::Dawn.sun_temperature(), 2500);
}

#[test]
fn tod_early_morning_sun_temperature() {
    assert_eq!(TimeOfDay::EarlyMorning.sun_temperature(), 3500);
}

#[test]
fn tod_morning_sun_temperature() {
    assert_eq!(TimeOfDay::Morning.sun_temperature(), 5500);
}

#[test]
fn tod_noon_sun_temperature() {
    assert_eq!(TimeOfDay::Noon.sun_temperature(), 6500);
}

#[test]
fn tod_afternoon_sun_temperature() {
    assert_eq!(TimeOfDay::Afternoon.sun_temperature(), 5800);
}

#[test]
fn tod_golden_hour_sun_temperature() {
    assert_eq!(TimeOfDay::GoldenHour.sun_temperature(), 3000);
}

#[test]
fn tod_sunset_sun_temperature() {
    assert_eq!(TimeOfDay::Sunset.sun_temperature(), 2200);
}

#[test]
fn tod_dusk_sun_temperature() {
    assert_eq!(TimeOfDay::Dusk.sun_temperature(), 2800);
}

#[test]
fn tod_blue_hour_sun_temperature() {
    assert_eq!(TimeOfDay::BlueHour.sun_temperature(), 8000);
}

#[test]
fn tod_night_sun_temperature() {
    assert_eq!(TimeOfDay::Night.sun_temperature(), 10000);
}

#[test]
fn tod_midnight_sun_temperature() {
    assert_eq!(TimeOfDay::Midnight.sun_temperature(), 12000);
}

#[test]
fn tod_late_night_sun_temperature() {
    assert_eq!(TimeOfDay::LateNight.sun_temperature(), 10000);
}

// ============================================================================
// TIME OF DAY — HOUR (12 variants)
// ============================================================================

#[test]
fn tod_dawn_hour() {
    assert!((TimeOfDay::Dawn.hour() - 5.5).abs() < 0.01);
}

#[test]
fn tod_early_morning_hour() {
    assert!((TimeOfDay::EarlyMorning.hour() - 7.0).abs() < 0.01);
}

#[test]
fn tod_morning_hour() {
    assert!((TimeOfDay::Morning.hour() - 9.0).abs() < 0.01);
}

#[test]
fn tod_noon_hour() {
    assert!((TimeOfDay::Noon.hour() - 12.0).abs() < 0.01);
}

#[test]
fn tod_afternoon_hour() {
    assert!((TimeOfDay::Afternoon.hour() - 15.0).abs() < 0.01);
}

#[test]
fn tod_golden_hour_hour() {
    assert!((TimeOfDay::GoldenHour.hour() - 18.0).abs() < 0.01);
}

#[test]
fn tod_sunset_hour() {
    assert!((TimeOfDay::Sunset.hour() - 19.5).abs() < 0.01);
}

#[test]
fn tod_dusk_hour() {
    assert!((TimeOfDay::Dusk.hour() - 20.0).abs() < 0.01);
}

#[test]
fn tod_blue_hour_hour() {
    assert!((TimeOfDay::BlueHour.hour() - 20.5).abs() < 0.01);
}

#[test]
fn tod_night_hour() {
    assert!((TimeOfDay::Night.hour() - 22.0).abs() < 0.01);
}

#[test]
fn tod_midnight_hour() {
    assert!((TimeOfDay::Midnight.hour() - 0.0).abs() < 0.01);
}

#[test]
fn tod_late_night_hour() {
    assert!((TimeOfDay::LateNight.hour() - 3.0).abs() < 0.01);
}

// ============================================================================
// TIME OF DAY — IS_DAYTIME (exactly: elevation > 0.0)
// ============================================================================

#[test]
fn tod_dawn_is_daytime() {
    assert!(TimeOfDay::Dawn.is_daytime()); // 5.0 > 0
}

#[test]
fn tod_early_morning_is_daytime() {
    assert!(TimeOfDay::EarlyMorning.is_daytime());
}

#[test]
fn tod_morning_is_daytime() {
    assert!(TimeOfDay::Morning.is_daytime());
}

#[test]
fn tod_noon_is_daytime() {
    assert!(TimeOfDay::Noon.is_daytime());
}

#[test]
fn tod_afternoon_is_daytime() {
    assert!(TimeOfDay::Afternoon.is_daytime());
}

#[test]
fn tod_golden_hour_is_daytime() {
    assert!(TimeOfDay::GoldenHour.is_daytime());
}

#[test]
fn tod_sunset_is_daytime() {
    assert!(TimeOfDay::Sunset.is_daytime()); // 3.0 > 0
}

#[test]
fn tod_dusk_not_daytime() {
    assert!(!TimeOfDay::Dusk.is_daytime()); // -5.0
}

#[test]
fn tod_blue_hour_not_daytime() {
    assert!(!TimeOfDay::BlueHour.is_daytime()); // -10.0
}

#[test]
fn tod_night_not_daytime() {
    assert!(!TimeOfDay::Night.is_daytime()); // -30.0
}

#[test]
fn tod_midnight_not_daytime() {
    assert!(!TimeOfDay::Midnight.is_daytime()); // -45.0
}

#[test]
fn tod_late_night_not_daytime() {
    assert!(!TimeOfDay::LateNight.is_daytime()); // -35.0
}

// ============================================================================
// TIME OF DAY — NAME / ICON / DISPLAY
// ============================================================================

#[test]
fn tod_name_for_each_variant() {
    assert_eq!(TimeOfDay::Dawn.name(), "Dawn");
    assert_eq!(TimeOfDay::EarlyMorning.name(), "Early Morning");
    assert_eq!(TimeOfDay::Morning.name(), "Morning");
    assert_eq!(TimeOfDay::Noon.name(), "Noon");
    assert_eq!(TimeOfDay::Afternoon.name(), "Afternoon");
    assert_eq!(TimeOfDay::GoldenHour.name(), "Golden Hour");
    assert_eq!(TimeOfDay::Sunset.name(), "Sunset");
    assert_eq!(TimeOfDay::Dusk.name(), "Dusk");
    assert_eq!(TimeOfDay::BlueHour.name(), "Blue Hour");
    assert_eq!(TimeOfDay::Night.name(), "Night");
    assert_eq!(TimeOfDay::Midnight.name(), "Midnight");
    assert_eq!(TimeOfDay::LateNight.name(), "Late Night");
}

#[test]
fn tod_icon_for_each_variant() {
    // Just verify non-empty return for each
    for tod in TimeOfDay::all() {
        assert!(!tod.icon().is_empty());
    }
}

#[test]
fn tod_display_contains_name() {
    for tod in TimeOfDay::all() {
        let s = format!("{}", tod);
        assert!(s.contains(tod.name()), "Display for {:?} missing name", tod);
    }
}

#[test]
fn tod_default_is_noon() {
    assert_eq!(TimeOfDay::default(), TimeOfDay::Noon);
}

// ============================================================================
// WEATHER CONDITION — CLOUD COVERAGE (14 variants)
// ============================================================================

#[test]
fn weather_clear_cloud_coverage() {
    assert!((WeatherCondition::Clear.cloud_coverage() - 0.0).abs() < 0.01);
}

#[test]
fn weather_partly_cloudy_cloud_coverage() {
    assert!((WeatherCondition::PartlyCloudy.cloud_coverage() - 0.3).abs() < 0.01);
}

#[test]
fn weather_overcast_cloud_coverage() {
    assert!((WeatherCondition::Overcast.cloud_coverage() - 0.9).abs() < 0.01);
}

#[test]
fn weather_light_rain_cloud_coverage() {
    assert!((WeatherCondition::LightRain.cloud_coverage() - 0.7).abs() < 0.01);
}

#[test]
fn weather_heavy_rain_cloud_coverage() {
    assert!((WeatherCondition::HeavyRain.cloud_coverage() - 0.95).abs() < 0.01);
}

#[test]
fn weather_thunderstorm_cloud_coverage() {
    assert!((WeatherCondition::Thunderstorm.cloud_coverage() - 1.0).abs() < 0.01);
}

#[test]
fn weather_light_snow_cloud_coverage() {
    assert!((WeatherCondition::LightSnow.cloud_coverage() - 0.6).abs() < 0.01);
}

#[test]
fn weather_heavy_snow_cloud_coverage() {
    assert!((WeatherCondition::HeavySnow.cloud_coverage() - 0.85).abs() < 0.01);
}

#[test]
fn weather_blizzard_cloud_coverage() {
    assert!((WeatherCondition::Blizzard.cloud_coverage() - 1.0).abs() < 0.01);
}

#[test]
fn weather_foggy_cloud_coverage() {
    assert!((WeatherCondition::Foggy.cloud_coverage() - 0.4).abs() < 0.01);
}

#[test]
fn weather_dense_fog_cloud_coverage() {
    assert!((WeatherCondition::DenseFog.cloud_coverage() - 0.5).abs() < 0.01);
}

#[test]
fn weather_sandstorm_cloud_coverage() {
    assert!((WeatherCondition::Sandstorm.cloud_coverage() - 0.3).abs() < 0.01);
}

#[test]
fn weather_haze_cloud_coverage() {
    assert!((WeatherCondition::Haze.cloud_coverage() - 0.2).abs() < 0.01);
}

#[test]
fn weather_windy_cloud_coverage() {
    assert!((WeatherCondition::Windy.cloud_coverage() - 0.2).abs() < 0.01);
}

// ============================================================================
// WEATHER CONDITION — FOG MULTIPLIER (14 variants)
// ============================================================================

#[test]
fn weather_clear_fog_multiplier() {
    assert!((WeatherCondition::Clear.fog_multiplier() - 0.2).abs() < 0.01);
}

#[test]
fn weather_partly_cloudy_fog_multiplier() {
    assert!((WeatherCondition::PartlyCloudy.fog_multiplier() - 0.3).abs() < 0.01);
}

#[test]
fn weather_overcast_fog_multiplier() {
    assert!((WeatherCondition::Overcast.fog_multiplier() - 0.5).abs() < 0.01);
}

#[test]
fn weather_light_rain_fog_multiplier() {
    assert!((WeatherCondition::LightRain.fog_multiplier() - 0.6).abs() < 0.01);
}

#[test]
fn weather_heavy_rain_fog_multiplier() {
    assert!((WeatherCondition::HeavyRain.fog_multiplier() - 0.8).abs() < 0.01);
}

#[test]
fn weather_thunderstorm_fog_multiplier() {
    assert!((WeatherCondition::Thunderstorm.fog_multiplier() - 0.9).abs() < 0.01);
}

#[test]
fn weather_light_snow_fog_multiplier() {
    assert!((WeatherCondition::LightSnow.fog_multiplier() - 0.5).abs() < 0.01);
}

#[test]
fn weather_heavy_snow_fog_multiplier() {
    assert!((WeatherCondition::HeavySnow.fog_multiplier() - 0.7).abs() < 0.01);
}

#[test]
fn weather_blizzard_fog_multiplier() {
    assert!((WeatherCondition::Blizzard.fog_multiplier() - 0.95).abs() < 0.01);
}

#[test]
fn weather_foggy_fog_multiplier() {
    assert!((WeatherCondition::Foggy.fog_multiplier() - 1.0).abs() < 0.01);
}

#[test]
fn weather_dense_fog_fog_multiplier() {
    assert!((WeatherCondition::DenseFog.fog_multiplier() - 1.5).abs() < 0.01);
}

#[test]
fn weather_sandstorm_fog_multiplier() {
    assert!((WeatherCondition::Sandstorm.fog_multiplier() - 1.2).abs() < 0.01);
}

#[test]
fn weather_haze_fog_multiplier() {
    assert!((WeatherCondition::Haze.fog_multiplier() - 0.7).abs() < 0.01);
}

#[test]
fn weather_windy_fog_multiplier() {
    assert!((WeatherCondition::Windy.fog_multiplier() - 0.3).abs() < 0.01);
}

// ============================================================================
// WEATHER CONDITION — SUN ATTENUATION (14 variants)
// ============================================================================

#[test]
fn weather_clear_sun_attenuation() {
    assert!((WeatherCondition::Clear.sun_attenuation() - 0.0).abs() < 0.01);
}

#[test]
fn weather_partly_cloudy_sun_attenuation() {
    assert!((WeatherCondition::PartlyCloudy.sun_attenuation() - 0.2).abs() < 0.01);
}

#[test]
fn weather_overcast_sun_attenuation() {
    assert!((WeatherCondition::Overcast.sun_attenuation() - 0.7).abs() < 0.01);
}

#[test]
fn weather_light_rain_sun_attenuation() {
    assert!((WeatherCondition::LightRain.sun_attenuation() - 0.6).abs() < 0.01);
}

#[test]
fn weather_heavy_rain_sun_attenuation() {
    assert!((WeatherCondition::HeavyRain.sun_attenuation() - 0.85).abs() < 0.01);
}

#[test]
fn weather_thunderstorm_sun_attenuation() {
    assert!((WeatherCondition::Thunderstorm.sun_attenuation() - 0.95).abs() < 0.01);
}

#[test]
fn weather_light_snow_sun_attenuation() {
    assert!((WeatherCondition::LightSnow.sun_attenuation() - 0.5).abs() < 0.01);
}

#[test]
fn weather_heavy_snow_sun_attenuation() {
    assert!((WeatherCondition::HeavySnow.sun_attenuation() - 0.7).abs() < 0.01);
}

#[test]
fn weather_blizzard_sun_attenuation() {
    assert!((WeatherCondition::Blizzard.sun_attenuation() - 0.9).abs() < 0.01);
}

#[test]
fn weather_foggy_sun_attenuation() {
    assert!((WeatherCondition::Foggy.sun_attenuation() - 0.4).abs() < 0.01);
}

#[test]
fn weather_dense_fog_sun_attenuation() {
    assert!((WeatherCondition::DenseFog.sun_attenuation() - 0.7).abs() < 0.01);
}

#[test]
fn weather_sandstorm_sun_attenuation() {
    assert!((WeatherCondition::Sandstorm.sun_attenuation() - 0.6).abs() < 0.01);
}

#[test]
fn weather_haze_sun_attenuation() {
    assert!((WeatherCondition::Haze.sun_attenuation() - 0.3).abs() < 0.01);
}

#[test]
fn weather_windy_sun_attenuation() {
    assert!((WeatherCondition::Windy.sun_attenuation() - 0.0).abs() < 0.01);
}

// ============================================================================
// WEATHER CONDITION — HAS PRECIPITATION (14 variants)
// ============================================================================

#[test]
fn weather_clear_no_precipitation() {
    assert!(!WeatherCondition::Clear.has_precipitation());
}

#[test]
fn weather_partly_cloudy_no_precipitation() {
    assert!(!WeatherCondition::PartlyCloudy.has_precipitation());
}

#[test]
fn weather_overcast_no_precipitation() {
    assert!(!WeatherCondition::Overcast.has_precipitation());
}

#[test]
fn weather_light_rain_has_precipitation() {
    assert!(WeatherCondition::LightRain.has_precipitation());
}

#[test]
fn weather_heavy_rain_has_precipitation() {
    assert!(WeatherCondition::HeavyRain.has_precipitation());
}

#[test]
fn weather_thunderstorm_has_precipitation() {
    assert!(WeatherCondition::Thunderstorm.has_precipitation());
}

#[test]
fn weather_light_snow_has_precipitation() {
    assert!(WeatherCondition::LightSnow.has_precipitation());
}

#[test]
fn weather_heavy_snow_has_precipitation() {
    assert!(WeatherCondition::HeavySnow.has_precipitation());
}

#[test]
fn weather_blizzard_has_precipitation() {
    assert!(WeatherCondition::Blizzard.has_precipitation());
}

#[test]
fn weather_foggy_no_precipitation() {
    assert!(!WeatherCondition::Foggy.has_precipitation());
}

#[test]
fn weather_dense_fog_no_precipitation() {
    assert!(!WeatherCondition::DenseFog.has_precipitation());
}

#[test]
fn weather_sandstorm_no_precipitation() {
    assert!(!WeatherCondition::Sandstorm.has_precipitation());
}

#[test]
fn weather_haze_no_precipitation() {
    assert!(!WeatherCondition::Haze.has_precipitation());
}

#[test]
fn weather_windy_no_precipitation() {
    assert!(!WeatherCondition::Windy.has_precipitation());
}

// ============================================================================
// WEATHER CONDITION — NAME / DISPLAY
// ============================================================================

#[test]
fn weather_name_all_variants() {
    assert_eq!(WeatherCondition::Clear.name(), "Clear");
    assert_eq!(WeatherCondition::PartlyCloudy.name(), "Partly Cloudy");
    assert_eq!(WeatherCondition::Overcast.name(), "Overcast");
    assert_eq!(WeatherCondition::LightRain.name(), "Light Rain");
    assert_eq!(WeatherCondition::HeavyRain.name(), "Heavy Rain");
    assert_eq!(WeatherCondition::Thunderstorm.name(), "Thunderstorm");
    assert_eq!(WeatherCondition::LightSnow.name(), "Light Snow");
    assert_eq!(WeatherCondition::HeavySnow.name(), "Heavy Snow");
    assert_eq!(WeatherCondition::Blizzard.name(), "Blizzard");
    assert_eq!(WeatherCondition::Foggy.name(), "Foggy");
    assert_eq!(WeatherCondition::DenseFog.name(), "Dense Fog");
    assert_eq!(WeatherCondition::Sandstorm.name(), "Sandstorm");
    assert_eq!(WeatherCondition::Haze.name(), "Haze");
    assert_eq!(WeatherCondition::Windy.name(), "Windy");
}

#[test]
fn weather_display_contains_name() {
    for w in WeatherCondition::all() {
        let s = format!("{}", w);
        assert!(s.contains(w.name()), "Display for {:?} missing name", w);
    }
}

#[test]
fn weather_default_is_clear() {
    assert_eq!(WeatherCondition::default(), WeatherCondition::Clear);
}

// ============================================================================
// SKY TYPE — SUPPORTS_TIME_OF_DAY (4 variants)
// ============================================================================

#[test]
fn sky_procedural_supports_time_of_day() {
    assert!(SkyType::Procedural.supports_time_of_day());
}

#[test]
fn sky_hdri_no_time_of_day() {
    assert!(!SkyType::Hdri.supports_time_of_day());
}

#[test]
fn sky_solid_color_no_time_of_day() {
    assert!(!SkyType::SolidColor.supports_time_of_day());
}

#[test]
fn sky_gradient_no_time_of_day() {
    assert!(!SkyType::Gradient.supports_time_of_day());
}

#[test]
fn sky_type_all_count() {
    assert_eq!(SkyType::all().len(), 4);
}

#[test]
fn sky_type_names() {
    assert_eq!(SkyType::Procedural.name(), "Procedural");
    assert_eq!(SkyType::Hdri.name(), "HDRI");
    assert_eq!(SkyType::SolidColor.name(), "Solid Color");
    assert_eq!(SkyType::Gradient.name(), "Gradient");
}

#[test]
fn sky_type_default_is_procedural() {
    assert_eq!(SkyType::default(), SkyType::Procedural);
}

// ============================================================================
// FOG TYPE — PERFORMANCE COST (6 variants)
// ============================================================================

#[test]
fn fog_none_performance_cost() {
    assert_eq!(FogType::None.performance_cost(), 0);
}

#[test]
fn fog_linear_performance_cost() {
    assert_eq!(FogType::Linear.performance_cost(), 1);
}

#[test]
fn fog_exponential_performance_cost() {
    assert_eq!(FogType::Exponential.performance_cost(), 1);
}

#[test]
fn fog_exponential_squared_performance_cost() {
    assert_eq!(FogType::ExponentialSquared.performance_cost(), 1);
}

#[test]
fn fog_height_performance_cost() {
    assert_eq!(FogType::Height.performance_cost(), 2);
}

#[test]
fn fog_volumetric_performance_cost() {
    assert_eq!(FogType::Volumetric.performance_cost(), 4);
}

#[test]
fn fog_type_all_count() {
    assert_eq!(FogType::all().len(), 6);
}

#[test]
fn fog_type_names() {
    assert_eq!(FogType::None.name(), "None");
    assert_eq!(FogType::Linear.name(), "Linear");
    assert_eq!(FogType::Exponential.name(), "Exponential");
    assert_eq!(FogType::ExponentialSquared.name(), "Exp²");
    assert_eq!(FogType::Height.name(), "Height");
    assert_eq!(FogType::Volumetric.name(), "Volumetric");
}

#[test]
fn fog_type_default_is_linear() {
    assert_eq!(FogType::default(), FogType::Linear);
}

// ============================================================================
// TONEMAPPER — IS_COLOR_PRESERVING / IS_FILMIC (8 variants)
// ============================================================================

#[test]
fn tonemapper_none_not_color_preserving() {
    assert!(!Tonemapper::None.is_color_preserving());
}

#[test]
fn tonemapper_reinhard_is_color_preserving() {
    assert!(Tonemapper::Reinhard.is_color_preserving());
}

#[test]
fn tonemapper_reinhard_extended_is_color_preserving() {
    assert!(Tonemapper::ReinhardExtended.is_color_preserving());
}

#[test]
fn tonemapper_aces_not_color_preserving() {
    assert!(!Tonemapper::Aces.is_color_preserving());
}

#[test]
fn tonemapper_aces_narkowicz_not_color_preserving() {
    assert!(!Tonemapper::AcesNarkowicz.is_color_preserving());
}

#[test]
fn tonemapper_uncharted2_not_color_preserving() {
    assert!(!Tonemapper::Uncharted2.is_color_preserving());
}

#[test]
fn tonemapper_khronos_not_color_preserving() {
    assert!(!Tonemapper::Khronos.is_color_preserving());
}

#[test]
fn tonemapper_agx_not_color_preserving() {
    assert!(!Tonemapper::AgX.is_color_preserving());
}

#[test]
fn tonemapper_none_not_filmic() {
    assert!(!Tonemapper::None.is_filmic());
}

#[test]
fn tonemapper_reinhard_not_filmic() {
    assert!(!Tonemapper::Reinhard.is_filmic());
}

#[test]
fn tonemapper_aces_is_filmic() {
    assert!(Tonemapper::Aces.is_filmic());
}

#[test]
fn tonemapper_aces_narkowicz_is_filmic() {
    assert!(Tonemapper::AcesNarkowicz.is_filmic());
}

#[test]
fn tonemapper_uncharted2_is_filmic() {
    assert!(Tonemapper::Uncharted2.is_filmic());
}

#[test]
fn tonemapper_khronos_not_filmic() {
    assert!(!Tonemapper::Khronos.is_filmic());
}

#[test]
fn tonemapper_agx_not_filmic() {
    assert!(!Tonemapper::AgX.is_filmic());
}

#[test]
fn tonemapper_all_count() {
    assert_eq!(Tonemapper::all().len(), 8);
}

#[test]
fn tonemapper_names() {
    assert_eq!(Tonemapper::None.name(), "None");
    assert_eq!(Tonemapper::Reinhard.name(), "Reinhard");
    assert_eq!(Tonemapper::ReinhardExtended.name(), "Reinhard Extended");
    assert_eq!(Tonemapper::Aces.name(), "ACES");
    assert_eq!(Tonemapper::AcesNarkowicz.name(), "ACES (Narkowicz)");
    assert_eq!(Tonemapper::Uncharted2.name(), "Uncharted 2");
    assert_eq!(Tonemapper::Khronos.name(), "Khronos PBR Neutral");
    assert_eq!(Tonemapper::AgX.name(), "AgX");
}

#[test]
fn tonemapper_descriptions() {
    assert_eq!(Tonemapper::None.description(), "No tonemapping (raw HDR)");
    assert_eq!(
        Tonemapper::Aces.description(),
        "Academy standard filmic look"
    );
    assert_eq!(Tonemapper::AgX.description(), "Modern neutral look");
}

#[test]
fn tonemapper_default_is_aces() {
    assert_eq!(Tonemapper::default(), Tonemapper::Aces);
}

// ============================================================================
// MOOD PRESET — RECOMMENDED TONEMAPPER (14 variants)
// ============================================================================

#[test]
fn mood_neutral_tonemapper() {
    assert_eq!(
        MoodPreset::Neutral.recommended_tonemapper(),
        Tonemapper::Khronos
    );
}

#[test]
fn mood_bright_tonemapper() {
    assert_eq!(
        MoodPreset::Bright.recommended_tonemapper(),
        Tonemapper::Reinhard
    );
}

#[test]
fn mood_moody_tonemapper() {
    assert_eq!(
        MoodPreset::Moody.recommended_tonemapper(),
        Tonemapper::Uncharted2
    );
}

#[test]
fn mood_dramatic_tonemapper() {
    assert_eq!(
        MoodPreset::Dramatic.recommended_tonemapper(),
        Tonemapper::Aces
    );
}

#[test]
fn mood_horror_tonemapper() {
    assert_eq!(
        MoodPreset::Horror.recommended_tonemapper(),
        Tonemapper::Uncharted2
    );
}

#[test]
fn mood_cinematic_tonemapper() {
    assert_eq!(
        MoodPreset::Cinematic.recommended_tonemapper(),
        Tonemapper::Aces
    );
}

#[test]
fn mood_dreamy_tonemapper() {
    assert_eq!(
        MoodPreset::Dreamy.recommended_tonemapper(),
        Tonemapper::ReinhardExtended
    );
}

#[test]
fn mood_vintage_tonemapper() {
    assert_eq!(
        MoodPreset::Vintage.recommended_tonemapper(),
        Tonemapper::AgX
    );
}

#[test]
fn mood_cyberpunk_tonemapper() {
    assert_eq!(
        MoodPreset::CyberPunk.recommended_tonemapper(),
        Tonemapper::Aces
    );
}

#[test]
fn mood_desert_tonemapper() {
    assert_eq!(
        MoodPreset::Desert.recommended_tonemapper(),
        Tonemapper::Aces
    );
}

#[test]
fn mood_arctic_tonemapper() {
    assert_eq!(
        MoodPreset::Arctic.recommended_tonemapper(),
        Tonemapper::Reinhard
    );
}

#[test]
fn mood_tropical_tonemapper() {
    assert_eq!(
        MoodPreset::Tropical.recommended_tonemapper(),
        Tonemapper::Aces
    );
}

#[test]
fn mood_noir_tonemapper() {
    assert_eq!(
        MoodPreset::Noir.recommended_tonemapper(),
        Tonemapper::Uncharted2
    );
}

#[test]
fn mood_fantasy_tonemapper() {
    assert_eq!(
        MoodPreset::Fantasy.recommended_tonemapper(),
        Tonemapper::Reinhard
    );
}

// ============================================================================
// MOOD PRESET — CONTRAST (14 variants)
// ============================================================================

#[test]
fn mood_neutral_contrast() {
    assert!((MoodPreset::Neutral.contrast() - 0.0).abs() < 0.001);
}

#[test]
fn mood_bright_contrast() {
    assert!((MoodPreset::Bright.contrast() - (-0.1)).abs() < 0.001);
}

#[test]
fn mood_moody_contrast() {
    assert!((MoodPreset::Moody.contrast() - 0.2).abs() < 0.001);
}

#[test]
fn mood_dramatic_contrast() {
    assert!((MoodPreset::Dramatic.contrast() - 0.4).abs() < 0.001);
}

#[test]
fn mood_horror_contrast() {
    assert!((MoodPreset::Horror.contrast() - 0.3).abs() < 0.001);
}

#[test]
fn mood_cinematic_contrast() {
    assert!((MoodPreset::Cinematic.contrast() - 0.15).abs() < 0.001);
}

#[test]
fn mood_dreamy_contrast() {
    assert!((MoodPreset::Dreamy.contrast() - (-0.2)).abs() < 0.001);
}

#[test]
fn mood_vintage_contrast() {
    assert!((MoodPreset::Vintage.contrast() - 0.1).abs() < 0.001);
}

#[test]
fn mood_cyberpunk_contrast() {
    assert!((MoodPreset::CyberPunk.contrast() - 0.25).abs() < 0.001);
}

#[test]
fn mood_desert_contrast() {
    assert!((MoodPreset::Desert.contrast() - 0.1).abs() < 0.001);
}

#[test]
fn mood_arctic_contrast() {
    assert!((MoodPreset::Arctic.contrast() - 0.0).abs() < 0.001);
}

#[test]
fn mood_tropical_contrast() {
    assert!((MoodPreset::Tropical.contrast() - 0.1).abs() < 0.001);
}

#[test]
fn mood_noir_contrast() {
    assert!((MoodPreset::Noir.contrast() - 0.5).abs() < 0.001);
}

#[test]
fn mood_fantasy_contrast() {
    assert!((MoodPreset::Fantasy.contrast() - 0.05).abs() < 0.001);
}

// ============================================================================
// MOOD PRESET — SATURATION (14 variants)
// ============================================================================

#[test]
fn mood_neutral_saturation() {
    assert!((MoodPreset::Neutral.saturation() - 0.0).abs() < 0.001);
}

#[test]
fn mood_bright_saturation() {
    assert!((MoodPreset::Bright.saturation() - 0.15).abs() < 0.001);
}

#[test]
fn mood_moody_saturation() {
    assert!((MoodPreset::Moody.saturation() - (-0.2)).abs() < 0.001);
}

#[test]
fn mood_dramatic_saturation() {
    assert!((MoodPreset::Dramatic.saturation() - 0.0).abs() < 0.001);
}

#[test]
fn mood_horror_saturation() {
    assert!((MoodPreset::Horror.saturation() - (-0.3)).abs() < 0.001);
}

#[test]
fn mood_cinematic_saturation() {
    assert!((MoodPreset::Cinematic.saturation() - 0.05).abs() < 0.001);
}

#[test]
fn mood_dreamy_saturation() {
    assert!((MoodPreset::Dreamy.saturation() - 0.1).abs() < 0.001);
}

#[test]
fn mood_vintage_saturation() {
    assert!((MoodPreset::Vintage.saturation() - (-0.15)).abs() < 0.001);
}

#[test]
fn mood_cyberpunk_saturation() {
    assert!((MoodPreset::CyberPunk.saturation() - 0.3).abs() < 0.001);
}

#[test]
fn mood_desert_saturation() {
    assert!((MoodPreset::Desert.saturation() - 0.1).abs() < 0.001);
}

#[test]
fn mood_arctic_saturation() {
    assert!((MoodPreset::Arctic.saturation() - (-0.2)).abs() < 0.001);
}

#[test]
fn mood_tropical_saturation() {
    assert!((MoodPreset::Tropical.saturation() - 0.25).abs() < 0.001);
}

#[test]
fn mood_noir_saturation() {
    assert!((MoodPreset::Noir.saturation() - (-0.8)).abs() < 0.001);
}

#[test]
fn mood_fantasy_saturation() {
    assert!((MoodPreset::Fantasy.saturation() - 0.2).abs() < 0.001);
}

#[test]
fn mood_all_count() {
    assert_eq!(MoodPreset::all().len(), 14);
}

#[test]
fn mood_default_is_neutral() {
    assert_eq!(MoodPreset::default(), MoodPreset::Neutral);
}

#[test]
fn mood_name_all_variants() {
    assert_eq!(MoodPreset::Neutral.name(), "Neutral");
    assert_eq!(MoodPreset::Bright.name(), "Bright");
    assert_eq!(MoodPreset::Moody.name(), "Moody");
    assert_eq!(MoodPreset::Dramatic.name(), "Dramatic");
    assert_eq!(MoodPreset::Horror.name(), "Horror");
    assert_eq!(MoodPreset::Cinematic.name(), "Cinematic");
    assert_eq!(MoodPreset::Dreamy.name(), "Dreamy");
    assert_eq!(MoodPreset::Vintage.name(), "Vintage");
    assert_eq!(MoodPreset::CyberPunk.name(), "Cyberpunk");
    assert_eq!(MoodPreset::Desert.name(), "Desert");
    assert_eq!(MoodPreset::Arctic.name(), "Arctic");
    assert_eq!(MoodPreset::Tropical.name(), "Tropical");
    assert_eq!(MoodPreset::Noir.name(), "Noir");
    assert_eq!(MoodPreset::Fantasy.name(), "Fantasy");
}

// ============================================================================
// ENVIRONMENT SETTINGS — FROM_MOOD / APPLY_TIME / APPLY_WEATHER
// ============================================================================

#[test]
fn settings_from_mood_horror() {
    let s = EnvironmentSettings::from_mood(MoodPreset::Horror);
    assert_eq!(s.mood, MoodPreset::Horror);
    assert_eq!(s.tonemapper, Tonemapper::Uncharted2);
    assert!((s.contrast - 0.3).abs() < 0.001);
    assert!((s.saturation - (-0.3)).abs() < 0.001);
}

#[test]
fn settings_from_mood_cinematic() {
    let s = EnvironmentSettings::from_mood(MoodPreset::Cinematic);
    assert_eq!(s.tonemapper, Tonemapper::Aces);
    assert!((s.contrast - 0.15).abs() < 0.001);
}

#[test]
fn settings_from_mood_noir() {
    let s = EnvironmentSettings::from_mood(MoodPreset::Noir);
    assert_eq!(s.tonemapper, Tonemapper::Uncharted2);
    assert!((s.saturation - (-0.8)).abs() < 0.001);
}

#[test]
fn settings_default_values() {
    let s = EnvironmentSettings::default();
    assert_eq!(s.time_of_day, TimeOfDay::Noon);
    assert_eq!(s.weather, WeatherCondition::Clear);
    assert_eq!(s.sky_type, SkyType::Procedural);
    assert!((s.sky_intensity - 1.0).abs() < 0.001);
    assert_eq!(s.fog_type, FogType::Linear);
    assert!((s.fog_density - 0.001).abs() < 0.0001);
    assert!((s.fog_start - 50.0).abs() < 0.01);
    assert!((s.fog_end - 500.0).abs() < 0.01);
    assert!((s.sun_intensity - 1.0).abs() < 0.001);
    assert!((s.ambient_intensity - 0.3).abs() < 0.001);
    assert_eq!(s.tonemapper, Tonemapper::Aces);
    assert!((s.exposure - 0.0).abs() < 0.001);
    assert!((s.vignette - 0.0).abs() < 0.001);
    assert_eq!(s.mood, MoodPreset::Neutral);
}

#[test]
fn settings_apply_time_daytime() {
    let mut s = EnvironmentSettings::default();
    s.apply_time(TimeOfDay::Noon);
    assert_eq!(s.time_of_day, TimeOfDay::Noon);
    assert!(s.sun_intensity > 0.0, "Daytime should have positive sun");
}

#[test]
fn settings_apply_time_nighttime() {
    let mut s = EnvironmentSettings::default();
    s.apply_time(TimeOfDay::Midnight);
    assert_eq!(s.time_of_day, TimeOfDay::Midnight);
    assert!(
        (s.sun_intensity - 0.0).abs() < 0.001,
        "Night should have zero sun"
    );
    assert!((s.ambient_intensity - 0.1).abs() < 0.001);
}

#[test]
fn settings_apply_weather_clear() {
    let mut s = EnvironmentSettings::default();
    let fog_before = s.fog_density;
    s.apply_weather(WeatherCondition::Clear);
    assert_eq!(s.weather, WeatherCondition::Clear);
    // Clear has fog_multiplier 0.2, so fog density decreases
    assert!(s.fog_density < fog_before + 0.001);
}

#[test]
fn settings_apply_weather_dense_fog() {
    let mut s = EnvironmentSettings::default();
    let fog_before = s.fog_density;
    s.apply_weather(WeatherCondition::DenseFog);
    // DenseFog multiplier 1.5, so fog_density *= 1.5
    let expected = fog_before * 1.5;
    assert!((s.fog_density - expected).abs() < 0.0001);
}

#[test]
fn settings_apply_weather_attenuates_sun() {
    let mut s = EnvironmentSettings::default();
    s.apply_weather(WeatherCondition::Thunderstorm);
    // Thunderstorm attenuation 0.95 → sun * 0.05
    assert!(
        s.sun_intensity < 0.1,
        "Thunderstorm should dim sun to ~0.05"
    );
}

// ============================================================================
// ENVIRONMENT PRESET PANEL — STATE MANAGEMENT
// ============================================================================

#[test]
fn panel_new_defaults() {
    let p = EnvironmentPresetPanel::new();
    assert!(p.preview_enabled);
    assert!((p.transition_duration - 2.0).abs() < 0.001);
    assert!(p.saved_presets.is_empty());
    assert!(!p.has_pending_actions());
}

#[test]
fn panel_take_actions_empty() {
    let mut p = EnvironmentPresetPanel::new();
    let actions = p.take_actions();
    assert!(actions.is_empty());
}

#[test]
fn panel_current_settings_returns_ref() {
    let p = EnvironmentPresetPanel::new();
    let s = p.current_settings();
    assert_eq!(s.time_of_day, TimeOfDay::Noon);
}
