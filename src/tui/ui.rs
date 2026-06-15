use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::tui::app::{App, AppState};

pub fn render(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let block = Block::default()
        .title(" Wakfu Builder CLI ")
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
    let mut text = vec![Line::from(Span::styled("OPTIMAL BUILD FOUND:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)))];
    text.push(Line::from(""));

    for item in &app.best_build {
        text.push(Line::from(format!("- [Lvl {}] {} ({})", item.level, item.name, item.equipment_type)));
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
