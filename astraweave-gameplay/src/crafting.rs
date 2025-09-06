use crate::{Inventory, Item, ItemKind, ResourceKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CraftCost {
    pub kind: ResourceKind,
    pub count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CraftRecipe {
    pub name: String,
    pub output_item: ItemKind,
    pub costs: Vec<CraftCost>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecipeBook {
    pub recipes: Vec<CraftRecipe>,
}

impl RecipeBook {
    pub fn craft(&self, name: &str, inv: &mut Inventory) -> Option<Item> {
        let r = self.recipes.iter().find(|r| r.name == name)?;
        // check costs
        for c in &r.costs {
            let have = inv
                .resources
                .iter()
                .find(|(k, _)| *k == c.kind)
                .map(|(_, n)| *n)
                .unwrap_or(0);
            if have < c.count {
                return None;
            }
        }
        // pay costs
        for c in &r.costs {
            if !inv.remove_resource(c.kind, c.count) {
                return None;
            }
        }
        // create item
        let itm = Item {
            id: rand::random::<u32>(),
            name: r.name.clone(),
            kind: r.output_item.clone(),
            echo: None,
        };
        Some(itm)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FactionStanding {
    pub name: String,
    pub reputation: i32,
} // -100..100

#[derive(Clone, Debug)]
pub struct CraftBench {
    pub quality: i32, // -2..+3
}

impl CraftBench {
    pub fn success_chance(
        &self,
        player_power: i32,
        faction: Option<&FactionStanding>,
        rarity: Option<&crate::items::Rarity>,
    ) -> f32 {
        let base = 0.75 + (self.quality as f32) * 0.05 + (player_power as f32) * 0.003;
        let fac = faction
            .map(|f| (f.reputation as f32) * 0.001)
            .unwrap_or(0.0);
        let rarity_penalty = match rarity {
            Some(crate::items::Rarity::Epic) => -0.15,
            Some(crate::items::Rarity::Legendary) => -0.30,
            _ => 0.0,
        };
        (base + fac + rarity_penalty).clamp(0.05, 0.98)
    }
}
