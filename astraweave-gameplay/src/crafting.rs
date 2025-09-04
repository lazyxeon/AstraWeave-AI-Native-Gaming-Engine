use serde::{Serialize, Deserialize};
use crate::{Inventory, Item, ItemKind, EchoMod, ResourceKind};

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
            let have = inv.resources.iter().find(|(k,_)| *k == c.kind).map(|(_,n)|*n).unwrap_or(0);
            if have < c.count { return None; }
        }
        // pay costs
        for c in &r.costs {
            if !inv.remove_resource(c.kind, c.count) { return None; }
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
