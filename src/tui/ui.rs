use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::optimizer::{BuildProfile, Optimizer};
use crate::tui::app::{App, AppState};

pub fn render(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let block = Block::default()
        .title(" Wakfu Builder ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(size);

    let header = Paragraph::new("ZenithWakfu Scraper & Optimizer")
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(header, chunks[0]);

    match app.state {
        AppState::Setup => render_setup(f, app, chunks[1]),
        AppState::Results => render_results(f, app, chunks[1]),
        _ => {
            let p = Paragraph::new("Calculating optimal build...")
                .alignment(Alignment::Center);
            f.render_widget(p, chunks[1]);
        }
    }

    let help_text = Paragraph::new(" [Q] Quit | [Space] Generate | [Up/Down] Select | [Left/Right] Adjust ")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help_text, chunks[2]);
}

fn render_setup(f: &mut Frame, app: &App, area: Rect) {
    let level_style = if app.selected_index == 0 { Style::default().fg(Color::Yellow).add_modifier(Modifier::REVERSED) } else { Style::default() };
    let role_style = if app.selected_index == 1 { Style::default().fg(Color::Yellow).add_modifier(Modifier::REVERSED) } else { Style::default() };
    let mode_style = if app.selected_index == 2 { Style::default().fg(Color::Yellow).add_modifier(Modifier::REVERSED) } else { Style::default() };
    let range_style = if app.selected_index == 3 { Style::default().fg(Color::Yellow).add_modifier(Modifier::REVERSED) } else { Style::default() };
    let element_style = if app.selected_index == 4 { Style::default().fg(Color::Yellow).add_modifier(Modifier::REVERSED) } else { Style::default() };

    let current_role = match app.role {
        crate::optimizer::Role::DPS => "DPS",
        crate::optimizer::Role::Tank => "Tank",
        crate::optimizer::Role::Support => "Support",
    };
    let current_mode = match app.mode {
        crate::optimizer::Mode::Solo => "Solo",
        crate::optimizer::Mode::Team => "Team",
    };
    let current_range = match app.range {
        crate::optimizer::Range::Melee => "Melee",
        crate::optimizer::Range::Distance => "Distance",
        crate::optimizer::Range::Hybrid => "Hybrid",
    };
    let current_element = match app.element {
        crate::optimizer::Element::Fire => "Feu",
        crate::optimizer::Element::Earth => "Terre",
        crate::optimizer::Element::Water => "Eau",
        crate::optimizer::Element::Air => "Air",
        crate::optimizer::Element::All => "Auto (Best)",
    };

    let text = vec![
        Line::from(vec![Span::raw("Target Level: "), Span::styled(format!(" {} ", app.level), level_style)]),
        Line::from(""),
        Line::from(vec![Span::raw("Role:         "), Span::styled(format!(" {} ", current_role), role_style)]),
        Line::from(""),
        Line::from(vec![Span::raw("Mode:         "), Span::styled(format!(" {} ", current_mode), mode_style)]),
        Line::from(""),
        Line::from(vec![Span::raw("Range:        "), Span::styled(format!(" {} ", current_range), range_style)]),
        Line::from(""),
        Line::from(vec![Span::raw("Element:      "), Span::styled(format!(" {} ", current_element), element_style)]),
    ];

    let p = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" Build Configuration "))
        .alignment(Alignment::Center);

    // Center the paragraph in the area
    let area = centered_rect(60, 40, area);
    f.render_widget(p, area);
}

fn render_results(f: &mut Frame, app: &App, area: Rect) {
    let mut text = vec![
        Line::from(Span::styled(
            "OPTIMAL BUILD FOUND:",
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // Slot name mapping (same as main.rs)
    let slot_name = |id_type: i32| -> &'static str {
        match id_type {
            134 => "Coiffe",
            120 => "Amulette",
            138 => "Épaulières",
            132 => "Cape",
            136 => "Plastron",
            133 => "Ceinture",
            103 => "Anneau",
            119 => "Bottes",
            519 => "Arme 2H",
            518 => "Arme 1H",
            112 => "Dague",
            189 => "Bouclier",
            582 => "Familier",
            646 => "Emblème",
            _ => "Autre",
        }
    };

    // Build items display
    if app.best_build.is_empty() {
        text.push(Line::from(Span::styled(
            "No build generated yet.",
            Style::default().fg(Color::Red),
        )));
    } else {
        let profile = BuildProfile::new_with_constraints(
            app.role,
            app.mode,
            app.range,
            app.element,
            10,
            4,
            0.0,
        );
        let opt = Optimizer::new(app.items.clone());
        // Get enchantment recommendations via global solver
        let (_total_v, recs) = opt.optimize_global_enchantment(&app.best_build, &profile);
        // Build a map: item id -> recomended stat id
        let rec_map: std::collections::HashMap<i32, i32> = recs.into_iter().collect();

        for item in &app.best_build {
            let rarity_tag = match item.id_rarity {
                7 => "[ÉPIQUE] ",
                5 => "[RELIQUE] ",
                _ => "",
            };

            let slot = slot_name(item.id_type);

            // Enchantment string
            let ench_str = if let Some(&stat_id) = rec_map.get(&item.id) {
                if stat_id == 0 {
                    String::new()
                } else {
                    let name = match stat_id {
                        1052 => "Mêlée",
                        1053 => "Distance",
                        120 => "Élém.",
                        20 => "Vie",
                        80 => "Rési",
                        180 => "Dos",
                        149 => "Crit M.",
                        1054 => "Zone",
                        26 => "Soin",
                        1055 => "Berserk",
                        171 => "Init.",
                        173 => "Tacle",
                        175 => "Esquive",
                        _ => "Stat",
                    };
                    let doubled = if opt.is_stat_doubled_on_slot(stat_id, item.id_type) {
                        " (DOUBLÉ)"
                    } else {
                        ""
                    };
                    format!("| 4x {}{}", name, doubled)
                }
            } else {
                String::new()
            };

            let line = format!(
                "{:<12}: [Lvl {:>3}] {:<40} {}",
                slot,
                item.level,
                format!("{}{}", rarity_tag, item.name),
                ench_str,
            );
            text.push(Line::from(line));
        }

        // --- Stats Aggregation ---
        let total_stats = opt.aggregate_stats(&app.best_build, &profile);

        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            "STATS TOTALES CUMULÉES (Inclus Enchantement Opti 4 Châsses) :",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));

        let level = app.level;
        let major_pa = if level >= 25 { 1.0 } else { 0.0 };

        let lines = [
            ("Points de Vie", format!("{}", total_stats.get(&20).unwrap_or(&0.0))),
            ("PA", format!("{}", total_stats.get(&31).unwrap_or(&0.0) + 6.0 + major_pa)),
            ("PM", format!("{}", total_stats.get(&41).unwrap_or(&0.0) + 3.0)),
            ("Maîtrise Élém. Max", {
                let max_v = [122, 123, 124, 125]
                    .iter()
                    .map(|id| total_stats.get(id).copied().unwrap_or(0.0))
                    .fold(0.0_f32, f32::max);
                format!("{}", max_v)
            }),
            ("Maîtrise Mêlée", format!("{}", total_stats.get(&1052).unwrap_or(&0.0))),
            ("PUISSANCE CONTACT", {
                let melee = total_stats.get(&1052).copied().unwrap_or(0.0);
                let max_v = [122, 123, 124, 125]
                    .iter()
                    .map(|id| total_stats.get(id).copied().unwrap_or(0.0))
                    .fold(0.0_f32, f32::max);
                format!("{}", max_v + melee)
            }),
            ("Maîtrise Dos", format!("{}", total_stats.get(&180).unwrap_or(&0.0))),
            ("Coup Critique", format!("{}%", total_stats.get(&150).unwrap_or(&0.0) + 3.0)),
            ("Résistance Moyenne", format!("{}", total_stats.get(&80).unwrap_or(&0.0))),
        ];

        for (label, value) in &lines {
            text.push(Line::from(format!("{:<20}: {:>6}", label, value)));
        }
    }

    let p = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" Optimization Results "))
        .alignment(Alignment::Left);
    f.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
