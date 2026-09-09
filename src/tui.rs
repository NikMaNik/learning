use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::models::{ActiveScreen, App, InputField};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Main layout: content + status bar
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // main content
            Constraint::Length(1), // status bar
        ])
        .split(area);

    match &app.current_screen {
        ActiveScreen::Main => render_main(frame, app, chunks[0]),
        ActiveScreen::Review => render_review(frame, app, chunks[0]),
        ActiveScreen::Forgot => render_forgot(frame, app, chunks[0]),
        ActiveScreen::Upcoming => render_upcoming(frame, app, chunks[0]),
        ActiveScreen::Add => render_add(frame, app, chunks[0]),
        ActiveScreen::Update => render_update(frame, app, chunks[0]),
        ActiveScreen::UpdateSearch => render_update_search(frame, app, chunks[0]),
        ActiveScreen::UpdateEdit => render_update_edit(frame, app, chunks[0]),
        ActiveScreen::Delete => render_delete(frame, app, chunks[0]),
        ActiveScreen::DeleteConfirm => render_delete_confirm(frame, app, chunks[0]),
        ActiveScreen::Import => render_import(frame, app, chunks[0]),
        ActiveScreen::ImportResult => render_import_result(frame, app, chunks[0]),
    }

    render_status_bar(frame, app, chunks[1]);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" LEARNING ENGLISH ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items = vec![
        ListItem::new(Line::from(vec![
            Span::styled(
                "  [1] ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "REVIEW",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("     — повторение слов по расписанию (SM-2)"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "  [2] ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "ADD",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("     — добавить новое слово (english → russian)"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "  [3] ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "UPDATE",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  — обновить существующее слово"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "  [4] ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "IMPORT",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  — импорт слов из .md файла"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "  [5] ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "DEL",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  — удалить слово из базы"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "  [6] ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "FORGOT",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — повторение забытых слов (quality < 3)"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "  [7] ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "UPCOMING",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — список слов на ближайшие N дней (слово + перевод)"),
        ])),
        ListItem::new(Line::from("")),
        ListItem::new(Line::from(vec![
            Span::styled(
                "  [q] ",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("EXIT", Style::default().fg(Color::DarkGray)),
            Span::raw("    — выход из приложения"),
        ])),
        ListItem::new(Line::from("")),
        ListItem::new(Line::from(vec![
            Span::styled("  Статистика: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{} слов", app.total_words()),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" │ "),
            Span::styled(
                format!("{} на повторении", app.words_for_review_count()),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" │ "),
            Span::styled(
                format!("{} забытых", app.forgotten_count()),
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" │ "),
            Span::styled(
                format!("{} на ближайшие {} дн.", app.upcoming_count(), app.upcoming_days),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ])),
    ];

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::NONE)
            .style(Style::default().fg(Color::White)),
    );
    frame.render_widget(list, inner);
}

fn render_review(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" REVIEW — Повторение слов ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(3), // word display
            Constraint::Length(3), // input
            Constraint::Length(3), // correct answer (if wrong)
            Constraint::Length(1), // result indicator
            Constraint::Min(1),    // spacer
            Constraint::Length(1), // help
        ])
        .split(inner);

    match app.current_review_word() {
        Some(word) => {
            // Show the word
            let word_block = Block::default()
                .title(" Слово для перевода ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));

            let word_text = Line::from(vec![Span::styled(
                format!("    {}", word.english),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]);

            let word_para = Paragraph::new(word_text)
                .block(word_block)
                .wrap(Wrap { trim: false });
            frame.render_widget(word_para, chunks[0]);

            // Input field
            let input_block = Block::default()
                .title(" Твой перевод ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(
                    if app.active_field == InputField::ReviewTranslation {
                        Color::Green
                    } else {
                        Color::DarkGray
                    },
                ));

            let input_text = Line::from(vec![
                Span::styled(
                    format!("    {}", app.input_review),
                    Style::default().fg(Color::White),
                ),
                Span::styled("█", Style::default().fg(Color::Green)),
            ]);

            let input_para = Paragraph::new(input_text).block(input_block);
            frame.render_widget(input_para, chunks[1]);

            // Show correct answer if wrong
            if app.show_review_answer {
                let answer_block = Block::default()
                    .title(" Правильный ответ ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green));

                let answer_text = Line::from(vec![Span::styled(
                    format!("    {}", word.russian),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )]);

                let answer_para = Paragraph::new(answer_text)
                    .block(answer_block)
                    .wrap(Wrap { trim: false });
                frame.render_widget(answer_para, chunks[2]);
            } else {
                // Empty space when not showing answer
                let empty = Paragraph::new("");
                frame.render_widget(empty, chunks[2]);
            }

            // Result indicator
            if app.show_review_answer {
                let is_correct = app.last_quality >= 3;
                let (icon, color) = if is_correct {
                    ("  \u{2713} Верно!", Color::Green)
                } else {
                    ("  \u{2717} Неверно", Color::Red)
                };

                let result_text = Line::from(vec![
                    Span::styled(
                        icon,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  Quality: "),
                    Span::styled(
                        format!("{}/5", app.last_quality),
                        Style::default()
                            .fg(if is_correct { Color::Green } else { Color::Red })
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  \u{2022}  Расстояние: "),
                    Span::styled(
                        format!("{}", app.last_distance),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw("  \u{2022}  Следующее повторение: "),
                    Span::styled(
                        format!("{} дн.", app.last_next_interval),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]);

                let result_para = Paragraph::new(result_text);
                frame.render_widget(result_para, chunks[3]);
            } else {
                let empty = Paragraph::new("");
                frame.render_widget(empty, chunks[3]);
            }

            // Help text
            let help = if app.show_review_answer {
                vec![Line::from(vec![
                    Span::styled(
                        "  Enter",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" — следующее слово  "),
                    Span::styled(
                        "Esc",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" — назад"),
                ])]
            } else {
                vec![Line::from(vec![
                    Span::styled(
                        "  Enter",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" — проверить  "),
                    Span::styled(
                        "Tab",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" — пропуст  "),
                    Span::styled(
                        "Esc",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" — назад"),
                ])]
            };
            let help_para = Paragraph::new(help).wrap(Wrap { trim: false });
            frame.render_widget(help_para, chunks[5]);
        }
        None => {
            let no_words = vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    "  Нет слов для повторения!",
                    Style::default().fg(Color::Yellow),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "  Все слова изучены или ещё не пора повторять.",
                    Style::default().fg(Color::DarkGray),
                )]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "  Esc",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" — назад в меню"),
                ]),
            ];
            let no_words_para = Paragraph::new(no_words).wrap(Wrap { trim: false });
            frame.render_widget(no_words_para, inner);
        }
    }
}

fn render_forgot(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" FORGOT — Повторение забытых слов ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(3), // word display
            Constraint::Length(3), // input
            Constraint::Length(3), // correct answer (if wrong)
            Constraint::Length(1), // result indicator
            Constraint::Min(1),    // spacer
            Constraint::Length(1), // help
        ])
        .split(inner);

    match app.forgotten_queue.get(app.current_word_index) {
        Some(word) => {
            let word_block = Block::default()
                .title(" Слово для перевода ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));

            let word_text = Line::from(vec![Span::styled(
                format!("    {}", word.english),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]);

            let word_para = Paragraph::new(word_text)
                .block(word_block)
                .wrap(Wrap { trim: false });
            frame.render_widget(word_para, chunks[0]);

            let input_block = Block::default()
                .title(" Твой перевод ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if app.active_field == InputField::ReviewTranslation {
                    Color::Green
                } else {
                    Color::DarkGray
                }));

            let input_text = Line::from(vec![
                Span::styled(
                    format!("    {}", app.input_review),
                    Style::default().fg(Color::White),
                ),
                Span::styled("█", Style::default().fg(Color::Green)),
            ]);

            let input_para = Paragraph::new(input_text).block(input_block);
            frame.render_widget(input_para, chunks[1]);

            if app.show_review_answer {
                let answer_block = Block::default()
                    .title(" Правильный ответ ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green));

                let answer_text = Line::from(vec![Span::styled(
                    format!("    {}", word.russian),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )]);

                let answer_para = Paragraph::new(answer_text)
                    .block(answer_block)
                    .wrap(Wrap { trim: false });
                frame.render_widget(answer_para, chunks[2]);
            } else {
                let empty = Paragraph::new("");
                frame.render_widget(empty, chunks[2]);
            }

            if app.show_review_answer {
                let is_correct = app.last_quality >= 3;
                let (icon, color) = if is_correct {
                    ("  \u{2713} Верно!", Color::Green)
                } else {
                    ("  \u{2717} Неверно", Color::Red)
                };

                let result_text = Line::from(vec![
                    Span::styled(
                        icon,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  Quality: "),
                    Span::styled(
                        format!("{}/5", app.last_quality),
                        Style::default()
                            .fg(if is_correct { Color::Green } else { Color::Red })
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  \u{2022}  Расстояние: "),
                    Span::styled(
                        format!("{}", app.last_distance),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw("  \u{2022}  Следующее повторение: "),
                    Span::styled(
                        format!("{} дн.", app.last_next_interval),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]);

                let result_para = Paragraph::new(result_text);
                frame.render_widget(result_para, chunks[3]);
            } else {
                let empty = Paragraph::new("");
                frame.render_widget(empty, chunks[3]);
            }

            let help = if app.show_review_answer {
                vec![Line::from(vec![
                    Span::styled(
                        "  Enter",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" — следующее слово  "),
                    Span::styled(
                        "Esc",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" — назад"),
                ])]
            } else {
                vec![Line::from(vec![
                    Span::styled(
                        "  Enter",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" — проверить  "),
                    Span::styled(
                        "Tab",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" — пропустить  "),
                    Span::styled(
                        "Esc",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" — назад"),
                ])]
            };
            let help_para = Paragraph::new(help).wrap(Wrap { trim: false });
            frame.render_widget(help_para, chunks[5]);
        }
        None => {
            let no_words = vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    "  Нет забытых слов!",
                    Style::default().fg(Color::Yellow),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "  Здесь появятся слова, которые ты не запомнил (quality < 3).",
                    Style::default().fg(Color::DarkGray),
                )]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "  Esc",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" — назад в меню"),
                ]),
            ];
            let no_words_para = Paragraph::new(no_words).wrap(Wrap { trim: false });
            frame.render_widget(no_words_para, inner);
        }
    }
}

fn render_upcoming(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!(
            " UPCOMING — Слова на ближайшие {} дн. ",
            app.upcoming_days
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(3), // days input
            Constraint::Min(5),    // word list
            Constraint::Length(3), // help
        ])
        .split(inner);

    let days_block = Block::default()
        .title(" Сколько дней вперёд (Enter — применить) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let days_text = Line::from(vec![
        Span::styled(
            format!("  {}", app.upcoming_input),
            Style::default().fg(Color::White),
        ),
        Span::styled("█", Style::default().fg(Color::Green)),
    ]);

    let days_para = Paragraph::new(days_text).block(days_block);
    frame.render_widget(days_para, chunks[0]);

    if app.upcoming_words.is_empty() {
        let no_words = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  Нет слов на ближайшие {} дн.", app.upcoming_days),
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Измени число дней или вернись позже.",
                Style::default().fg(Color::DarkGray),
            )]),
        ];
        let no_words_para = Paragraph::new(no_words).wrap(Wrap { trim: false });
        frame.render_widget(no_words_para, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .upcoming_words
            .iter()
            .enumerate()
            .map(|(i, word)| {
                let style = if i == app.selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("  {}. ", i + 1), style),
                    Span::styled(&word.english, style),
                    Span::styled(" — ", style),
                    Span::styled(&word.russian, style),
                ]))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(format!(" Слов: {} ", app.upcoming_words.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );
        frame.render_widget(list, chunks[1]);
    }

    let help = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  j/↓",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — вниз  "),
            Span::styled(
                "k/↑",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — вверх  "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — применить N дней  "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — назад"),
        ]),
    ];
    let help_para = Paragraph::new(help).wrap(Wrap { trim: false });
    frame.render_widget(help_para, chunks[2]);
}

fn render_add(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" ADD — Добавить слово ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(3), // english input
            Constraint::Length(1), // spacer
            Constraint::Length(3), // russian input
            Constraint::Min(3),    // help
        ])
        .split(inner);

    // English input
    let en_block = Block::default()
        .title(" English ")
        .borders(Borders::ALL)
        .border_style(
            Style::default().fg(if app.active_field == InputField::English {
                Color::Green
            } else {
                Color::DarkGray
            }),
        );

    let en_text = Line::from(vec![
        Span::styled(
            format!("  {}", app.input_english),
            Style::default().fg(Color::White),
        ),
        Span::styled("█", Style::default().fg(Color::Green)),
    ]);

    let en_para = Paragraph::new(en_text).block(en_block);
    frame.render_widget(en_para, chunks[0]);

    // Russian input
    let ru_block = Block::default()
        .title(" Russian ")
        .borders(Borders::ALL)
        .border_style(
            Style::default().fg(if app.active_field == InputField::Russian {
                Color::Green
            } else {
                Color::DarkGray
            }),
        );

    let ru_text = Line::from(vec![
        Span::styled(
            format!("  {}", app.input_russian),
            Style::default().fg(Color::White),
        ),
        Span::styled("█", Style::default().fg(Color::Green)),
    ]);

    let ru_para = Paragraph::new(ru_text).block(ru_block);
    frame.render_widget(ru_para, chunks[2]);

    // Help
    let help = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Tab",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — переключить поле  "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — сохранить  "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — назад"),
        ]),
    ];
    let help_para = Paragraph::new(help).wrap(Wrap { trim: false });
    frame.render_widget(help_para, chunks[3]);
}

fn render_update_search(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" UPDATE — Поиск слова ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(3), // search input
            Constraint::Min(3),    // help
        ])
        .split(inner);

    // Search input
    let search_block = Block::default()
        .title(" Введи слово для поиска (english или russian) ")
        .borders(Borders::ALL)
        .border_style(
            Style::default().fg(if app.active_field == InputField::ReviewTranslation {
                Color::Green
            } else {
                Color::DarkGray
            }),
        );

    let search_text = Line::from(vec![
        Span::styled(
            format!("  {}", app.search_query),
            Style::default().fg(Color::White),
        ),
        Span::styled("█", Style::default().fg(Color::Green)),
    ]);

    let search_para = Paragraph::new(search_text).block(search_block);
    frame.render_widget(search_para, chunks[0]);

    // Help
    let help = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Enter",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — поиск  "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — назад"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Если слово найдено — сразу редактирование.",
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(vec![Span::styled(
            "  Если нет — покажет похожие слова для выбора.",
            Style::default().fg(Color::DarkGray),
        )]),
    ];
    let help_para = Paragraph::new(help).wrap(Wrap { trim: false });
    frame.render_widget(help_para, chunks[1]);
}

fn render_update(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" UPDATE — Похожие слова ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // word list
            Constraint::Length(3), // help
        ])
        .split(inner);

    if app.search_results.is_empty() {
        let no_words = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Похожие слова не найдены.",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Esc",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" — назад к поиску"),
            ]),
        ];
        let no_words_para = Paragraph::new(no_words).wrap(Wrap { trim: false });
        frame.render_widget(no_words_para, inner);
        return;
    }

    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .enumerate()
        .map(|(i, word)| {
            let style = if i == app.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {}. ", i + 1), style),
                Span::styled(&word.english, style),
                Span::styled(" — ", style),
                Span::styled(&word.russian, style),
            ]))
        })
        .collect();

    let title = format!(
        " Найдено: {} слов(а) — выбери для редактирования ",
        app.search_results.len()
    );
    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );
    frame.render_widget(list, chunks[0]);

    let help = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  j/↓",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — вниз  "),
            Span::styled(
                "k/↑",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — вверх  "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — редактировать  "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — назад"),
        ]),
    ];
    let help_para = Paragraph::new(help).wrap(Wrap { trim: false });
    frame.render_widget(help_para, chunks[1]);
}

fn render_update_edit(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" UPDATE — Редактирование ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(3), // english input
            Constraint::Length(1), // spacer
            Constraint::Length(3), // russian input
            Constraint::Min(3),    // help
        ])
        .split(inner);

    // English input
    let en_block = Block::default()
        .title(" English ")
        .borders(Borders::ALL)
        .border_style(
            Style::default().fg(if app.active_field == InputField::English {
                Color::Green
            } else {
                Color::DarkGray
            }),
        );

    let en_text = Line::from(vec![
        Span::styled(
            format!("  {}", app.input_english),
            Style::default().fg(Color::White),
        ),
        Span::styled("█", Style::default().fg(Color::Green)),
    ]);

    let en_para = Paragraph::new(en_text).block(en_block);
    frame.render_widget(en_para, chunks[0]);

    // Russian input
    let ru_block = Block::default()
        .title(" Russian ")
        .borders(Borders::ALL)
        .border_style(
            Style::default().fg(if app.active_field == InputField::Russian {
                Color::Green
            } else {
                Color::DarkGray
            }),
        );

    let ru_text = Line::from(vec![
        Span::styled(
            format!("  {}", app.input_russian),
            Style::default().fg(Color::White),
        ),
        Span::styled("█", Style::default().fg(Color::Green)),
    ]);

    let ru_para = Paragraph::new(ru_text).block(ru_block);
    frame.render_widget(ru_para, chunks[2]);

    // Help
    let help = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Tab",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — переключить поле  "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — сохранить  "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — отмена"),
        ]),
    ];
    let help_para = Paragraph::new(help).wrap(Wrap { trim: false });
    frame.render_widget(help_para, chunks[3]);
}

fn render_delete(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" DELETE — Удалить слово ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // word list
            Constraint::Length(3), // help
        ])
        .split(inner);

    let items: Vec<ListItem> = app
        .words
        .iter()
        .enumerate()
        .map(|(i, word)| {
            let style = if i == app.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {}. ", i + 1), style),
                Span::styled(&word.english, style),
                Span::styled(" — ", style),
                Span::styled(&word.russian, style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Выбери слово для удаления ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
    );
    frame.render_widget(list, chunks[0]);

    let help = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  j/↓",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — вниз  "),
            Span::styled(
                "k/↑",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — вверх  "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — удалить  "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — назад"),
        ]),
    ];
    let help_para = Paragraph::new(help).wrap(Wrap { trim: false });
    frame.render_widget(help_para, chunks[1]);
}

fn render_delete_confirm(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" DELETE — Подтверждение ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(word) = app.words.get(app.selected) {
        let text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Ты точно хочешь удалить слово?",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(
                    &word.english,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" — ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &word.russian,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    "[Y] ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled("Да, удалить   ", Style::default().fg(Color::White)),
                Span::styled(
                    "[N] ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Нет, отмена", Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Esc",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" — отмена"),
            ]),
        ];

        let para = Paragraph::new(text).wrap(Wrap { trim: false });
        frame.render_widget(para, inner);
    }
}

fn render_import(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" IMPORT — Импорт из .md файла ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(3), // path input
            Constraint::Length(8), // format info
            Constraint::Min(3),    // help
        ])
        .split(inner);

    // Path input
    let path_block = Block::default()
        .title(" Путь к файлу ")
        .borders(Borders::ALL)
        .border_style(
            Style::default().fg(if app.active_field == InputField::Path {
                Color::Green
            } else {
                Color::DarkGray
            }),
        );

    let path_text = Line::from(vec![
        Span::styled(
            format!("  {}", app.input_path),
            Style::default().fg(Color::White),
        ),
        Span::styled("█", Style::default().fg(Color::Green)),
    ]);

    let path_para = Paragraph::new(path_text).block(path_block);
    frame.render_widget(path_para, chunks[0]);

    // Format info
    let info = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Поддерживаемые форматы:",
            Style::default().fg(Color::Yellow),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "    - hello — привет",
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(vec![Span::styled(
            "    | english | russian |",
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(vec![Span::styled(
            "    apple — яблоко",
            Style::default().fg(Color::DarkGray),
        )]),
    ];
    let info_para = Paragraph::new(info).wrap(Wrap { trim: false });
    frame.render_widget(info_para, chunks[1]);

    // Help
    let help = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Enter",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — импортировать  "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — назад"),
        ]),
    ];
    let help_para = Paragraph::new(help).wrap(Wrap { trim: false });
    frame.render_widget(help_para, chunks[2]);
}

fn render_import_result(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" IMPORT — Результат ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Импорт завершён!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Добавлено слов: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", app.import_count),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — назад в меню"),
        ]),
    ];

    let para = Paragraph::new(text).wrap(Wrap { trim: false });
    frame.render_widget(para, inner);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status_style = if app.status_is_error {
        Style::default().fg(Color::Red)
    } else if app.status_message.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Green)
    };

    let status_text = if app.status_message.is_empty() {
        Line::from(vec![
            Span::styled("  Learning English", Style::default().fg(Color::DarkGray)),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} слов", app.total_words()),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} на повторении", app.words_for_review_count()),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} забытых", app.forgotten_count()),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} на ближайшие {} дн.", app.upcoming_count(), app.upcoming_days),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(" │ API: ", Style::default().fg(Color::DarkGray)),
            Span::styled("localhost:3000", Style::default().fg(Color::Cyan)),
        ])
    } else {
        Line::from(vec![Span::styled(
            format!("  {}", app.status_message),
            status_style.add_modifier(Modifier::BOLD),
        )])
    };

    let status_bar = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let status_para = Paragraph::new(status_text).block(status_bar);
    frame.render_widget(status_para, area);
}
