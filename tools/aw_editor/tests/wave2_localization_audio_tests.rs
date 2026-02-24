//! Wave 2 mutation remediation tests — Localization + Audio panels
//! Covers: Language, StringCategory, ExportFormat, LocalizationTab, LocalizedString, PluralForms,
//!         MusicMood, SpatialPreset, ReverbEnvironment, AudioTab, AudioEmitterInfo

use aw_editor_lib::panels::audio_panel::{
    AudioEmitterInfo, AudioTab, MusicMood, MusicTrackEntry, ReverbEnvironment, SpatialPreset,
};
use aw_editor_lib::panels::localization_panel::{
    ExportFormat, Language, LocalizationTab, LocalizedString, PluralForms, StringCategory,
};

// ═══════════════════════════════════════════════════════════════════════════════════
// LANGUAGE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn language_all_count() {
    // all() excludes Custom variant
    assert_eq!(Language::all().len(), 12);
}

#[test]
fn language_codes() {
    assert_eq!(Language::English.code(), "en");
    assert_eq!(Language::Spanish.code(), "es");
    assert_eq!(Language::French.code(), "fr");
    assert_eq!(Language::German.code(), "de");
    assert_eq!(Language::Italian.code(), "it");
    assert_eq!(Language::Portuguese.code(), "pt");
    assert_eq!(Language::Russian.code(), "ru");
    assert_eq!(Language::Japanese.code(), "ja");
    assert_eq!(Language::Korean.code(), "ko");
    assert_eq!(Language::SimplifiedChinese.code(), "zh-CN");
    assert_eq!(Language::TraditionalChinese.code(), "zh-TW");
    assert_eq!(Language::Arabic.code(), "ar");
    assert_eq!(Language::Custom(42).code(), "custom");
}

#[test]
fn language_names() {
    assert_eq!(Language::English.name(), "English");
    assert_eq!(Language::Spanish.name(), "Español");
    assert_eq!(Language::French.name(), "Français");
    assert_eq!(Language::German.name(), "Deutsch");
    assert_eq!(Language::Italian.name(), "Italiano");
    assert_eq!(Language::Portuguese.name(), "Português");
    assert_eq!(Language::Russian.name(), "Русский");
    assert_eq!(Language::Japanese.name(), "日本語");
    assert_eq!(Language::Korean.name(), "한국어");
    assert_eq!(Language::SimplifiedChinese.name(), "简体中文");
    assert_eq!(Language::TraditionalChinese.name(), "繁體中文");
    assert_eq!(Language::Arabic.name(), "العربية");
    assert_eq!(Language::Custom(0).name(), "Custom");
}

#[test]
fn language_flag_emojis_non_empty() {
    for lang in Language::all() {
        assert!(!lang.flag_emoji().is_empty(), "{:?} flag empty", lang);
    }
    assert!(!Language::Custom(0).flag_emoji().is_empty());
}

#[test]
fn language_display() {
    for lang in Language::all() {
        let s = format!("{}", lang);
        assert!(
            s.contains(lang.name()),
            "Display missing name for {:?}",
            lang
        );
    }
}

#[test]
fn language_default_is_english() {
    assert_eq!(Language::default(), Language::English);
}

#[test]
fn language_custom_equality() {
    assert_eq!(Language::Custom(1), Language::Custom(1));
    assert_ne!(Language::Custom(1), Language::Custom(2));
}

// ═══════════════════════════════════════════════════════════════════════════════════
// STRING CATEGORY
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn string_category_all_count() {
    assert_eq!(StringCategory::all().len(), 8);
}

#[test]
fn string_category_names() {
    assert_eq!(StringCategory::Ui.name(), "UI");
    assert_eq!(StringCategory::Dialogue.name(), "Dialogue");
    assert_eq!(StringCategory::Quest.name(), "Quest");
    assert_eq!(StringCategory::Item.name(), "Item");
    assert_eq!(StringCategory::Achievement.name(), "Achievement");
    assert_eq!(StringCategory::Tutorial.name(), "Tutorial");
    assert_eq!(StringCategory::System.name(), "System");
    assert_eq!(StringCategory::Error.name(), "Error");
}

#[test]
fn string_category_icons_non_empty() {
    for cat in StringCategory::all() {
        assert!(!cat.icon().is_empty(), "{:?} icon empty", cat);
    }
}

#[test]
fn string_category_display() {
    for cat in StringCategory::all() {
        let s = format!("{}", cat);
        assert!(s.contains(cat.name()));
    }
}

#[test]
fn string_category_default_is_ui() {
    assert_eq!(StringCategory::default(), StringCategory::Ui);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// EXPORT FORMAT
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn export_format_all_count() {
    assert_eq!(ExportFormat::all().len(), 5);
}

#[test]
fn export_format_names() {
    assert_eq!(ExportFormat::Csv.name(), "CSV");
    assert_eq!(ExportFormat::Xliff.name(), "XLIFF");
    assert_eq!(ExportFormat::Po.name(), "PO (Gettext)");
    assert_eq!(ExportFormat::Json.name(), "JSON");
    assert_eq!(ExportFormat::Resx.name(), "RESX (.NET)");
}

#[test]
fn export_format_extensions() {
    assert_eq!(ExportFormat::Csv.extension(), ".csv");
    assert_eq!(ExportFormat::Xliff.extension(), ".xlf");
    assert_eq!(ExportFormat::Po.extension(), ".po");
    assert_eq!(ExportFormat::Json.extension(), ".json");
    assert_eq!(ExportFormat::Resx.extension(), ".resx");
}

#[test]
fn export_format_icons_non_empty() {
    for fmt in ExportFormat::all() {
        assert!(!fmt.icon().is_empty(), "{:?} icon empty", fmt);
    }
}

#[test]
fn export_format_default_is_csv() {
    assert_eq!(ExportFormat::default(), ExportFormat::Csv);
}

#[test]
fn export_format_display() {
    for fmt in ExportFormat::all() {
        let s = format!("{}", fmt);
        assert!(s.contains(fmt.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// LOCALIZATION TAB
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn localization_tab_all_count() {
    assert_eq!(LocalizationTab::all().len(), 5);
}

#[test]
fn localization_tab_names() {
    assert_eq!(LocalizationTab::Strings.name(), "Strings");
    assert_eq!(LocalizationTab::Languages.name(), "Languages");
    assert_eq!(LocalizationTab::Statistics.name(), "Statistics");
    assert_eq!(LocalizationTab::ImportExport.name(), "Import/Export");
    assert_eq!(LocalizationTab::Settings.name(), "Settings");
}

#[test]
fn localization_tab_icons_non_empty() {
    for tab in LocalizationTab::all() {
        assert!(!tab.icon().is_empty(), "{:?} icon empty", tab);
    }
}

#[test]
fn localization_tab_default_is_strings() {
    assert_eq!(LocalizationTab::default(), LocalizationTab::Strings);
}

#[test]
fn localization_tab_display() {
    for tab in LocalizationTab::all() {
        let s = format!("{}", tab);
        assert!(s.contains(tab.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// LOCALIZED STRING DEFAULTS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn localized_string_defaults() {
    let ls = LocalizedString::default();
    assert!(ls.key.is_empty());
    assert_eq!(ls.category, StringCategory::Ui);
    assert!(ls.context.is_empty());
    assert!(ls.translations.is_empty());
    assert!(!ls.needs_review);
    assert!(ls.max_length.is_none());
    assert!(ls.plural_forms.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════════
// PLURAL FORMS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn plural_forms_defaults() {
    let pf = PluralForms::default();
    assert!(pf.zero.is_empty());
    assert!(pf.one.is_empty());
    assert!(pf.two.is_empty());
    assert!(pf.few.is_empty());
    assert!(pf.many.is_empty());
    assert!(pf.other.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════════
// MUSIC MOOD
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn music_mood_all_count() {
    assert_eq!(MusicMood::all().len(), 9);
}

#[test]
fn music_mood_names() {
    assert_eq!(MusicMood::Ambient.name(), "Ambient");
    assert_eq!(MusicMood::Calm.name(), "Calm");
    assert_eq!(MusicMood::Exploration.name(), "Exploration");
    assert_eq!(MusicMood::Combat.name(), "Combat");
    assert_eq!(MusicMood::Tension.name(), "Tension");
    assert_eq!(MusicMood::Victory.name(), "Victory");
    assert_eq!(MusicMood::Defeat.name(), "Defeat");
    assert_eq!(MusicMood::Boss.name(), "Boss");
    assert_eq!(MusicMood::Menu.name(), "Menu");
}

#[test]
fn music_mood_icons_non_empty() {
    for mood in MusicMood::all() {
        assert!(!mood.icon().is_empty(), "{:?} icon empty", mood);
    }
}

#[test]
fn music_mood_is_combat_related() {
    assert!(!MusicMood::Ambient.is_combat_related());
    assert!(!MusicMood::Calm.is_combat_related());
    assert!(!MusicMood::Exploration.is_combat_related());
    assert!(MusicMood::Combat.is_combat_related());
    assert!(MusicMood::Tension.is_combat_related());
    assert!(!MusicMood::Victory.is_combat_related());
    assert!(!MusicMood::Defeat.is_combat_related());
    assert!(MusicMood::Boss.is_combat_related());
    assert!(!MusicMood::Menu.is_combat_related());
}

#[test]
fn music_mood_is_positive() {
    assert!(!MusicMood::Ambient.is_positive());
    assert!(MusicMood::Calm.is_positive());
    assert!(!MusicMood::Exploration.is_positive());
    assert!(!MusicMood::Combat.is_positive());
    assert!(!MusicMood::Tension.is_positive());
    assert!(MusicMood::Victory.is_positive());
    assert!(!MusicMood::Defeat.is_positive());
    assert!(!MusicMood::Boss.is_positive());
    assert!(!MusicMood::Menu.is_positive());
}

#[test]
fn music_mood_intensity_values() {
    assert_eq!(MusicMood::Ambient.intensity(), 1);
    assert_eq!(MusicMood::Calm.intensity(), 1);
    assert_eq!(MusicMood::Exploration.intensity(), 2);
    assert_eq!(MusicMood::Combat.intensity(), 4);
    assert_eq!(MusicMood::Tension.intensity(), 3);
    assert_eq!(MusicMood::Victory.intensity(), 4);
    assert_eq!(MusicMood::Defeat.intensity(), 2);
    assert_eq!(MusicMood::Boss.intensity(), 5);
    assert_eq!(MusicMood::Menu.intensity(), 1);
}

#[test]
fn music_mood_display() {
    for mood in MusicMood::all() {
        let s = format!("{}", mood);
        assert!(s.contains(mood.name()));
    }
}

#[test]
fn music_mood_default() {
    assert_eq!(MusicMood::default(), MusicMood::Ambient);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// SPATIAL PRESET
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn spatial_preset_all_count() {
    assert_eq!(SpatialPreset::all().len(), 5);
}

#[test]
fn spatial_preset_names() {
    assert_eq!(SpatialPreset::Standard.name(), "Standard");
    assert_eq!(SpatialPreset::Headphones.name(), "Headphones");
    assert_eq!(SpatialPreset::Speakers.name(), "Speakers");
    assert_eq!(SpatialPreset::Surround.name(), "Surround");
    assert_eq!(SpatialPreset::VR.name(), "VR");
}

#[test]
fn spatial_preset_ear_separation_values() {
    assert!((SpatialPreset::Standard.ear_separation() - 0.2).abs() < f32::EPSILON);
    assert!((SpatialPreset::Headphones.ear_separation() - 0.18).abs() < f32::EPSILON);
    assert!((SpatialPreset::Speakers.ear_separation() - 0.5).abs() < f32::EPSILON);
    assert!((SpatialPreset::Surround.ear_separation() - 0.25).abs() < f32::EPSILON);
    assert!((SpatialPreset::VR.ear_separation() - 0.2).abs() < f32::EPSILON);
}

#[test]
fn spatial_preset_is_multichannel() {
    assert!(!SpatialPreset::Standard.is_multichannel());
    assert!(!SpatialPreset::Headphones.is_multichannel());
    assert!(!SpatialPreset::Speakers.is_multichannel());
    assert!(SpatialPreset::Surround.is_multichannel());
    assert!(SpatialPreset::VR.is_multichannel());
}

#[test]
fn spatial_preset_description_non_empty() {
    for preset in SpatialPreset::all() {
        assert!(
            !preset.description().is_empty(),
            "{:?} description empty",
            preset
        );
    }
}

#[test]
fn spatial_preset_display() {
    for preset in SpatialPreset::all() {
        let s = format!("{}", preset);
        assert!(s.contains(preset.name()));
    }
}

#[test]
fn spatial_preset_default() {
    assert_eq!(SpatialPreset::default(), SpatialPreset::Standard);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// REVERB ENVIRONMENT
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn reverb_environment_all_count() {
    assert_eq!(ReverbEnvironment::all().len(), 8);
}

#[test]
fn reverb_environment_names() {
    assert_eq!(ReverbEnvironment::None.name(), "None");
    assert_eq!(ReverbEnvironment::SmallRoom.name(), "Small Room");
    assert_eq!(ReverbEnvironment::LargeRoom.name(), "Large Room");
    assert_eq!(ReverbEnvironment::Hall.name(), "Hall");
    assert_eq!(ReverbEnvironment::Cave.name(), "Cave");
    assert_eq!(ReverbEnvironment::Forest.name(), "Forest");
    assert_eq!(ReverbEnvironment::Underwater.name(), "Underwater");
    assert_eq!(ReverbEnvironment::Cathedral.name(), "Cathedral");
}

#[test]
fn reverb_environment_decay_times() {
    assert!((ReverbEnvironment::None.decay_time() - 0.0).abs() < f32::EPSILON);
    assert!((ReverbEnvironment::SmallRoom.decay_time() - 0.5).abs() < f32::EPSILON);
    assert!((ReverbEnvironment::LargeRoom.decay_time() - 1.2).abs() < f32::EPSILON);
    assert!((ReverbEnvironment::Hall.decay_time() - 2.5).abs() < f32::EPSILON);
    assert!((ReverbEnvironment::Cave.decay_time() - 4.0).abs() < f32::EPSILON);
    assert!((ReverbEnvironment::Forest.decay_time() - 0.8).abs() < f32::EPSILON);
    assert!((ReverbEnvironment::Underwater.decay_time() - 3.0).abs() < f32::EPSILON);
    assert!((ReverbEnvironment::Cathedral.decay_time() - 5.0).abs() < f32::EPSILON);
}

#[test]
fn reverb_environment_wet_dry_mix() {
    assert!((ReverbEnvironment::None.wet_dry_mix() - 0.0).abs() < f32::EPSILON);
    assert!((ReverbEnvironment::SmallRoom.wet_dry_mix() - 0.2).abs() < f32::EPSILON);
    assert!((ReverbEnvironment::LargeRoom.wet_dry_mix() - 0.3).abs() < f32::EPSILON);
    assert!((ReverbEnvironment::Hall.wet_dry_mix() - 0.4).abs() < f32::EPSILON);
    assert!((ReverbEnvironment::Cave.wet_dry_mix() - 0.6).abs() < f32::EPSILON);
    assert!((ReverbEnvironment::Forest.wet_dry_mix() - 0.25).abs() < f32::EPSILON);
    assert!((ReverbEnvironment::Underwater.wet_dry_mix() - 0.7).abs() < f32::EPSILON);
    assert!((ReverbEnvironment::Cathedral.wet_dry_mix() - 0.5).abs() < f32::EPSILON);
}

#[test]
fn reverb_environment_is_indoor() {
    assert!(!ReverbEnvironment::None.is_indoor());
    assert!(ReverbEnvironment::SmallRoom.is_indoor());
    assert!(ReverbEnvironment::LargeRoom.is_indoor());
    assert!(ReverbEnvironment::Hall.is_indoor());
    assert!(!ReverbEnvironment::Cave.is_indoor());
    assert!(!ReverbEnvironment::Forest.is_indoor());
    assert!(!ReverbEnvironment::Underwater.is_indoor());
    assert!(ReverbEnvironment::Cathedral.is_indoor());
}

#[test]
fn reverb_environment_is_natural() {
    assert!(!ReverbEnvironment::None.is_natural());
    assert!(!ReverbEnvironment::SmallRoom.is_natural());
    assert!(!ReverbEnvironment::LargeRoom.is_natural());
    assert!(!ReverbEnvironment::Hall.is_natural());
    assert!(ReverbEnvironment::Cave.is_natural());
    assert!(ReverbEnvironment::Forest.is_natural());
    assert!(ReverbEnvironment::Underwater.is_natural());
    assert!(!ReverbEnvironment::Cathedral.is_natural());
}

#[test]
fn reverb_environment_display() {
    for env in ReverbEnvironment::all() {
        let s = format!("{}", env);
        assert!(s.contains(env.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// AUDIO TAB
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn audio_tab_all_count() {
    assert_eq!(AudioTab::all().len(), 5);
}

#[test]
fn audio_tab_names() {
    assert_eq!(AudioTab::Mixer.name(), "Mixer");
    assert_eq!(AudioTab::Music.name(), "Music");
    assert_eq!(AudioTab::Spatial.name(), "Spatial");
    assert_eq!(AudioTab::Emitters.name(), "Emitters");
    assert_eq!(AudioTab::Preview.name(), "Preview");
}

#[test]
fn audio_tab_icons_non_empty() {
    for tab in AudioTab::all() {
        assert!(!tab.icon().is_empty(), "{:?} icon empty", tab);
    }
}

#[test]
fn audio_tab_default_is_mixer() {
    assert_eq!(AudioTab::default(), AudioTab::Mixer);
}

#[test]
fn audio_tab_display() {
    for tab in AudioTab::all() {
        let s = format!("{}", tab);
        assert!(s.contains(tab.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// AUDIO EMITTER INFO
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn audio_emitter_info_defaults() {
    let info = AudioEmitterInfo::default();
    assert!(info.name.is_empty());
    assert_eq!(info.position, [0.0, 0.0, 0.0]);
    assert!(!info.is_playing);
    assert!(info.current_sound.is_none());
    assert!((info.volume - 0.0).abs() < f32::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// MUSIC TRACK ENTRY
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn music_track_entry_defaults() {
    let entry = MusicTrackEntry::default();
    assert!(entry.name.is_empty());
    assert!(entry.path.is_empty());
    assert!((entry.duration_sec - 0.0).abs() < f32::EPSILON);
    assert!(entry.bpm.is_none());
    assert_eq!(entry.mood, MusicMood::Ambient);
}
