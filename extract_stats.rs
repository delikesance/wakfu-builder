use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

fn main() -> anyhow::Result<()> {
    let file = File::open("data/equipment.json")?;
    let reader = BufReader::new(file);
    let items: Vec<Value> = serde_json::from_reader(reader)?;

    let mut stats_map: HashMap<i64, String> = HashMap::new();

    for item in items {
        if let Some(effects) = item.get("effects").and_then(|e| e.as_array()) {
            for effect in effects {
                if let Some(values) = effect.get("values").and_then(|v| v.as_array()) {
                    for val in values {
                        if let Some(stats) = val.get("statistics") {
                            if let (Some(id), Some(name)) = (
                                stats.get("id_stats").and_then(|i| i.as_i64()),
                                stats.get("name_stats").and_then(|n| n.as_str())
                            ) {
                                stats_map.insert(id, name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    let mut sorted_stats: Vec<_> = stats_map.into_iter().collect();
    sorted_stats.sort_by_key(|s| s.0);

    for (id, name) in sorted_stats {
        println!("{}: {}", id, name);
    }

    Ok(())
}
