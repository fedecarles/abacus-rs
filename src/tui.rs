//! This module provides the TUI interface for the ledger.

use crate::ledger::Ledger;
use crate::utils::read_ledger_files;
use chrono::Datelike;
use chrono::NaiveDate;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, TableState, Tabs},
    Frame, Terminal,
};
use std::io;
use std::str::FromStr;

#[derive(PartialEq, Clone)]
enum AppState {
    Balances,
    Accounts,
    Journal,
    FilterInput,
}

pub struct App {
    ledger: Ledger,
    state: AppState,
    balances_group: Option<String>,
    balances_price: Option<String>,
    balances_from: Option<String>,
    balances_to: Option<String>,
    journal_from: Option<String>,
    journal_to: Option<String>,
    filter_input_field: FilterField,
    filter_input_buffer: String,
    table_state: TableState,
}

#[derive(Clone)]
enum FilterField {
    BalancesFrom,
    BalancesTo,
    JournalFrom,
    JournalTo,
}

impl App {
    fn new(ledger: Ledger) -> Self {
        Self {
            ledger,
            state: AppState::Balances,
            balances_group: None,
            balances_price: None,
            balances_from: None,
            balances_to: None,
            journal_from: None,
            journal_to: None,
            filter_input_field: FilterField::BalancesFrom,
            filter_input_buffer: String::new(),
            table_state: TableState::default(),
        }
    }

    fn apply_filter(&mut self) {
        let date_from = if self.filter_input_buffer.is_empty() {
            None
        } else {
            NaiveDate::from_str(&self.filter_input_buffer).ok()
        };

        match self.filter_input_field {
            FilterField::BalancesFrom => {
                self.balances_from = date_from.map(|d| d.to_string());
            }
            FilterField::BalancesTo => {
                self.balances_to = date_from.map(|d| d.to_string());
            }
            FilterField::JournalFrom => {
                self.journal_from = date_from.map(|d| d.to_string());
            }
            FilterField::JournalTo => {
                self.journal_to = date_from.map(|d| d.to_string());
            }
        }
        self.filter_input_buffer.clear();
    }

    fn get_filter_prompt(&self) -> String {
        match self.filter_input_field {
            FilterField::BalancesFrom => "Enter from date (YYYY-MM-DD): ".to_string(),
            FilterField::BalancesTo => "Enter to date (YYYY-MM-DD): ".to_string(),
            FilterField::JournalFrom => "Enter from date (YYYY-MM-DD): ".to_string(),
            FilterField::JournalTo => "Enter to date (YYYY-MM-DD): ".to_string(),
        }
    }

    fn next(&mut self) {
        let len = self.get_current_list_len();
        if len == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => (i + 1) % len,
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn previous(&mut self) {
        let len = self.get_current_list_len();
        if len == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => (i + len - 1) % len,
            None => len - 1,
        };
        self.table_state.select(Some(i));
    }

    fn get_current_list_len(&self) -> usize {
        match self.state {
            AppState::Balances => self.get_balances_rows().len(),
            AppState::Accounts => self.ledger.accounts.len(),
            AppState::Journal => self
                .filter_journal_transactions(&self.ledger.transactions)
                .len(),
            AppState::FilterInput => 0,
        }
    }

    fn get_price(&self, commodity: &str) -> Option<f32> {
        let target_currency = self.balances_price.as_deref()?;

        self.ledger
            .prices
            .iter()
            .filter(|p| p.commodity == commodity && p.currency == target_currency)
            .max_by_key(|p| p.date)
            .map(|p| p.price)
    }

    fn get_balances_rows(&self) -> Vec<Row<'static>> {
        let group = self.balances_group.as_deref();
        let price_currency = self.balances_price.as_deref();

        if group.is_none() {
            let balances = self.calculate_total_balances();

            if balances.is_empty() {
                return vec![];
            }

            let mut rows: Vec<(String, String, String)> = Vec::new();

            for (account_name, amount) in balances {
                let account = self
                    .ledger
                    .accounts
                    .iter()
                    .find(|a| &a.name == &account_name);
                let account_type = account
                    .map(|a| a.account_type.to_string())
                    .unwrap_or_default();
                let currency = account
                    .map(|a| a.currency.clone())
                    .unwrap_or_else(|| "USD".to_string());

                let display_amount = if let Some(target) = price_currency {
                    if let Some(price) = self.get_price(&currency) {
                        format!("{:.2} {}", amount * price, target)
                    } else {
                        format!("{:.2} {}", amount, currency)
                    }
                } else {
                    format!("{:.2} {}", amount, currency)
                };

                rows.push((account_type.clone(), account_name.clone(), display_amount));
            }

            rows.sort_by(|a, b| {
                let type_cmp = a.0.cmp(&b.0);
                if type_cmp == std::cmp::Ordering::Equal {
                    a.1.cmp(&b.1)
                } else {
                    type_cmp
                }
            });

            return rows
                .into_iter()
                .map(|(t, n, v)| Row::new(vec![t, n, v]))
                .collect();
        }

        let (periods, balances_by_period) = self.calculate_balances_by_period(group.unwrap());

        if periods.is_empty() {
            return vec![];
        }

        let mut rows: Vec<(String, String, Vec<String>)> = Vec::new();

        for a in &self.ledger.accounts {
            let mut values: Vec<String> = Vec::new();
            let mut has_value = false;

            for period in &periods {
                let key = format!("{}|{}", period, a.name);
                let amount = balances_by_period.get(&key).copied().unwrap_or(0.0);

                let display_value = if let Some(target) = price_currency {
                    if let Some(price) = self.get_price(&a.currency) {
                        format!("{:.2} {}", amount * price, target)
                    } else {
                        format!("{:.2} {}", amount, a.currency)
                    }
                } else {
                    format!("{:.2}", amount)
                };

                values.push(display_value);
                if (amount - 0.0).abs() > f32::EPSILON {
                    has_value = true;
                }
            }

            if has_value {
                let mut row_values = vec![a.account_type.to_string(), a.name.clone()];
                row_values.extend(values);
                rows.push((a.account_type.to_string(), a.name.clone(), row_values));
            }
        }

        rows.sort_by(|a, b| {
            let type_cmp = a.0.cmp(&b.0);
            if type_cmp == std::cmp::Ordering::Equal {
                a.1.cmp(&b.1)
            } else {
                type_cmp
            }
        });

        rows.into_iter()
            .map(|(_, _, values)| Row::new(values))
            .collect()
    }

    fn calculate_balances_by_period(
        &self,
        group: &str,
    ) -> (Vec<String>, std::collections::HashMap<String, f32>) {
        use std::collections::HashMap;

        let mut balances: HashMap<String, f32> = HashMap::new();
        let mut periods: Vec<String> = Vec::new();

        let filtered_transactions = self.filter_transactions(&self.ledger.transactions);

        for t in filtered_transactions {
            let period_key = self.get_period_key(t.date, group);

            if !periods.contains(&period_key) {
                periods.push(period_key.clone());
            }

            let main_key = format!("{}|{}", period_key, t.account);
            let offset_key = format!("{}|{}", period_key, t.offset_account);

            *balances.entry(main_key).or_insert(0.0) += t.amount * t.quantity;
            *balances.entry(offset_key).or_insert(0.0) += t.offset_amount;
        }

        periods.sort();
        (periods, balances)
    }

    fn get_period_key(&self, date: chrono::NaiveDate, group: &str) -> String {
        let quarter = (date.month() - 1) / 3 + 1;
        match group {
            "M" => format!("{:04}-{:02}", date.year(), date.month()),
            "Q" => format!("{:04}-Q{}", date.year(), quarter),
            "Y" => format!("{:04}", date.year()),
            _ => "".to_string(),
        }
    }

    fn get_periods(&self) -> Vec<String> {
        let group = match self.balances_group.as_deref() {
            Some(g) => g,
            None => return vec![],
        };

        let mut periods: Vec<String> = Vec::new();

        let filtered_transactions = self.filter_transactions(&self.ledger.transactions);

        for t in filtered_transactions {
            let period = self.get_period_key(t.date, group);
            if !periods.contains(&period) {
                periods.push(period);
            }
        }

        periods.sort();
        periods
    }

    fn filter_transactions<'a>(
        &self,
        transactions: &'a [crate::transaction::Transaction],
    ) -> Vec<&'a crate::transaction::Transaction> {
        let from = self.balances_from.as_ref();
        let to = self.balances_to.as_ref();

        let from_date = from.and_then(|f| NaiveDate::from_str(f).ok());
        let to_date = to.and_then(|t| NaiveDate::from_str(t).ok());

        transactions
            .iter()
            .filter(|t| {
                let passes_from = from_date.map_or(true, |d| t.date >= d);
                let passes_to = to_date.map_or(true, |d| t.date <= d);
                passes_from && passes_to
            })
            .collect()
    }

    fn filter_journal_transactions<'a>(
        &self,
        transactions: &'a [crate::transaction::Transaction],
    ) -> Vec<&'a crate::transaction::Transaction> {
        let from = self.journal_from.as_ref();
        let to = self.journal_to.as_ref();

        let from_date = from.and_then(|f| NaiveDate::from_str(f).ok());
        let to_date = to.and_then(|t| NaiveDate::from_str(t).ok());

        transactions
            .iter()
            .filter(|t| {
                let passes_from = from_date.map_or(true, |d| t.date >= d);
                let passes_to = to_date.map_or(true, |d| t.date <= d);
                passes_from && passes_to
            })
            .collect()
    }

    fn calculate_total_balances(&self) -> std::collections::HashMap<String, f32> {
        use std::collections::HashMap;

        let mut balances: HashMap<String, f32> = HashMap::new();

        for a in &self.ledger.accounts {
            *balances.entry(a.name.clone()).or_insert(0.0) += a.opening_balance.unwrap_or_default();
        }

        let filtered_transactions = self.filter_transactions(&self.ledger.transactions);

        for t in filtered_transactions {
            *balances.entry(t.account.clone()).or_insert(0.0) += t.amount * t.quantity;
            *balances.entry(t.offset_account.clone()).or_insert(0.0) += t.offset_amount;
        }

        balances.retain(|_, &mut v| (v - 0.0).abs() > f32::EPSILON);
        balances
    }
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.state {
                    AppState::Balances => {
                        if key.code == KeyCode::Char('q') {
                            return Ok(());
                        } else if key.code == KeyCode::Char('a') {
                            app.state = AppState::Accounts;
                        } else if key.code == KeyCode::Char('j') {
                            app.state = AppState::Journal;
                        } else if key.code == KeyCode::Char('g') {
                            app.balances_group = match app.balances_group.as_deref() {
                                None => Some("M".to_string()),
                                Some("M") => Some("Q".to_string()),
                                Some("Q") => Some("Y".to_string()),
                                Some("Y") => None,
                                _ => None,
                            };
                        } else if key.code == KeyCode::Char('p') {
                            app.balances_price = if app.balances_price.is_some() {
                                None
                            } else {
                                Some("USD".to_string())
                            };
                        } else if key.code == KeyCode::Char('f') {
                            app.state = AppState::FilterInput;
                            app.filter_input_field = FilterField::BalancesFrom;
                            app.filter_input_buffer.clear();
                        } else if key.code == KeyCode::Char('t') {
                            app.state = AppState::FilterInput;
                            app.filter_input_field = FilterField::BalancesTo;
                            app.filter_input_buffer.clear();
                        } else if key.code == KeyCode::Char('c') {
                            app.balances_from = None;
                            app.balances_to = None;
                        } else if key.code == KeyCode::Down {
                            app.next();
                        } else if key.code == KeyCode::Up {
                            app.previous();
                        }
                    }
                    AppState::Accounts => {
                        if key.code == KeyCode::Char('q') {
                            return Ok(());
                        } else if key.code == KeyCode::Char('b') {
                            app.state = AppState::Balances;
                        } else if key.code == KeyCode::Char('j') {
                            app.state = AppState::Journal;
                        } else if key.code == KeyCode::Down {
                            app.next();
                        } else if key.code == KeyCode::Up {
                            app.previous();
                        }
                    }
                    AppState::Journal => {
                        if key.code == KeyCode::Char('q') {
                            return Ok(());
                        } else if key.code == KeyCode::Char('b') {
                            app.state = AppState::Balances;
                        } else if key.code == KeyCode::Char('a') {
                            app.state = AppState::Accounts;
                        } else if key.code == KeyCode::Char('f') {
                            app.state = AppState::FilterInput;
                            app.filter_input_field = FilterField::JournalFrom;
                            app.filter_input_buffer.clear();
                        } else if key.code == KeyCode::Char('t') {
                            app.state = AppState::FilterInput;
                            app.filter_input_field = FilterField::JournalTo;
                            app.filter_input_buffer.clear();
                        } else if key.code == KeyCode::Char('c') {
                            app.journal_from = None;
                            app.journal_to = None;
                        } else if key.code == KeyCode::Down {
                            app.next();
                        } else if key.code == KeyCode::Up {
                            app.previous();
                        }
                    }
                    AppState::FilterInput => {
                        if key.code == KeyCode::Char('q') {
                            return Ok(());
                        } else if key.code == KeyCode::Esc {
                            app.state = AppState::Balances;
                            app.filter_input_buffer.clear();
                        } else if key.code == KeyCode::Enter {
                            app.apply_filter();
                            if matches!(
                                app.filter_input_field,
                                FilterField::BalancesFrom | FilterField::BalancesTo
                            ) {
                                app.state = AppState::Balances;
                            } else {
                                app.state = AppState::Journal;
                            }
                        } else if let KeyCode::Char(c) = key.code {
                            app.filter_input_buffer.push(c);
                        } else if key.code == KeyCode::Backspace {
                            app.filter_input_buffer.pop();
                        }
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    let menu_items = vec![
        "Balances".to_string(),
        "Accounts".to_string(),
        "Journal".to_string(),
    ];

    let selected = match app.state {
        AppState::Balances | AppState::FilterInput => 0,
        AppState::Accounts => 1,
        AppState::Journal => 2,
    };

    let tabs = Tabs::new(menu_items)
        .block(Block::default().borders(Borders::ALL).title("Menu"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(selected);
    f.render_widget(tabs, chunks[0]);

    match app.state {
        AppState::Balances => {
            let group_label = match app.balances_group.as_ref() {
                Some(g) => match g.as_str() {
                    "M" => "Month",
                    "Q" => "Quarter",
                    "Y" => "Year",
                    _ => "None",
                },
                None => "None",
            };
            let price_label = app.balances_price.as_deref().unwrap_or("Off");
            let from_label = app.balances_from.as_deref().unwrap_or("-");
            let to_label = app.balances_to.as_deref().unwrap_or("-");
            let help_text = format!(
                " Group: [Y] Year [Q] Quarter [M] Month [{}] | Price: {} | From: {} | To: {} | (f) from | (t) to | (c) clear ",
                group_label, price_label, from_label, to_label
            );
            let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
            f.render_widget(help, chunks[1]);
        }
        AppState::Accounts => {
            let help = Paragraph::new(" View all accounts ".to_string())
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(help, chunks[1]);
        }
        AppState::Journal => {
            let from_label = app.journal_from.as_deref().unwrap_or("-");
            let to_label = app.journal_to.as_deref().unwrap_or("-");
            let help_text = format!(
                " From: {} | To: {} | (f) from | (t) to | (c) clear ",
                from_label, to_label
            );
            let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
            f.render_widget(help, chunks[1]);
        }
        AppState::FilterInput => {
            let prompt = format!("{}{}", app.get_filter_prompt(), app.filter_input_buffer);
            let help = Paragraph::new(prompt)
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().title("Filter").borders(Borders::ALL));
            f.render_widget(help, chunks[1]);
        }
    }

    match app.state {
        AppState::Balances => render_balances(f, app, chunks[2]),
        AppState::Accounts => render_accounts(f, app, chunks[2]),
        AppState::Journal => render_journal(f, app, chunks[2]),
        AppState::FilterInput => {
            let placeholder = Paragraph::new("Press Enter to apply, Esc to cancel")
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(placeholder, chunks[2]);
        }
    }

    let footer = Paragraph::new("(b)alances (a)ccounts (j)ournal (q)uit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[3]);
}

fn render_balances(f: &mut Frame, app: &mut App, area: Rect) {
    let balances_rows = app.get_balances_rows();
    let group = app.balances_group.as_deref();

    if group.is_none() {
        let table = Table::new(
            balances_rows,
            [
                Constraint::Length(12),
                Constraint::Min(30),
                Constraint::Length(15),
            ],
        )
        .header(
            Row::new(vec!["Type", "Account", "Amount"]).style(Style::default().fg(Color::Yellow)),
        )
        .block(Block::default().title("Balances").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));

        return f.render_stateful_widget(table, area, &mut app.table_state);
    }

    let periods = app.get_periods();
    let mut header = vec!["Type".to_string(), "Account".to_string()];
    header.extend(periods.clone());

    let widths: Vec<Constraint> = std::iter::once(Constraint::Length(12))
        .chain(std::iter::once(Constraint::Min(30)))
        .chain(periods.iter().map(|_| Constraint::Length(15)))
        .collect();

    let table = Table::new(balances_rows, widths)
        .header(
            Row::new(header.iter().map(|s| s.as_str()).collect::<Vec<&str>>())
                .style(Style::default().fg(Color::Yellow)),
        )
        .block(Block::default().title("Balances").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_accounts(f: &mut Frame, app: &mut App, area: Rect) {
    let mut accounts_list: Vec<ListItem> = Vec::new();

    for a in &app.ledger.accounts {
        let line = Line::from(vec![
            Span::raw(format!("{:12} ", a.open)),
            Span::raw(format!("{:12} ", a.account_type)),
            Span::raw(format!("{:30} ", a.name)),
            Span::raw(a.currency.clone()),
        ]);
        accounts_list.push(ListItem::new(line));
    }

    let list = List::new(accounts_list)
        .block(Block::default().title("Accounts").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(list, area);
}

fn render_journal(f: &mut Frame, app: &mut App, area: Rect) {
    let filtered_transactions = app.filter_journal_transactions(&app.ledger.transactions);

    let transactions: Vec<Row<'static>> = filtered_transactions
        .iter()
        .map(|t| {
            Row::new(vec![
                t.date.to_string(),
                t.account.clone(),
                format!("{:.2}", t.amount),
                t.payee.clone().unwrap_or_default(),
            ])
        })
        .collect();

    let table = Table::new(
        transactions,
        [
            Constraint::Length(12),
            Constraint::Length(30),
            Constraint::Length(12),
            Constraint::Min(20),
        ],
    )
    .header(
        Row::new(vec!["Date", "Account", "Amount", "Payee"])
            .style(Style::default().fg(Color::Yellow)),
    )
    .block(Block::default().title("Journal").borders(Borders::ALL))
    .style(Style::default().fg(Color::White))
    .highlight_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));

    f.render_stateful_widget(table, area, &mut app.table_state);
}

pub fn run_tui(ledger_path: &str) -> io::Result<()> {
    let ledger = read_ledger_files(ledger_path).expect("Failed to read ledger");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(ledger);
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}
