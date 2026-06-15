mod models;
mod scraper;
mod optimizer;
mod tui;

use anyhow::Result;
use clap::Parser;
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tui::App;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target level for the build (1-245)
    #[arg(short, long)]
    level: Option<i32>,

    /// Character role (dps, tank, support)
    #[arg(short, long, value_enum)]
    role: Option<optimizer::Role>,

    /// Game mode (solo, team)
    #[arg(short, long, value_enum)]
    mode: Option<optimizer::Mode>,

    /// Combat range (melee, distance, hybrid)
    #[arg(short = 'a', long, value_enum)]
    range: Option<optimizer::Range>,

    /// Target element (fire, earth, water, air)
    #[arg(short, long, value_enum)]
    element: Option<optimizer::Element>,

    /// Minimum AP (Action Points) constraint
    #[arg(long)]
    min_ap: Option<i32>,

    /// Minimum MP (Movement Points) constraint
    #[arg(long)]
    min_mp: Option<i32>,

    /// Minimum Average Resistance constraint
    #[arg(long)]
    min_res: Option<f32>,

    /// If provided, run in non-interactive CLI mode instead of TUI
    #[arg(long)]
    cli: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Ensure data is cached
    let scraper = scraper::Scraper::new();
    if !std::path::Path::new("data/equipment.json").exists() {
        println!("No equipment cache found. Scraping from ZenithWakfu...");
        let equipment = scraper.fetch_all_equipment().await?;
        scraper.save_cache(&equipment, "equipment.json")?;
        println!("Saved {} items to cache.", equipment.len());
    }

    if args.cli || args.level.is_some() || args.role.is_some() || args.mode.is_some() || args.min_ap.is_some() {
        run_cli(args).await?;
    } else {
        run_tui().await?;
    }

    Ok(())
}

async fn run_cli(args: Args) -> Result<()> {
    let level = args.level.unwrap_or(200);
    let role = args.role.unwrap_or(optimizer::Role::DPS);
    let mode = args.mode.unwrap_or(optimizer::Mode::Solo);
    let range = args.range.unwrap_or(optimizer::Range::Hybrid);
    let element = args.element.unwrap_or(optimizer::Element::All);
    let min_ap = args.min_ap.unwrap_or(10);
    let min_mp = args.min_mp.unwrap_or(4);
    let min_res = args.min_res.unwrap_or(0.0);

    println!("--- Wakfu Builder CLI Mode ---");
    println!("Config: Level {}, Role {:?}, Mode {:?}, Range {:?}, Element {:?}", level, role, mode, range, element);
    println!("Constraints: >= {} AP | >= {} MP | >= {} Res", min_ap, min_mp, min_res);

    let file = std::fs::File::open("data/equipment.json")?;
    let reader = std::io::BufReader::new(file);
    let items: Vec<models::Equipment> = serde_json::from_reader(reader)?;

    let profile = optimizer::BuildProfile::new_with_constraints(role, mode, range, element, min_ap, min_mp, min_res);
    let opt = optimizer::Optimizer::new(items);

    let slot_names = [
        "Coiffe", "Amulette", "Épaulières", "Cape", "Plastron", "Ceinture", "Anneau 1", "Anneau 2", "Bottes", "Arme", "Familier", "Emblème"
    ];

    println!("Optimisation globale en cours...");
    let final_items = opt.find_perfect_build(level, &profile);

    println!("\nSTUFF OPTIMAL TROUVÉ (Recherche Globale) :");
    for item in &final_items {
        let rarity_tag = match item.id_rarity {
            7 => "[ÉPIQUE] ",
            5 => "[RELIQUE] ",
            _ => "",
        };
        
        let slot_name = match item.id_type {
            134 => "Coiffe", 120 => "Amulette", 138 => "Épaulières", 132 => "Cape",
            136 => "Plastron", 133 => "Ceinture", 103 => "Anneau", 119 => "Bottes",
            519 => "Arme 2H", 518 => "Arme 1H", 112 => "Dague", 189 => "Bouclier",
            582 => "Familier", 646 => "Emblème", _ => "Autre",
        };

        // Get enchantment recommendation
        let mut ench_str = String::new();
        if ![646, 112, 189, 582, 611].contains(&item.id_type) {
            let potential = opt.get_socket_potential(item, &profile);
            for (id, _) in potential {
                let name = match id {
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
                let doubled = if opt.is_stat_doubled_on_slot(id, item.id_type) { " (DOUBLÉ)" } else { "" };
                ench_str = format!("| 4x {}{}", name, doubled);
            }
        }

        println!("{:<12}: [Lvl {:>3}] {:<30} {}", slot_name, item.level, format!("{}{}", rarity_tag, item.name), ench_str);
    }

    // --- Stats Aggregation ---
    let total_stats = opt.aggregate_stats(&final_items, &profile);

    println!("\nSTATS TOTALES CUMULÉES (Inclus Enchantement Opti 4 Châsses) :");
    println!("{:<20}: {:>6}", "Points de Vie", total_stats.get(&20).unwrap_or(&0.0));
    
    let major_pa = if level >= 25 { 1.0 } else { 0.0 };
    println!("{:<20}: {:>6}", "PA", total_stats.get(&31).unwrap_or(&0.0) + 6.0 + major_pa); 
    println!("{:<20}: {:>6}", "PM", total_stats.get(&41).unwrap_or(&0.0) + 3.0); 
    
    let max_mastery = [
        total_stats.get(&122).cloned().unwrap_or(0.0),
        total_stats.get(&123).cloned().unwrap_or(0.0),
        total_stats.get(&124).cloned().unwrap_or(0.0),
        total_stats.get(&125).cloned().unwrap_or(0.0),
    ].into_iter().fold(0.0, f32::max);

    let melee_mastery = total_stats.get(&1052).unwrap_or(&0.0);

    println!("{:<20}: {:>6}", "Maîtrise Élém. Max", max_mastery);
    println!("{:<20}: {:>6}", "Maîtrise Mêlée", melee_mastery);
    println!("{:<20}: {:>6}", "PUISSANCE CONTACT", max_mastery + melee_mastery);
    println!("{:<20}: {:>6}", "Maîtrise Dos", total_stats.get(&180).unwrap_or(&0.0));
    println!("{:<20}: {:>6}%", "Coup Critique", total_stats.get(&150).unwrap_or(&0.0) + 3.0);
    println!("{:<20}: {:>6}", "Résistance Moyenne", total_stats.get(&80).unwrap_or(&0.0));

    Ok(())
}

async fn run_tui() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    // Run app
    let res = run_app_tui(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app_tui<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    // Load cached items into app initially
    if std::path::Path::new("data/equipment.json").exists() {
        let file = std::fs::File::open("data/equipment.json")?;
        let reader = std::io::BufReader::new(file);
        app.items = serde_json::from_reader(reader)?;
    }

    loop {
        terminal.draw(|f| tui::ui::render(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Up => app.prev_setting(),
                        KeyCode::Down => app.next_setting(),
                        KeyCode::Left => app.adjust_setting(-1),
                        KeyCode::Right => app.adjust_setting(1),
                        KeyCode::Char(' ') => {
                            if app.state == tui::app::AppState::Setup {
                                app.state = tui::app::AppState::Optimizing;
                                // Use default constraints for TUI
                                let profile = optimizer::BuildProfile::new_with_constraints(app.role, app.mode, app.range, app.element, 10, 4, 0.0);
                                let opt = optimizer::Optimizer::new(app.items.clone());
                                let final_items = opt.find_perfect_build(app.level, &profile);
                                app.best_build = final_items;
                                app.state = tui::app::AppState::Results;
                            } else {
                                app.state = tui::app::AppState::Setup;
                            }
                        }

                        _ => {}
                    }
                }
            }
        }
    }
}
