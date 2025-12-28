mod api;
mod app;
mod ui;

use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use api::CoinGeckoClient;
use app::App;
use ui::ui;

const UPDATE_INTERVAL: Duration = Duration::from_secs(60); // 1 minute update interval
const TICK_RATE: Duration = Duration::from_millis(200);

fn fetch_data_blocking(client: &CoinGeckoClient, vs_currency: &str) -> Result<Vec<api::CoinMarket>, String> {
    // Create a new tokio runtime for this blocking call
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(async {
        client.get_markets(vs_currency).await.map_err(|e| e.to_string())
    })
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

fn print_version() {
    println!("{} {}", NAME, VERSION);
}

fn print_help() {
    println!("{} {} - Terminal-based cryptocurrency price tracker", NAME, VERSION);
    println!();
    println!("USAGE:");
    println!("    coins [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print help information");
    println!("    -V, --version    Print version information");
    println!();
    println!("CONTROLS:");
    println!("    ↑/k              Move selection up");
    println!("    ↓/j              Move selection down");
    println!("    PgUp/PgDn        Page up/down");
    println!("    g/G              Go to top/bottom");
    println!("    T                Cycle chart timeframe");
    println!("    r                Refresh data");
    println!("    q/Esc            Quit");
}

fn main() -> Result<()> {
    // Handle command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "-V" | "--version" => {
                print_version();
                return Ok(());
            }
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown option: {}", args[1]);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
    }

    // First, test the API before setting up TUI
    let client = CoinGeckoClient::new();
        
    let initial_coins = match fetch_data_blocking(&client, "usd") {
        Ok(coins) => {
            coins
        }
        Err(e) => {
            eprintln!("Warning: Could not fetch initial data: {}", e);
            Vec::new()
        }
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app with initial data
    let mut app = App::new();
    app.coins = initial_coins;
    if !app.coins.is_empty() {
        app.last_update = Some(Instant::now());
        app.loading = false;
        app.update_chart_data();
    }
    
    let res = run_app(&mut terminal, app, client);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    client: CoinGeckoClient,
) -> Result<()> {
    let mut last_tick = Instant::now();
    let mut last_fetch = Instant::now();

    loop {
        // Draw UI
        terminal.draw(|f| ui(f, &app))?;

        // Check if we need to fetch data (auto-refresh every minute)
        if !app.loading && last_fetch.elapsed() >= UPDATE_INTERVAL {
            app.loading = true;
            terminal.draw(|f| ui(f, &app))?;

            match fetch_data_blocking(&client, &app.vs_currency) {
                Ok(coins) => {
                    app.coins = coins;
                    app.last_update = Some(Instant::now());
                    app.error_message = None;
                    app.update_chart_data();
                }
                Err(e) => {
                    app.error_message = Some(format!("Failed to fetch data: {}", e));
                }
            }
            
            app.loading = false;
            last_fetch = Instant::now();
        }

        // Handle input with timeout
        let timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Calculate visible rows: total height - chart (20%) - footer (3) - table header (3) - borders
                let total_height = terminal.size()?.height as usize;
                let chart_height = total_height / 5; // 20%
                let visible_rows = total_height.saturating_sub(chart_height + 10);
                let prev_selected = app.selected_index;

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.should_quit = true;
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.should_quit = true;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.move_selection_up(visible_rows);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.move_selection_down(visible_rows);
                    }
                    KeyCode::PageUp => {
                        app.page_up(visible_rows);
                    }
                    KeyCode::PageDown => {
                        app.page_down(visible_rows);
                    }
                    KeyCode::Char('g') => {
                        app.go_to_top();
                    }
                    KeyCode::Char('G') => {
                        app.go_to_bottom();
                    }
                    KeyCode::Char('t') | KeyCode::Char('T') => {
                        app.cycle_timeframe();
                        app.update_chart_data();
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        if !app.loading {
                            app.loading = true;
                            terminal.draw(|f| ui(f, &app))?;
                            
                            match fetch_data_blocking(&client, &app.vs_currency) {
                                Ok(coins) => {
                                    app.coins = coins;
                                    app.last_update = Some(Instant::now());
                                    app.error_message = None;
                                    app.update_chart_data();
                                }
                                Err(e) => {
                                    app.error_message = Some(format!("Failed to fetch data: {}", e));
                                }
                            }
                            
                            app.loading = false;
                            last_fetch = Instant::now();
                        }
                    }
                    _ => {}
                }

                // Update chart data if selection changed
                if prev_selected != app.selected_index {
                    app.update_scroll_offset(visible_rows);
                    app.update_chart_data();
                }
            }
        }

        if last_tick.elapsed() >= TICK_RATE {
            last_tick = Instant::now();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
