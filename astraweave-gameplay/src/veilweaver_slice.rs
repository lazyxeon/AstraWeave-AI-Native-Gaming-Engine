use anyhow::{Context, Result};
use astraweave_scene::world_partition::{Cell, CellComponentView, GridCoord};
use serde::de::DeserializeOwned;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct WeaveAnchorSpec {
    pub cell: GridCoord,
    pub position: [f32; 3],
    pub anchor_id: String,
    pub anchor_type: Option<String>,
    pub stability: Option<String>,
    pub echo_cost: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct TriggerZoneSpec {
    pub cell: GridCoord,
    pub position: [f32; 3],
    pub trigger_id: String,
    pub shape: Option<String>,
    pub radius: Option<f32>,
    pub extents: Option<[f32; 3]>,
}

#[derive(Debug, Clone)]
pub struct DecisionPromptSpec {
    pub cell: GridCoord,
    pub position: [f32; 3],
    pub trigger_id: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EncounterTriggerSpec {
    pub cell: GridCoord,
    pub position: [f32; 3],
    pub trigger_id: String,
    pub script: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EncounterCompleteSpec {
    pub cell: GridCoord,
    pub position: [f32; 3],
    pub trigger_id: String,
    pub next_cell: Option<GridCoord>,
}

#[derive(Debug, Clone)]
pub struct EffectAnchorSpec {
    pub cell: GridCoord,
    pub position: [f32; 3],
    pub trigger_id: Option<String>,
    pub effect: Option<String>,
    pub intensity: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct SpawnPointSpec {
    pub cell: GridCoord,
    pub position: [f32; 3],
    pub spawn_id: String,
    pub facing: Option<[f32; 3]>,
}

#[derive(Debug, Default, Clone)]
pub struct VeilweaverSliceMetadata {
    pub anchors: Vec<WeaveAnchorSpec>,
    pub trigger_zones: Vec<TriggerZoneSpec>,
    pub decision_prompts: Vec<DecisionPromptSpec>,
    pub encounter_triggers: Vec<EncounterTriggerSpec>,
    pub encounter_completes: Vec<EncounterCompleteSpec>,
    pub effect_anchors: Vec<EffectAnchorSpec>,
    pub spawn_points: Vec<SpawnPointSpec>,
}

impl VeilweaverSliceMetadata {
    pub fn extend_cell(&mut self, cell: &Cell) -> Result<()> {
        self.anchors
            .extend(parse_components(cell, "WeaveAnchor", parse_weave_anchor)?);
        self.trigger_zones.extend(parse_components(
            cell,
            "TriggerZone",
            parse_trigger_zone,
        )?);
        self.decision_prompts.extend(parse_components(
            cell,
            "DecisionPrompt",
            parse_decision_prompt,
        )?);
        self.encounter_triggers.extend(parse_components(
            cell,
            "EncounterTrigger",
            parse_encounter_trigger,
        )?);
        self.encounter_completes.extend(parse_components(
            cell,
            "EncounterComplete",
            parse_encounter_complete,
        )?);
        self.effect_anchors.extend(parse_components(
            cell,
            "EffectAnchor",
            parse_effect_anchor,
        )?);
        self.spawn_points
            .extend(parse_components(cell, "SpawnPoint", parse_spawn_point)?);
        Ok(())
    }

    pub fn from_cells(cells: &[&Cell]) -> Result<Self> {
        let mut meta = Self::default();
        for cell in cells {
            meta.extend_cell(cell)?;
        }
        Ok(meta)
    }
}

fn parse_components<T, F>(cell: &Cell, component_type: &str, parser: F) -> Result<Vec<T>>
where
    F: Fn(CellComponentView<'_>, GridCoord) -> Result<T>,
{
    cell.components_of_type(component_type)
        .map(|view| parser(view, cell.coord))
        .collect()
}

fn parse_weave_anchor(view: CellComponentView<'_>, coord: GridCoord) -> Result<WeaveAnchorSpec> {
    #[derive(Deserialize)]
    struct Data {
        anchor_id: String,
        #[serde(rename = "type")]
        anchor_type: Option<String>,
        stability: Option<String>,
        echo_cost: Option<f32>,
    }
    let data: Data = deserialize_component(&view)?;
    Ok(WeaveAnchorSpec {
        cell: coord,
        position: view.entity.position,
        anchor_id: data.anchor_id,
        anchor_type: data.anchor_type,
        stability: data.stability,
        echo_cost: data.echo_cost,
    })
}

fn parse_trigger_zone(view: CellComponentView<'_>, coord: GridCoord) -> Result<TriggerZoneSpec> {
    #[derive(Deserialize)]
    struct Data {
        trigger_id: String,
        shape: Option<String>,
        radius: Option<f32>,
        extents: Option<[f32; 3]>,
    }
    let data: Data = deserialize_component(&view)?;
    Ok(TriggerZoneSpec {
        cell: coord,
        position: view.entity.position,
        trigger_id: data.trigger_id,
        shape: data.shape,
        radius: data.radius,
        extents: data.extents,
    })
}

fn parse_decision_prompt(
    view: CellComponentView<'_>,
    coord: GridCoord,
) -> Result<DecisionPromptSpec> {
    #[derive(Deserialize)]
    struct Data {
        trigger_id: String,
        #[serde(default)]
        options: Vec<String>,
    }
    let data: Data = deserialize_component(&view)?;
    Ok(DecisionPromptSpec {
        cell: coord,
        position: view.entity.position,
        trigger_id: data.trigger_id,
        options: data.options,
    })
}

fn parse_encounter_trigger(
    view: CellComponentView<'_>,
    coord: GridCoord,
) -> Result<EncounterTriggerSpec> {
    #[derive(Deserialize)]
    struct Data {
        trigger_id: String,
        #[serde(default)]
        script: Option<String>,
    }
    let data: Data = deserialize_component(&view)?;
    Ok(EncounterTriggerSpec {
        cell: coord,
        position: view.entity.position,
        trigger_id: data.trigger_id,
        script: data.script,
    })
}

fn parse_encounter_complete(
    view: CellComponentView<'_>,
    coord: GridCoord,
) -> Result<EncounterCompleteSpec> {
    #[derive(Deserialize)]
    struct Data {
        trigger_id: String,
        #[serde(default)]
        next_cell: Option<[i32; 3]>,
    }
    let data: Data = deserialize_component(&view)?;
    Ok(EncounterCompleteSpec {
        cell: coord,
        position: view.entity.position,
        trigger_id: data.trigger_id,
        next_cell: data
            .next_cell
            .map(|c| GridCoord::new(c[0], c[1], c[2])),
    })
}

fn parse_effect_anchor(view: CellComponentView<'_>, coord: GridCoord) -> Result<EffectAnchorSpec> {
    #[derive(Deserialize)]
    struct Data {
        #[serde(default)]
        trigger_id: Option<String>,
        #[serde(default)]
        effect: Option<String>,
        #[serde(default)]
        intensity: Option<f32>,
    }
    let data: Data = deserialize_component(&view)?;
    Ok(EffectAnchorSpec {
        cell: coord,
        position: view.entity.position,
        trigger_id: data.trigger_id,
        effect: data.effect,
        intensity: data.intensity,
    })
}

fn parse_spawn_point(view: CellComponentView<'_>, coord: GridCoord) -> Result<SpawnPointSpec> {
    #[derive(Deserialize)]
    struct Data {
        id: String,
        #[serde(default)]
        facing: Option<[f32; 3]>,
    }
    let data: Data = deserialize_component(&view)?;
    Ok(SpawnPointSpec {
        cell: coord,
        position: view.entity.position,
        spawn_id: data.id,
        facing: data.facing,
    })
}

fn deserialize_component<T>(view: &CellComponentView<'_>) -> Result<T>
where
    T: DeserializeOwned,
{
    serde_json::from_str(&view.component.data).context(format!(
        "Failed to parse component '{}' JSON",
        view.component.component_type
    ))
}

