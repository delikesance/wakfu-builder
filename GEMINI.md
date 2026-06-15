# Wakfu Builder CLI

## Project Overview
A Rust-based CLI TUI for scraping ZenithWakfu equipment/spell data and generating optimal builds using mathematical combinatorial optimization.

## Tech Stack
- **Language:** Rust
- **TUI Framework:** Ratatui + Crossterm
- **HTTP Client:** Reqwest
- **Serialization:** SerDe (JSON)

## Architectural Guidelines
- **Math-First:** Optimization logic must be purely mathematical (weighted scoring) and not AI-based.
- **Cache-Heavy:** Scraped data must be cached locally to minimize API hits.
- **Surgical Scraping:** Follow the specific headers and pagination logic identified from browser requests.
- **State Management:** Use a clear state machine for the TUI (Setup -> Syncing -> Optimization -> Results).
- **Mathematical Reporting:** All responses must include a mathematical justification of changes, a proof of optimality where applicable, and a detailed technical report.

## Optimization Algorithm
- Weights are assigned based on Role (Tank, DPS, Support) and Mode (Solo, Team).
- Solo mode prioritizes balance across all survivability and offensive stats.
- Team mode allows specialized min-maxing for specific roles.
