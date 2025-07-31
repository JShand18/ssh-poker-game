use crate::app::{App, AppState, FocusedComponent};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),      // Title
                Constraint::Min(10),        // Main content
                Constraint::Length(3),      // Status bar
            ]
            .as_ref(),
        )
        .split(f.size());

    // Draw title
    draw_title(f, chunks[0]);

    // Draw main content based on state
    match app.state {
        AppState::MainMenu => draw_main_menu(f, chunks[1], app),
        AppState::InGame => draw_game(f, chunks[1], app),
        AppState::Help => draw_help(f, chunks[1], app),
        AppState::GameOver => draw_game_over(f, chunks[1], app),
    }

    // Draw status bar
    draw_status_bar(f, chunks[2], app);
}

fn draw_title(f: &mut Frame, area: Rect) {
    let title = Paragraph::new("🎰 SSH Poker 🎰")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, area);
}

fn draw_main_menu(f: &mut Frame, area: Rect, app: &App) {
    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .map(|m| ListItem::new(m.as_str()))
        .collect();

    let messages_list = List::new(messages)
        .block(Block::default().borders(Borders::ALL).title("Main Menu"))
        .style(Style::default().fg(Color::White));

    f.render_widget(messages_list, area);
}

fn draw_help(f: &mut Frame, area: Rect, app: &App) {
    let help_text = app.messages.join("\n");
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: false });
    f.render_widget(help, area);
}

fn draw_game(f: &mut Frame, area: Rect, app: &App) {
    // Split game area into sections
    let game_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(70), // Left: Game board
                Constraint::Percentage(30), // Right: Actions/Chat
            ]
            .as_ref(),
        )
        .split(area);

    // Draw game board
    draw_game_board(f, game_chunks[0], app);

    // Split right panel
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(50), // Actions
                Constraint::Percentage(50), // Chat
            ]
            .as_ref(),
        )
        .split(game_chunks[1]);

    // Draw actions
    draw_actions(f, right_chunks[0], app);

    // Draw chat
    draw_chat(f, right_chunks[1], app);
}

fn draw_game_board(f: &mut Frame, area: Rect, app: &App) {
    let mut lines = vec![];

    if let Some(game) = &app.game {
        // Community cards
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("Community Cards: "),
            Span::styled(
                format_community_cards(game),
                Style::default().fg(Color::Yellow),
            ),
        ]));
        lines.push(Line::from(""));

        // Pot
        lines.push(Line::from(vec![
            Span::raw("Pot: "),
            Span::styled(
                format!("${}", game.pots[0].amount),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));

        // Players
        for (i, player) in game.players.iter().enumerate() {
            let is_current = i == game.current_player_index;
            let style = if is_current {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let player_line = format!(
                "{} {} - Chips: ${} - Bet: ${}",
                if is_current { "→" } else { " " },
                player.name,
                player.chips,
                player.current_bet
            );

            lines.push(Line::from(Span::styled(player_line, style)));

            // Show hole cards for player 0 (human player)
            if i == 0 && player.hole_cards.is_some() {
                let cards = player.hole_cards.unwrap();
                let cards_line = format!("  Your cards: {} {}", 
                    format_card(&cards[0]), 
                    format_card(&cards[1])
                );
                lines.push(Line::from(Span::styled(
                    cards_line,
                    Style::default().fg(Color::Magenta),
                )));
            }
        }
    }

    let game_board = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Game Board")
                .border_style(Style::default().fg(Color::White)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(game_board, area);
}

fn draw_actions(f: &mut Frame, area: Rect, app: &App) {
    let actions = app.get_available_actions();
    let items: Vec<ListItem> = actions
        .iter()
        .enumerate()
        .map(|(i, action)| {
            let style = if i == app.selected_action {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(action.as_str()).style(style)
        })
        .collect();

    let border_style = if app.focused_component == FocusedComponent::ActionList {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let actions_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Actions")
                .border_style(border_style),
        )
        .highlight_style(Style::default().bg(Color::Blue));

    f.render_widget(actions_list, area);
}

fn draw_chat(f: &mut Frame, area: Rect, app: &App) {
    let chat_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
        .split(area);

    // Messages
    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .rev()
        .take(10)
        .rev()
        .map(|m| ListItem::new(m.as_str()))
        .collect();

    let messages_list = List::new(messages)
        .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT));

    f.render_widget(messages_list, chat_chunks[0]);

    // Input
    let border_style = if app.focused_component == FocusedComponent::ChatInput {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Chat")
                .border_style(border_style),
        );

    f.render_widget(input, chat_chunks[1]);
}

fn draw_game_over(f: &mut Frame, area: Rect, app: &App) {
    let game_over = Paragraph::new("Game Over!")
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(game_over, area);
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let status = match app.state {
        AppState::MainMenu => "Main Menu | Press 'n' for new game | 'h' for help | 'q' to quit",
        AppState::InGame => "In Game | Tab: switch focus | Enter: select | 'q' to quit",
        AppState::Help => "Help | Press any key to return",
        AppState::GameOver => "Game Over | Press 'n' for new game | 'q' to quit",
    };

    let status_bar = Paragraph::new(status)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(status_bar, area);
}

fn format_community_cards(game: &poker_engine::GameState) -> String {
    if game.community_cards.is_empty() {
        "None yet".to_string()
    } else {
        game.community_cards
            .iter()
            .map(|c| format_card(c))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn format_card(card: &poker_engine::Card) -> String {
    format!("{}", card)
} 