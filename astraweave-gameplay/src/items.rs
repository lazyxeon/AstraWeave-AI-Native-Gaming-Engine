use serde::{Serialize, Deserialize};
use crate::{DamageType, ResourceKind};

pub type ItemId = u32;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ItemKind {
    Weapon { base_damage: i32, dtype: DamageType },
    Armor  { defense: i32 },
    Consumable { heal: i32 },
    Material { r#type: ResourceKind },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EchoMod {
    pub name: String,
    pub power_mult: f32,       // multiplies base damage / power
    pub dtype_override: Option<DamageType>,
    pub special: Option<String>, // freeform tag for special behaviours
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub kind: ItemKind,
    pub echo: Option<EchoMod>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub items: Vec<Item>,
    pub resources: Vec<(ResourceKind, u32)>,
}

impl Inventory {
    pub fn add_resource(&mut self, kind: ResourceKind, n: u32) {
        if let Some((_,c)) = self.resources.iter_mut().find(|(k,_)| *k == kind) {
            *c += n;
        } else {
            self.resources.push((kind, n));
        }
    }

    pub fn remove_resource(&mut self, kind: ResourceKind, n: u32) -> bool {
        if let Some((_,c)) = self.resources.iter_mut().find(|(k,_)| *k == kind) {
            if *c >= n { *c -= n; return true; }
        }
        false
    }
}

pub fn infuse(item: &mut Item, echo: EchoMod) {
    item.echo = Some(echo);
}
