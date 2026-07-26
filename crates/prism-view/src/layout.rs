//! Deterministic layout: same (snapshot, kind, params, node set) ⇒ same coordinates.

use crate::model::ViewNode;
use xxhash_rust::xxh3::xxh3_64;

pub fn layout_seed(snapshot_id: &str, view_kind: &str, params_key: &str) -> String {
    let h = xxh3_64(format!("{snapshot_id}|{view_kind}|{params_key}").as_bytes());
    format!("{h:016x}")
}

/// Layered / grid placement from sorted node ids (screenshot-diffable).
pub fn assign_coordinates(nodes: &mut [ViewNode], algorithm: &str, seed: &str) {
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    let seed_u = u64::from_str_radix(seed.get(..8).unwrap_or("0"), 16).unwrap_or(0);
    match algorithm {
        "radial" => {
            let n = nodes.len().max(1) as f64;
            for (i, node) in nodes.iter_mut().enumerate() {
                let angle = (i as f64) * std::f64::consts::TAU / n
                    + (seed_u % 360) as f64 * std::f64::consts::PI / 180.0;
                let r = 120.0 + (node.lod_rank as f64) * 40.0;
                node.x = (r * angle.cos() * 1000.0).round() / 1000.0;
                node.y = (r * angle.sin() * 1000.0).round() / 1000.0;
            }
        }
        "path" => {
            for (i, node) in nodes.iter_mut().enumerate() {
                node.x = (i as f64) * 140.0;
                node.y = ((seed_u % 7) as f64) * 10.0 + (node.lod_rank as f64) * 20.0;
            }
        }
        _ => {
            // layered: columns by group/lod, rows by sorted id
            let cols = ((nodes.len() as f64).sqrt().ceil() as usize).max(1);
            for (i, node) in nodes.iter_mut().enumerate() {
                let col = i % cols;
                let row = i / cols;
                node.x = (col as f64) * 160.0 + (seed_u % 11) as f64;
                node.y = (row as f64) * 90.0 + (node.lod_rank as f64) * 5.0;
            }
        }
    }
}
