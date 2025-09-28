use astraweave_quests::{Quest, QuestStep};

#[test]
fn test_quest_validation() {
    let quest = Quest {
        title: "Find the Relic".into(),
        steps: vec![QuestStep {
            description: "Talk to the elder.".into(),
            completed: false,
        }],
    };
    assert!(quest.validate().is_ok());
}

#[test]
fn test_quest_empty_title() {
    let quest = Quest {
        title: "".into(),
        steps: vec![QuestStep {
            description: "Step 1".into(),
            completed: false,
        }],
    };
    assert!(quest.validate().is_err());
}

#[test]
fn test_quest_completion() {
    let quest = Quest {
        title: "Test".into(),
        steps: vec![
            QuestStep {
                description: "Step 1".into(),
                completed: true,
            },
            QuestStep {
                description: "Step 2".into(),
                completed: true,
            },
        ],
    };
    assert!(quest.is_complete());
}
