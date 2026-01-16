#[cfg(test)]
mod tests {
    use crate::polish::{SplashScreen, SplashSequence, LoadingStyle, LoadingScreen};
    use std::time::Duration;
    use std::path::PathBuf;

    #[test]
    fn test_splash_screen_defaults() {
        let splash = SplashScreen::default();
        assert_eq!(splash.image_path, PathBuf::from("splash.png"));
        assert_eq!(splash.duration, Some(Duration::from_secs(3)));
        assert!(splash.skippable);
    }

    #[test]
    fn test_splash_sequence_construction() {
        let seq = SplashSequence::new()
            .with_engine_logo()
            .with_publisher_logo("pub.png");
            
        assert_eq!(seq.screens.len(), 2);
        assert!(!seq.screens[0].skippable); // Engine logo usually not skippable
        assert!(seq.screens[1].skippable);
        assert_eq!(seq.screens[1].image_path, PathBuf::from("pub.png"));
    }

    #[test]
    fn test_sequence_total_duration() {
        let seq = SplashSequence::new()
             .with_engine_logo(); // 2s + 0.4 fadein + 0.4 fadeout = 2.8s
             
        let dur = seq.total_duration();
        // 2 + 0.4 + 0.4 = 2.8
        assert_eq!(dur.as_millis(), 2800);
    }

    #[test]
    fn test_loading_screen_style() {
        let style = LoadingStyle::default();
        assert_eq!(style, LoadingStyle::ProgressBar); 
    }
    
    #[test]
    fn test_loading_screen_builder() {
        let mut screen = LoadingScreen::default()
            .with_tips(vec!["Tip 1".to_string(), "Tip 2".to_string()]);
        
        screen.background_image = Some(PathBuf::from("bg.png"));
            
        assert_eq!(screen.tips.len(), 2);
        assert_eq!(screen.background_image, Some(PathBuf::from("bg.png")));
    }
}
