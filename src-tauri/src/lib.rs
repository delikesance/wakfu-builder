use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use wakfu_builder::models::Equipment;
use wakfu_builder::optimizer::{Optimizer, BuildProfile, Role, Mode, Range, Element};
use wakfu_builder::scraper::Scraper;

// ---------------------------------------------------------------------------
// Command 1: fetch_equipment
// ---------------------------------------------------------------------------
#[tauri::command]
async fn fetch_equipment() -> Result<Vec<Equipment>, String> {
    let scraper = Scraper::new().map_err(|e| e.to_string())?;
    let data_dir = wakfu_builder::scraper::ensure_data_dir().map_err(|e| e.to_string())?;
    let cache_path = data_dir.join("equipment.json");

    if cache_path.exists() {
        let file = std::fs::File::open(&cache_path).map_err(|e| e.to_string())?;
        let reader = std::io::BufReader::new(file);
        serde_json::from_reader(reader).map_err(|e| e.to_string())
    } else {
        let equipment = scraper.fetch_all_equipment().await.map_err(|e| e.to_string())?;
        scraper.save_cache(&equipment, "equipment.json").map_err(|e| e.to_string())?;
        Ok(equipment)
    }
}

// ---------------------------------------------------------------------------
// Command 2: optimize_build
// ---------------------------------------------------------------------------
#[derive(Deserialize)]
pub struct OptimizeRequest {
    level: i32,
    role: String,
    mode: String,
    range: String,
    element: String,
    min_ap: Option<i32>,
    min_mp: Option<i32>,
    min_res: Option<f32>,
}

#[derive(Serialize)]
pub struct BuildResult {
    items: Vec<ItemDisplay>,
    stats: HashMap<i32, f32>,
}

#[derive(Serialize)]
pub struct ItemDisplay {
    id: i32,
    name: String,
    level: i32,
    rarity: i32,
    slot_type: i32,
    slot_name: String,
    equipment_type: String,
    enchant_stat: Option<i32>,
    enchant_name: Option<String>,
    enchant_doubled: bool,
}

#[tauri::command]
async fn optimize_build(request: OptimizeRequest, items: Vec<Equipment>) -> Result<BuildResult, String> {
    let role = match request.role.as_str() {
        "dps" => Role::DPS,
        "tank" => Role::Tank,
        "support" => Role::Support,
        _ => return Err("Invalid role".into()),
    };
    let mode = match request.mode.as_str() {
        "solo" => Mode::Solo,
        "team" => Mode::Team,
        _ => return Err("Invalid mode".into()),
    };
    let range = match request.range.as_str() {
        "melee" => Range::Melee,
        "distance" => Range::Distance,
        "hybrid" => Range::Hybrid,
        _ => return Err("Invalid range".into()),
    };
    let element = match request.element.as_str() {
        "fire" => Element::Fire,
        "earth" => Element::Earth,
        "water" => Element::Water,
        "air" => Element::Air,
        "all" => Element::All,
        _ => return Err("Invalid element".into()),
    };

    let profile = BuildProfile::new_with_constraints(
        role, mode, range, element,
        request.min_ap.unwrap_or(10),
        request.min_mp.unwrap_or(4),
        request.min_res.unwrap_or(0.0),
    );

    let opt = Optimizer::new(items);
    let final_items = opt.find_perfect_build(request.level, &profile);
    let stats = opt.aggregate_stats(&final_items, &profile);

    let slot_names: HashMap<i32, &str> = [
        (134, "Coiffe"), (120, "Amulette"), (138, "Épaulières"), (132, "Cape"),
        (136, "Plastron"), (133, "Ceinture"), (103, "Anneau"), (119, "Bottes"),
        (519, "Arme 2H"), (518, "Arme 1H"), (112, "Dague"), (189, "Bouclier"),
        (582, "Familier"), (646, "Emblème"),
    ].into_iter().collect();

    let enchant_names: HashMap<i32, &str> = [
        (1052, "Mêlée"), (1053, "Distance"), (120, "Élém."), (20, "Vie"),
        (80, "Rési"), (180, "Dos"), (149, "Crit M."), (1054, "Zone"),
        (26, "Soin"), (1055, "Berserk"), (171, "Init."), (173, "Tacle"),
        (175, "Esquive"),
    ].into_iter().collect();

    let display_items: Vec<ItemDisplay> = final_items.iter().map(|item| {
        let potential = opt.get_socket_potential(item, &profile);
        let (enchant_stat, enchant_name, enchant_doubled) = if let Some((stat_id, _)) = potential.first() {
            let name = enchant_names.get(stat_id).copied().unwrap_or("Stat");
            let doubled = opt.is_stat_doubled_on_slot(*stat_id, item.id_type);
            (Some(*stat_id), Some(name.to_string()), doubled)
        } else {
            (None, None, false)
        };

        ItemDisplay {
            id: item.id,
            name: item.name.clone(),
            level: item.level,
            rarity: item.id_rarity,
            slot_type: item.id_type,
            slot_name: slot_names.get(&item.id_type).unwrap_or(&"Autre").to_string(),
            equipment_type: item.equipment_type.clone(),
            enchant_stat,
            enchant_name,
            enchant_doubled,
        }
    }).collect();

    Ok(BuildResult {
        items: display_items,
        stats,
    })
}

// ---------------------------------------------------------------------------
// Command 3: get_stats_info
// ---------------------------------------------------------------------------
#[derive(Serialize)]
pub struct StatInfoEntry {
    id: i32,
    name: &'static str,
    is_percentage: bool,
}

#[tauri::command]
async fn get_stats_info() -> Vec<StatInfoEntry> {
    vec![
        StatInfoEntry { id: 20,  name: "Vie",              is_percentage: false },
        StatInfoEntry { id: 31,  name: "PA",               is_percentage: false },
        StatInfoEntry { id: 41,  name: "PM",               is_percentage: false },
        StatInfoEntry { id: 80,  name: "Résistance",        is_percentage: false },
        StatInfoEntry { id: 120, name: "Maîtrise Élémentaire", is_percentage: false },
        StatInfoEntry { id: 122, name: "Maîtrise Feu",      is_percentage: false },
        StatInfoEntry { id: 123, name: "Maîtrise Terre",    is_percentage: false },
        StatInfoEntry { id: 124, name: "Maîtrise Eau",      is_percentage: false },
        StatInfoEntry { id: 125, name: "Maîtrise Air",      is_percentage: false },
        StatInfoEntry { id: 149, name: "Critique (Maîtrise)", is_percentage: false },
        StatInfoEntry { id: 150, name: "Taux Critique",     is_percentage: true },
        StatInfoEntry { id: 160, name: "Portée",            is_percentage: false },
        StatInfoEntry { id: 171, name: "Initiative",        is_percentage: false },
        StatInfoEntry { id: 173, name: "Tacle",             is_percentage: false },
        StatInfoEntry { id: 175, name: "Esquive",           is_percentage: false },
        StatInfoEntry { id: 180, name: "Dégâts Dos",        is_percentage: false },
        StatInfoEntry { id: 1052,name: "Maîtrise Mêlée",    is_percentage: false },
        StatInfoEntry { id: 1053,name: "Maîtrise Distance", is_percentage: false },
        StatInfoEntry { id: 1054,name: "Maîtrise Zone",     is_percentage: false },
        StatInfoEntry { id: 1055,name: "Maîtrise Berserk",  is_percentage: false },
        StatInfoEntry { id: 1068,name: "Maîtrise Élémentaire (total)", is_percentage: false },
        StatInfoEntry { id: 26,  name: "Soins",             is_percentage: false },
    ]
}

// ---------------------------------------------------------------------------
// App entry point
// ---------------------------------------------------------------------------
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            fetch_equipment,
            optimize_build,
            get_stats_info,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
