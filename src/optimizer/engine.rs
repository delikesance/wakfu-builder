use crate::models::Equipment;
use std::collections::{HashMap, HashSet};
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum Role { Tank, DPS, Support }
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum Mode { Solo, Team }
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum Range { Melee, Distance, Hybrid }
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum Element { Fire, Earth, Water, Air, All }

#[derive(Debug, Clone, Default)]
pub struct StatVector {
    pub values: [f32; 22],
}

impl StatVector {
    pub fn get(&self, id: i32) -> f32 {
        let idx = self.id_to_idx(id);
        if idx < self.values.len() { self.values[idx] } else { 0.0 }
    }
    pub fn add_assign(&mut self, other: &StatVector) {
        for i in 0..22 { self.values[i] += other.values[i]; }
    }
    fn id_to_idx(&self, id: i32) -> usize {
        match id {
            20 => 0, 31 => 1, 41 => 2, 80 => 3, 120 => 4, 122 => 5, 123 => 6, 124 => 7, 125 => 8,
            1052 => 9, 1053 => 10, 180 => 11, 150 => 12, 160 => 13, 1068 => 14, 1051 => 15, 1054 => 16, 173 => 17, 1055 => 18, 171 => 19,
            149 => 20, 175 => 21, _ => 22,
        }
    }
}

pub struct BuildProfile {
    pub role: Role, pub mode: Mode, pub range: Range, pub element: Element,
    pub min_ap: i32, pub min_mp: i32, pub min_res: f32,
    pub weights: HashMap<i32, f32>,
}

impl BuildProfile {
    pub fn new_with_constraints(role: Role, mode: Mode, range: Range, element: Element, min_ap: i32, min_mp: i32, min_res: f32) -> Self {
        let mut weights = HashMap::new();
        match (&role, &mode) {
            (Role::DPS, mode) => {
                let s_w = if *mode == Mode::Solo { 1.5 } else { 0.0 };
                weights.insert(20, 0.75 * s_w); weights.insert(80, 3.0 * s_w);
                weights.insert(120, 1.0); weights.insert(1068, 1.2); weights.insert(149, 1.0); weights.insert(150, 10.0);
                weights.insert(1051, 1.8); weights.insert(1054, 1.8);
                match range {
                    Range::Melee => { weights.insert(1052, 2.5); weights.insert(180, 2.0); weights.insert(1053, -1.0); weights.insert(173, 1.5); }
                    Range::Distance => { weights.insert(1053, 2.0); weights.insert(160, 8.0); weights.insert(1052, -1.0); }
                    Range::Hybrid => { weights.insert(1052, 1.0); weights.insert(1053, 1.0); weights.insert(180, 1.0); }
                }
            },
            _ => { weights.insert(20, 1.0); weights.insert(120, 1.0); weights.insert(80, 1.0); }
        }
        Self { role, mode, range, element, min_ap, min_mp, min_res, weights }
    }
}

pub struct Optimizer { pub items: Vec<Equipment> }

impl Optimizer {
    pub fn new(items: Vec<Equipment>) -> Self { Self { items } }

    pub fn is_stat_doubled_on_slot(&self, stat_id: i32, slot_type: i32) -> bool {
        match stat_id {
            120 => [136, 132].contains(&slot_type), 1052 => [134, 132].contains(&slot_type),
            1053 => [133, 518, 519].contains(&slot_type), 1055 => [120, 132].contains(&slot_type),
            149 => [138, 518, 519].contains(&slot_type), 180 => [133, 119].contains(&slot_type),
            173 => slot_type == 103, 175 => slot_type == 103, 171 => [120, 132].contains(&slot_type),
            82 => [136, 133].contains(&slot_type), 83 => [138, 136].contains(&slot_type),
            84 => [136, 119].contains(&slot_type), 85 => [136, 132].contains(&slot_type),
            20 => [518, 519, 134].contains(&slot_type), 26 => [138, 120].contains(&slot_type),
            _ => false,
        }
    }

    fn get_enchantment_max_level(&self, item_level: i32) -> i32 {
        if item_level >= 216 { 11 } else if item_level >= 186 { 10 } else if item_level >= 171 { 9 }
        else if item_level >= 141 { 8 } else if item_level >= 126 { 7 } else if item_level >= 96 { 6 }
        else if item_level >= 81 { 5 } else if item_level >= 66 { 4 } else if item_level >= 51 { 3 }
        else if item_level >= 36 { 2 } else { 1 }
    }

    fn get_enchantment_value(&self, stat_id: i32, ench_level: i32) -> f32 {
        match stat_id {
            20 => (ench_level as f32 * 8.0).min(88.0),
            80 | 82 | 83 | 84 | 85 => (ench_level as f32 * 2.5).min(27.0),
            120 | 122 | 123 | 124 | 125 | 1052 | 1053 | 1051 | 1054 | 149 | 180 | 26 | 1055 => (ench_level as f32 * 3.0).min(33.0),
            150 => (ench_level as f32 * 1.0).min(11.0),
            171 | 173 | 175 => (ench_level as f32 * 4.0).min(44.0),
            _ => 0.0,
        }
    }

    /// Global Enchantment Solver
    /// Given a build, finds the absolute best allocation of 40 sockets to maximize the total build score.
    /// Considers ALL sockets jointly (global), not per-item independently.
    pub fn optimize_global_enchantment(&self, build: &[Equipment], profile: &BuildProfile) -> (StatVector, Vec<(i32, i32)>) {
        let mut base_v = StatVector::default();
        for item in build {
            base_v.add_assign(&self.get_item_base_vector(item, profile));
        }

        let enchantment_stats = [120, 1052, 1053, 1055, 149, 180, 173, 175, 171, 82, 83, 84, 85, 20, 26];
        let mut best_total_gain = 0.0_f32;
        let mut best_alloc: Vec<(i32, i32)> = Vec::new();

        for &stat in &enchantment_stats {
            let mut gains: Vec<(f32, i32)> = Vec::new();
            for item in build {
                if [646, 112, 189, 582, 611].contains(&item.id_type) { continue; }
                let elvl = self.get_enchantment_max_level(item.level);
                let mut val = self.get_enchantment_value(stat, elvl) * 4.0;
                if self.is_stat_doubled_on_slot(stat, item.id_type) { val *= 2.0; }
                let gain = val * profile.weights.get(&stat).cloned().unwrap_or(0.1);
                gains.push((gain, item.id));
            }
            gains.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            let total_gain: f32 = gains.iter().map(|(g, _)| g).sum();
            if total_gain > best_total_gain {
                best_total_gain = total_gain;
                best_alloc = gains.into_iter().map(|(_, item_id)| (item_id, stat)).collect();
            }
        }

        // Build final StatVector from best global allocation
        let mut final_ench_v = base_v;
        for (item_id, stat_id) in &best_alloc {
            if let Some(item) = build.iter().find(|i| i.id == *item_id) {
                let elvl = self.get_enchantment_max_level(item.level);
                let mut val = self.get_enchantment_value(*stat_id, elvl) * 4.0;
                if self.is_stat_doubled_on_slot(*stat_id, item.id_type) { val *= 2.0; }
                let idx = final_ench_v.id_to_idx(*stat_id);
                if idx < final_ench_v.values.len() { final_ench_v.values[idx] += val; }
            }
        }

        (final_ench_v, best_alloc)
    }

    fn get_item_base_vector(&self, item: &Equipment, _profile: &BuildProfile) -> StatVector {
        let mut v = StatVector::default();
        for e in &item.effects {
            for val in &e.values {
                if let Some(s) = &val.statistics {
                    let id = s.id_stats;
                    let idx = v.id_to_idx(id);
                    if id == 1068 {
                        for i in 5..=8 { v.values[i] += val.damage; }
                    } else if idx < v.values.len() {
                        v.values[idx] += val.damage;
                    }
                }
            }
        }
        v
    }

    fn calculate_score(&self, v: &StatVector, profile: &BuildProfile) -> f32 {
        let peak = match profile.element {
            Element::Fire => v.values[5], Element::Earth => v.values[6], Element::Water => v.values[7], Element::Air => v.values[8],
            Element::All => v.values[5].max(v.values[6]).max(v.values[7]).max(v.values[8]),
        };
        let mut s = peak * 2.0;
        for (&id, &weight) in &profile.weights { s += v.get(id) * weight; }
        s
    }

    pub fn find_perfect_build(&self, level: i32, profile: &BuildProfile) -> Vec<Equipment> {
        let mut candidates: HashMap<i32, Vec<(Equipment, StatVector)>> = HashMap::new();
        for item in &self.items {
            if item.level > level || item.level <= 1 || item.name.to_lowercase().contains("ancien") { continue; }
            if item.id_type == 646 && (item.name.to_lowercase().contains("modulation") || (item.name.to_lowercase().starts_with("embl") && item.name.split_whitespace().count() == 2)) { continue; }
            let v = self.get_item_base_vector(item, profile);
            candidates.entry(item.id_type).or_default().push((item.clone(), v));
        }

        let mut reduced: HashMap<i32, Vec<(Equipment, StatVector)>> = HashMap::new();
        let dims: Vec<i32> = vec![20, 31, 41, 80, 120, 122, 123, 124, 125, 1052, 1053, 180, 150, 149, 173, 175, 171, 1055, 1051, 1054, 26];

        for (t, items) in candidates {
            let mut pool = Vec::new();
            for (i_a, v_a) in &items {
                let mut dominated = false;
                for (i_b, v_b) in &items {
                    if i_a.id == i_b.id { continue; }
                    let mut b_better_or_equal = true; let mut b_strictly_better = false;
                    for &d in &dims {
                        let idx = v_a.id_to_idx(d);
                        if idx >= v_a.values.len() { continue; }
                        if v_b.values[idx] < v_a.values[idx] { b_better_or_equal = false; break; }
                        if v_b.values[idx] > v_a.values[idx] { b_strictly_better = true; }
                    }
                    if b_better_or_equal && b_strictly_better { dominated = true; break; }
                }
                if !dominated { pool.push((i_a.clone(), v_a.clone())); }
            }
            pool.sort_by(|a, b| self.calculate_score(&b.1, profile).partial_cmp(&self.calculate_score(&a.1, profile)).unwrap_or(std::cmp::Ordering::Equal));
            pool.truncate(4);
            reduced.insert(t, pool);
        }

        let setups = vec![
            vec![134, 120, 138, 132, 136, 133, 103, 103, 119, 582, 646, 519],
            vec![134, 120, 138, 132, 136, 133, 103, 103, 119, 582, 646, 518, 112],
            vec![134, 120, 138, 132, 136, 133, 103, 103, 119, 582, 646, 518, 189],
        ];

        let mut g_best_s = f32::MIN; let mut g_best_b = Vec::new();
        let mut max_p = HashMap::new();
        for &t in &[134, 120, 138, 132, 136, 133, 103, 119, 582, 646, 519, 518, 112, 189] {
            let mut mv = StatVector::default();
            if let Some(items) = reduced.get(&t) {
                for (_, v) in items { for d in 0..22 { mv.values[d] = mv.values[d].max(v.values[d]); } }
            }
            max_p.insert(t, mv);
        }

        for s in setups {
            let mut b_b = Vec::new(); let mut b_s = f32::MIN;
            let mut rem_pots = vec![(0.0, 0.0, 0.0); s.len() + 1];
            for i in (0..s.len()).rev() {
                let p = max_p.get(&s[i]).cloned().unwrap_or_default();
                let prev = rem_pots[i+1];
                rem_pots[i] = (prev.0 + self.calculate_score(&p, profile), prev.1 + p.values[1], prev.2 + p.values[2]);
            }
            let mut cur_v = StatVector::default();
            self.dfs(&s, 0, &reduced, &rem_pots, profile, &mut Vec::new(), &mut cur_v, &mut HashSet::new(), false, false, 0.0, &mut b_b, &mut b_s);
            if b_s > g_best_s { g_best_s = b_s; g_best_b = b_b; }
        }
        g_best_b
    }

    #[allow(clippy::too_many_arguments)]
    fn dfs(&self, slots: &[i32], depth: usize, cand: &HashMap<i32, Vec<(Equipment, StatVector)>>, rem_p: &[(f32, f32, f32)], prof: &BuildProfile,
           cur: &mut Vec<Equipment>, cur_v: &mut StatVector, used: &mut HashSet<i32>, has_e: bool, has_r: bool, cur_s: f32,
           best_b: &mut Vec<Equipment>, best_s: &mut f32) {
        if depth == slots.len() {
            // Final step: solve enchantment globally for this exact set
            let (total_v, _) = self.optimize_global_enchantment(cur, prof);
            let major_pa = if prof.min_ap > 6 { 1.0 } else { 0.0 };
            let total_ap = total_v.get(31) + 6.0 + major_pa;
            let total_mp = total_v.get(41) + 3.0;
            let total_res = total_v.get(80);
            
            let mut penalty = 0.0;
            if total_ap < prof.min_ap as f32 { penalty += (prof.min_ap as f32 - total_ap) * 10000.0; }
            if total_mp < prof.min_mp as f32 { penalty += (prof.min_mp as f32 - total_mp) * 10000.0; }
            if total_res < prof.min_res { penalty += (prof.min_res - total_res) * 100.0; }
            
            let final_score = self.calculate_score(&total_v, prof) - penalty;
            if final_score > *best_s { *best_s = final_score; *best_b = cur.clone(); }
            return;
        }

        let (_r_s, _r_ap, _r_mp) = rem_p[depth];
        // Pruning temporairement désactivé (heuristic broken, see G2 fix)
        let st = slots[depth]; let mut branched = false;
        if let Some(items) = cand.get(&st) {
            for (item, v) in items {
                if used.contains(&item.id) { continue; }
                let (ie, ir) = (item.id_rarity == 7, item.id_rarity == 5);
                if (ie && has_e) || (ir && has_r) { continue; }
                cur.push(item.clone()); used.insert(item.id); branched = true;
                let old_v = cur_v.clone(); cur_v.add_assign(v);
                self.dfs(slots, depth+1, cand, rem_p, prof, cur, cur_v, used, has_e || ie, has_r || ir, cur_s + self.calculate_score(v, prof), best_b, best_s);
                *cur_v = old_v; used.remove(&item.id); cur.pop();
            }
        }
        if !branched { self.dfs(slots, depth+1, cand, rem_p, prof, cur, cur_v, used, has_e, has_r, cur_s, best_b, best_s); }
    }

    pub fn get_socket_potential(&self, item: &Equipment, profile: &BuildProfile) -> Vec<(i32, f32)> {
        if [646, 112, 189, 582, 611].contains(&item.id_type) {
            return Vec::new();
        }
        let enchantment_stats = [120, 1052, 1053, 1055, 149, 180, 173, 175, 171, 20, 26];
        let elvl = self.get_enchantment_max_level(item.level);
        let mut results: Vec<(i32, f32)> = Vec::new();
        for &stat in &enchantment_stats {
            let mut val = self.get_enchantment_value(stat, elvl) * 4.0;
            if self.is_stat_doubled_on_slot(stat, item.id_type) { val *= 2.0; }
            let gain = val * profile.weights.get(&stat).cloned().unwrap_or(0.1);
            results.push((stat, gain));
        }
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(1);
        results
    }

    pub fn aggregate_stats(&self, build: &[Equipment], profile: &BuildProfile) -> HashMap<i32, f32> {
        let (v, _) = self.optimize_global_enchantment(build, profile);
        let mut totals = HashMap::new();
        let ids = [20, 31, 41, 80, 122, 123, 124, 125, 1052, 1053, 180, 150, 160, 171, 173, 175, 26, 1055];
        for id in ids { totals.insert(id, v.get(id)); }
        totals
    }
}
