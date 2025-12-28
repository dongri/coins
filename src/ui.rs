use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        Axis, Block, Borders, Cell, Chart, Dataset, GraphType, Paragraph, Row, Table,
    },
    Frame,
};

use crate::app::App;

// Color scheme (matching the reference image)
const BORDER_COLOR: Color = Color::Rgb(70, 130, 180);
const HEADER_COLOR: Color = Color::Rgb(100, 200, 255);
const POSITIVE_COLOR: Color = Color::Rgb(0, 255, 127);
const NEGATIVE_COLOR: Color = Color::Rgb(255, 69, 100);
const SELECTED_BG: Color = Color::Rgb(30, 50, 70);
const TEXT_COLOR: Color = Color::Rgb(200, 200, 200);
const MUTED_COLOR: Color = Color::Rgb(120, 120, 120);
const CHART_COLOR: Color = Color::Rgb(100, 200, 255);

pub fn ui(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20), // Price Chart (top)
            Constraint::Min(10),        // Main content (coin list + info panels)
            Constraint::Length(3),      // Footer/Help
        ])
        .split(size);

    render_price_chart(frame, app, chunks[0]);
    render_main_content(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);
}

fn render_main_content(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Coin list
            Constraint::Percentage(50), // Info panels
        ])
        .split(area);

    render_coin_table(frame, app, chunks[0]);
    render_info_panel(frame, app, chunks[1]);
}

fn render_info_panel(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Price info
            Constraint::Length(12), // Changes info
            Constraint::Length(14), // Details info
            Constraint::Min(6),     // Supply info
        ])
        .split(area);

    render_price_info(frame, app, chunks[0]);
    render_changes_info(frame, app, chunks[1]);
    render_details_info(frame, app, chunks[2]);
    render_supply_info(frame, app, chunks[3]);
}

fn render_coin_table(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["#", "Coin", "Price (USD)", "1h %", "24h %", "7d %", "Market Cap"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(HEADER_COLOR)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1);

    let visible_height = area.height.saturating_sub(4) as usize;
    
    let rows = app.coins.iter().enumerate().skip(app.scroll_offset).take(visible_height).map(|(i, coin)| {
        let rank = coin.market_cap_rank.unwrap_or(0).to_string();
        let symbol = coin.symbol.to_uppercase();
        let price = format_price(coin.current_price.unwrap_or(0.0));
        let change_1h = format_percentage(coin.price_change_percentage_1h_in_currency);
        let change_24h = format_percentage(coin.price_change_percentage_24h_in_currency);
        let change_7d = format_percentage(coin.price_change_percentage_7d_in_currency);
        let market_cap = format_large_number(coin.market_cap.unwrap_or(0.0));

        let style = if i == app.selected_index {
            Style::default().bg(SELECTED_BG)
        } else {
            Style::default()
        };

        Row::new(vec![
            Cell::from(rank).style(Style::default().fg(MUTED_COLOR)),
            Cell::from(symbol).style(Style::default().fg(TEXT_COLOR)),
            Cell::from(price).style(Style::default().fg(TEXT_COLOR)),
            Cell::from(change_1h.0).style(Style::default().fg(change_1h.1)),
            Cell::from(change_24h.0).style(Style::default().fg(change_24h.1)),
            Cell::from(change_7d.0).style(Style::default().fg(change_7d.1)),
            Cell::from(market_cap).style(Style::default().fg(MUTED_COLOR)),
        ])
        .style(style)
        .height(1)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(8),
            Constraint::Length(14),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Min(12),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(format!(" Top 100 Coins by Market Cap ({}/{}) ", app.selected_index + 1, app.coins.len()))
            .title_style(Style::default().fg(HEADER_COLOR)),
    );

    frame.render_widget(table, area);
}

fn render_price_info(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(coin) = app.selected_coin() {
        let price = coin.current_price.unwrap_or(0.0);
        let high = coin.high_24h.unwrap_or(0.0);
        let low = coin.low_24h.unwrap_or(0.0);
        let change = coin.price_change_percentage_24h.unwrap_or(0.0);
        let change_color = if change >= 0.0 { POSITIVE_COLOR } else { NEGATIVE_COLOR };
        let arrow = if change >= 0.0 { "▲" } else { "▼" };

        vec![
            Line::from(vec![
                Span::styled(
                    format!("{} ", coin.symbol.to_uppercase()),
                    Style::default().fg(HEADER_COLOR).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &coin.name,
                    Style::default().fg(TEXT_COLOR),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    format!("${:.2} ", price),
                    Style::default().fg(TEXT_COLOR).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} {:.2}%", arrow, change.abs()),
                    Style::default().fg(change_color),
                ),
            ]),
            Line::from(vec![
                Span::styled("H: ", Style::default().fg(MUTED_COLOR)),
                Span::styled(format!("${:.2} ", high), Style::default().fg(POSITIVE_COLOR)),
                Span::styled("L: ", Style::default().fg(MUTED_COLOR)),
                Span::styled(format!("${:.2}", low), Style::default().fg(NEGATIVE_COLOR)),
            ]),
        ]
    } else {
        vec![Line::from("No coin selected")]
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_COLOR))
        .title(" Live Price (USD) ")
        .title_style(Style::default().fg(HEADER_COLOR));

    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, area);
}

fn render_changes_info(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(coin) = app.selected_coin() {
        let changes = [
            ("1H", coin.price_change_percentage_1h_in_currency),
            ("24H", coin.price_change_percentage_24h_in_currency),
            ("7D", coin.price_change_percentage_7d_in_currency),
            ("14D", coin.price_change_percentage_14d_in_currency),
            ("30D", coin.price_change_percentage_30d_in_currency),
            ("60D", coin.price_change_percentage_60d_in_currency),
            ("200D", coin.price_change_percentage_200d_in_currency),
            ("1Y", coin.price_change_percentage_1y_in_currency),
        ];

        changes
            .iter()
            .map(|(label, value)| {
                let (formatted, color) = format_percentage(*value);
                Line::from(vec![
                    Span::styled(format!("{:5} ", label), Style::default().fg(MUTED_COLOR)),
                    Span::styled(formatted, Style::default().fg(color)),
                ])
            })
            .collect::<Vec<_>>()
    } else {
        vec![Line::from("No coin selected")]
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_COLOR))
        .title(" Changes ")
        .title_style(Style::default().fg(HEADER_COLOR));

    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, area);
}

fn render_details_info(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(coin) = app.selected_coin() {
        let format_date = |date: &Option<String>| -> String {
            date.as_ref()
                .map(|d| d.chars().take(16).collect::<String>().replace("T", " "))
                .unwrap_or_else(|| "N/A".to_string())
        };

        vec![
            Line::from(vec![
                Span::styled("Name        ", Style::default().fg(MUTED_COLOR)),
                Span::styled(&coin.name, Style::default().fg(TEXT_COLOR)),
            ]),
            Line::from(vec![
                Span::styled("Symbol      ", Style::default().fg(MUTED_COLOR)),
                Span::styled(coin.symbol.to_uppercase(), Style::default().fg(TEXT_COLOR)),
            ]),
            Line::from(vec![
                Span::styled("Rank        ", Style::default().fg(MUTED_COLOR)),
                Span::styled(
                    coin.market_cap_rank.map(|r| r.to_string()).unwrap_or_else(|| "N/A".to_string()),
                    Style::default().fg(HEADER_COLOR),
                ),
            ]),
            Line::from(vec![
                Span::styled("MarketCap   ", Style::default().fg(MUTED_COLOR)),
                Span::styled(
                    format!("{} USD $", format_large_number(coin.market_cap.unwrap_or(0.0))),
                    Style::default().fg(TEXT_COLOR),
                ),
            ]),
            Line::from(vec![
                Span::styled("ATH         ", Style::default().fg(MUTED_COLOR)),
                Span::styled(
                    format!("{} USD $", format_large_number(coin.ath.unwrap_or(0.0))),
                    Style::default().fg(POSITIVE_COLOR),
                ),
            ]),
            Line::from(vec![
                Span::styled("ATHDate     ", Style::default().fg(MUTED_COLOR)),
                Span::styled(format_date(&coin.ath_date), Style::default().fg(MUTED_COLOR)),
            ]),
            Line::from(vec![
                Span::styled("ATL         ", Style::default().fg(MUTED_COLOR)),
                Span::styled(
                    format!("{} USD $", format_price(coin.atl.unwrap_or(0.0))),
                    Style::default().fg(NEGATIVE_COLOR),
                ),
            ]),
            Line::from(vec![
                Span::styled("ATLDate     ", Style::default().fg(MUTED_COLOR)),
                Span::styled(format_date(&coin.atl_date), Style::default().fg(MUTED_COLOR)),
            ]),
            Line::from(vec![
                Span::styled("TotalVolume ", Style::default().fg(MUTED_COLOR)),
                Span::styled(
                    format!("{} USD $", format_large_number(coin.total_volume.unwrap_or(0.0))),
                    Style::default().fg(TEXT_COLOR),
                ),
            ]),
            Line::from(vec![
                Span::styled("LastUpdate  ", Style::default().fg(MUTED_COLOR)),
                Span::styled(format_date(&coin.last_updated), Style::default().fg(MUTED_COLOR)),
            ]),
        ]
    } else {
        vec![Line::from("No coin selected")]
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_COLOR))
        .title(" Details ")
        .title_style(Style::default().fg(HEADER_COLOR));

    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, area);
}

fn render_supply_info(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(coin) = app.selected_coin() {
        let circulating = coin.circulating_supply.unwrap_or(0.0);
        let total = coin.total_supply.unwrap_or(0.0);
        let max = coin.max_supply;
        
        // Calculate supply ratio for progress bar
        let supply_ratio = if let Some(max_supply) = max {
            if max_supply > 0.0 {
                (circulating / max_supply * 100.0).min(100.0)
            } else {
                0.0
            }
        } else if total > 0.0 {
            (circulating / total * 100.0).min(100.0)
        } else {
            0.0
        };

        // Create visual progress bar
        let bar_width = 20;
        let filled = (supply_ratio / 100.0 * bar_width as f64) as usize;
        let bar = format!(
            "{}{}",
            "█".repeat(filled),
            "░".repeat(bar_width - filled)
        );

        vec![
            Line::from(vec![
                Span::styled("Circulating ", Style::default().fg(MUTED_COLOR)),
                Span::styled(format_large_number(circulating), Style::default().fg(POSITIVE_COLOR)),
            ]),
            Line::from(vec![
                Span::styled("Total       ", Style::default().fg(MUTED_COLOR)),
                Span::styled(format_large_number(total), Style::default().fg(TEXT_COLOR)),
            ]),
            Line::from(vec![
                Span::styled("Max Supply  ", Style::default().fg(MUTED_COLOR)),
                Span::styled(
                    max.map(format_large_number).unwrap_or_else(|| "∞ Unlimited".to_string()),
                    Style::default().fg(HEADER_COLOR),
                ),
            ]),
            Line::from(vec![
                Span::styled(bar, Style::default().fg(CHART_COLOR)),
                Span::styled(format!(" {:.1}%", supply_ratio), Style::default().fg(TEXT_COLOR)),
            ]),
        ]
    } else {
        vec![Line::from("No coin selected")]
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_COLOR))
        .title(" Supply ")
        .title_style(Style::default().fg(HEADER_COLOR));

    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, area);
}

fn render_price_chart(frame: &mut Frame, app: &App, area: Rect) {
    let timeframe_label = app.chart_timeframe.label();
    
    // Get the change percentage based on current timeframe
    let change_info = if let Some(coin) = app.selected_coin() {
        let change = match app.chart_timeframe {
            crate::app::ChartTimeframe::OneHour => coin.price_change_percentage_1h_in_currency,
            crate::app::ChartTimeframe::TwentyFourHours => coin.price_change_percentage_24h_in_currency,
            crate::app::ChartTimeframe::SevenDays => coin.price_change_percentage_7d_in_currency,
        };
        match change {
            Some(v) if v >= 0.0 => format!(" ▲ {:.2}%", v),
            Some(v) => format!(" ▼ {:.2}%", v.abs()),
            None => String::new(),
        }
    } else {
        String::new()
    };

    let chart_title = format!(" Price Chart ({}{}) [T to cycle] ", timeframe_label, change_info);
    
    if app.chart_data.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(chart_title)
            .title_style(Style::default().fg(HEADER_COLOR));
        
        let msg = if app.loading {
            "Loading chart data..."
        } else {
            "No chart data available"
        };
        
        let paragraph = Paragraph::new(msg)
            .style(Style::default().fg(MUTED_COLOR))
            .block(block);
        
        frame.render_widget(paragraph, area);
        return;
    }

    let data: Vec<(f64, f64)> = app.chart_data
        .iter()
        .enumerate()
        .map(|(i, &price)| (i as f64, price))
        .collect();

    let min_price = app.chart_data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_price = app.chart_data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let price_range = max_price - min_price;
    let padding = price_range * 0.1;

    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(CHART_COLOR))
        .data(&data)];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .title(chart_title)
                .title_style(Style::default().fg(HEADER_COLOR)),
        )
        .x_axis(
            Axis::default()
                .style(Style::default().fg(MUTED_COLOR))
                .bounds([0.0, data.len() as f64]),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(MUTED_COLOR))
                .bounds([min_price - padding, max_price + padding])
                .labels(vec![
                    Span::raw(format!("{:.0}", min_price)),
                    Span::raw(format!("{:.0}", max_price)),
                ]),
        );

    frame.render_widget(chart, area);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let update_status = if app.loading {
        "Loading...".to_string()
    } else if let Some(last) = app.last_update {
        let elapsed = last.elapsed().as_secs();
        format!("Updated {}s ago", elapsed)
    } else {
        "Not updated".to_string()
    };

    let help = vec![
        Line::from(vec![
            Span::styled(" ↑/k", Style::default().fg(HEADER_COLOR)),
            Span::styled(" Up  ", Style::default().fg(TEXT_COLOR)),
            Span::styled("↓/j", Style::default().fg(HEADER_COLOR)),
            Span::styled(" Down  ", Style::default().fg(TEXT_COLOR)),
            Span::styled("PgUp/PgDn", Style::default().fg(HEADER_COLOR)),
            Span::styled(" Page  ", Style::default().fg(TEXT_COLOR)),
            Span::styled("g/G", Style::default().fg(HEADER_COLOR)),
            Span::styled(" Top/Bottom  ", Style::default().fg(TEXT_COLOR)),
            Span::styled("T", Style::default().fg(HEADER_COLOR)),
            Span::styled(" Timeframe  ", Style::default().fg(TEXT_COLOR)),
            Span::styled("r", Style::default().fg(HEADER_COLOR)),
            Span::styled(" Refresh  ", Style::default().fg(TEXT_COLOR)),
            Span::styled("q", Style::default().fg(HEADER_COLOR)),
            Span::styled(" Quit  ", Style::default().fg(TEXT_COLOR)),
            Span::styled(&update_status, Style::default().fg(MUTED_COLOR)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_COLOR));

    let paragraph = Paragraph::new(help).block(block);
    frame.render_widget(paragraph, area);
}

fn format_price(price: f64) -> String {
    if price >= 1.0 {
        format!("{:.2}", price)
    } else if price >= 0.01 {
        format!("{:.4}", price)
    } else {
        format!("{:.8}", price)
    }
}

fn format_percentage(value: Option<f64>) -> (String, Color) {
    match value {
        Some(v) if v >= 0.0 => (format!("▲ {:.2}%", v), POSITIVE_COLOR),
        Some(v) => (format!("▼ {:.2}%", v.abs()), NEGATIVE_COLOR),
        None => ("N/A".to_string(), MUTED_COLOR),
    }
}

fn format_large_number(n: f64) -> String {
    if n >= 1_000_000_000_000.0 {
        format!("{:.2}T", n / 1_000_000_000_000.0)
    } else if n >= 1_000_000_000.0 {
        format!("{:.2}B", n / 1_000_000_000.0)
    } else if n >= 1_000_000.0 {
        format!("{:.2}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("{:.2}K", n / 1_000.0)
    } else {
        format!("{:.2}", n)
    }
}
