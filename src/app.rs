use crate::api::CoinMarket;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartTimeframe {
    OneHour,
    TwentyFourHours,
    SevenDays,
}

impl ChartTimeframe {
    pub fn label(&self) -> &'static str {
        match self {
            ChartTimeframe::OneHour => "1H",
            ChartTimeframe::TwentyFourHours => "24H",
            ChartTimeframe::SevenDays => "7D",
        }
    }
}

pub struct App {
    pub coins: Vec<CoinMarket>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub chart_timeframe: ChartTimeframe,
    pub chart_data: Vec<f64>,
    pub last_update: Option<Instant>,
    pub loading: bool,
    pub error_message: Option<String>,
    pub should_quit: bool,
    pub vs_currency: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            coins: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            chart_timeframe: ChartTimeframe::TwentyFourHours,
            chart_data: Vec::new(),
            last_update: None,
            loading: true,
            error_message: None,
            should_quit: false,
            vs_currency: "usd".to_string(),
        }
    }

    pub fn selected_coin(&self) -> Option<&CoinMarket> {
        self.coins.get(self.selected_index)
    }

    pub fn move_selection_up(&mut self, visible_rows: usize) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.update_scroll_offset(visible_rows);
        }
    }

    pub fn move_selection_down(&mut self, visible_rows: usize) {
        if self.selected_index < self.coins.len().saturating_sub(1) {
            self.selected_index += 1;
            self.update_scroll_offset(visible_rows);
        }
    }

    pub fn page_up(&mut self, visible_rows: usize) {
        if self.selected_index >= visible_rows {
            self.selected_index -= visible_rows;
        } else {
            self.selected_index = 0;
        }
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
    }

    pub fn page_down(&mut self, visible_rows: usize) {
        let max_index = self.coins.len().saturating_sub(1);
        self.selected_index = (self.selected_index + visible_rows).min(max_index);
    }

    pub fn go_to_top(&mut self) {
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn go_to_bottom(&mut self) {
        self.selected_index = self.coins.len().saturating_sub(1);
    }

    pub fn cycle_timeframe(&mut self) {
        self.chart_timeframe = match self.chart_timeframe {
            ChartTimeframe::OneHour => ChartTimeframe::TwentyFourHours,
            ChartTimeframe::TwentyFourHours => ChartTimeframe::SevenDays,
            ChartTimeframe::SevenDays => ChartTimeframe::OneHour,
        };
    }

    pub fn update_chart_data(&mut self) {
        if let Some(coin) = self.selected_coin() {
            if let Some(sparkline) = &coin.sparkline_in_7d {
                let prices = &sparkline.price;
                let len = prices.len();
                
                self.chart_data = match self.chart_timeframe {
                    ChartTimeframe::OneHour => {
                        // Last ~4 data points (7 days / 168 hours * 1 hour)
                        let points = (len / 168).max(1);
                        prices.iter().rev().take(points * 4).rev().cloned().collect()
                    }
                    ChartTimeframe::TwentyFourHours => {
                        // Last ~24 data points
                        let points = (len / 7).max(1);
                        prices.iter().rev().take(points).rev().cloned().collect()
                    }
                    ChartTimeframe::SevenDays => {
                        // All data points
                        prices.clone()
                    }
                };
            } else {
                self.chart_data.clear();
            }
        }
    }

    pub fn update_scroll_offset(&mut self, visible_rows: usize) {
        if self.selected_index >= self.scroll_offset + visible_rows {
            self.scroll_offset = self.selected_index - visible_rows + 1;
        } else if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
