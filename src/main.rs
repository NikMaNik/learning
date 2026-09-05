mod api;
mod db;
mod import;
mod models;
mod sm2;
mod tui;

use std::io::{self, Write};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use rusqlite::Connection;

use models::{ActiveScreen, App, InputField};

fn remove_spaces(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

fn setup_terminal() -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::cursor::EnableBlinking,
        crossterm::cursor::Show,
    )?;
    Ok(())
}

fn restore_terminal() {
    let _ = crossterm::terminal::disable_raw_mode();
    let mut stdout = io::stdout();
    let _ = crossterm::execute!(
        stdout,
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::cursor::Show,
    );
    let _ = stdout.flush();
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    tracing_subscriber::fmt::init();

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        original_hook(info);
    }));

    setup_terminal()?;

    let mut terminal = ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;

    let conn = Connection::open("words.db").expect("Failed to open database");
    db::init_db(&conn).expect("Failed to init database");

    let api_conn = Connection::open("words.db").expect("Failed to open database for API");
    tokio::spawn(async move {
        let router = api::build_router(api_conn);
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
            .await
            .expect("Failed to bind API server");
        axum::serve(listener, router)
            .await
            .expect("API server failed");
    });

    let mut app = App::new(conn);
    app.refresh_words();
    app.refresh_review_queue();

    let result = run_app(&mut terminal, &mut app);

    restore_terminal();

    if let Err(err) = result {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

fn run_app(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| tui::render(frame, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match app.current_screen {
                    ActiveScreen::Main => handle_main(app, key.code),
                    ActiveScreen::Review => handle_review(app, key.code),
                    ActiveScreen::Add => handle_add(app, key.code),
                    ActiveScreen::Update => handle_update(app, key.code),
                    ActiveScreen::UpdateSearch => handle_update_search(app, key.code),
                    ActiveScreen::UpdateEdit => handle_update_edit(app, key.code),
                    ActiveScreen::Delete => handle_delete(app, key.code),
                    ActiveScreen::DeleteConfirm => handle_delete_confirm(app, key.code),
                    ActiveScreen::Import => handle_import(app, key.code),
                    ActiveScreen::ImportResult => handle_import_result(app, key.code),
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_main(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('1') => {
            app.clear_inputs();
            app.refresh_review_queue();
            app.current_screen = ActiveScreen::Review;
            app.current_word_index = 0;
        }
        KeyCode::Char('2') => {
            app.clear_inputs();
            app.current_screen = ActiveScreen::Add;
            app.active_field = InputField::English;
        }
        KeyCode::Char('3') => {
            app.clear_inputs();
            app.refresh_words();
            app.current_screen = ActiveScreen::UpdateSearch;
            app.active_field = InputField::ReviewTranslation;
        }
        KeyCode::Char('4') => {
            app.clear_inputs();
            app.current_screen = ActiveScreen::Import;
            app.active_field = InputField::Path;
        }
        KeyCode::Char('5') => {
            app.clear_inputs();
            app.refresh_words();
            app.current_screen = ActiveScreen::Delete;
            app.selected = 0;
        }
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        _ => {}
    }
}

fn handle_review(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            app.clear_inputs();
            app.current_screen = ActiveScreen::Main;
        }
        KeyCode::Enter => {
            if app.show_review_answer {
                app.input_review.clear();
                app.show_review_answer = false;

                if app.current_word_index + 1 < app.review_queue.len() {
                    app.current_word_index += 1;
                } else {
                    app.refresh_review_queue();
                    if app.review_queue.is_empty() {
                        app.status_message = "Все слова повторены!".to_string();
                        app.status_is_error = false;
                        app.current_screen = ActiveScreen::Main;
                    }
                }
                return;
            }

            if let Some(word) = app.current_review_word().cloned() {
                let user_answer = remove_spaces(&app.input_review);
                let correct_answer = remove_spaces(&word.russian);

                let distance = models::levenshtein_distance(&user_answer, &correct_answer);
                let quality = models::distance_to_quality(distance, correct_answer.len());

                let (new_interval, new_ef, new_reps) =
                    sm2::calculate(quality, word.repetitions, word.ease_factor, word.interval);
                let _ = db::save_review(&app.db, word.id, quality, new_interval, new_ef, new_reps);

                app.last_quality = quality;
                app.last_distance = distance;
                app.last_next_interval = new_interval;

                if quality >= 3 {
                    app.status_message = format!(
                        "Верно! (quality: {quality}/5) Следующее повторение через {new_interval} дн."
                    );
                    app.status_is_error = false;
                } else {
                    app.status_message = format!(
                        "Почти! (quality: {quality}/5) Правильно: {} — {}",
                        word.english, word.russian
                    );
                    app.status_is_error = true;
                }
                app.show_review_answer = true;
            }
        }
        KeyCode::Tab => {
            if app.current_word_index + 1 < app.review_queue.len() {
                app.current_word_index += 1;
                app.input_review.clear();
                app.show_review_answer = false;
            } else {
                app.status_message = "Нет больше слов для повторения".to_string();
                app.status_is_error = false;
                app.current_screen = ActiveScreen::Main;
            }
        }
        KeyCode::Char(c) => {
            app.input_review.push(c);
        }
        KeyCode::Backspace => {
            app.input_review.pop();
        }
        _ => {}
    }
}

fn handle_add(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            app.clear_inputs();
            app.current_screen = ActiveScreen::Main;
        }
        KeyCode::Tab => {
            app.active_field = match app.active_field {
                InputField::English => InputField::Russian,
                _ => InputField::English,
            };
        }
        KeyCode::Enter => {
            let english = remove_spaces(&app.input_english);
            let russian = remove_spaces(&app.input_russian);

            if english.is_empty() || russian.is_empty() {
                app.status_message = "Заполни оба поля".to_string();
                app.status_is_error = true;
                return;
            }

            let is_duplicate = app.words.iter().any(|w| {
                w.english.to_lowercase() == english.to_lowercase()
                    || w.russian.to_lowercase() == russian.to_lowercase()
            });

            if is_duplicate {
                app.status_message =
                    format!("Слово \"{}\" или \"{}\" уже существует", english, russian);
                app.status_is_error = true;
                return;
            }

            match db::add_word(&app.db, &english, &russian) {
                Ok(_) => {
                    app.status_message = format!("Добавлено: {} — {}", english, russian);
                    app.status_is_error = false;
                    app.refresh_words();
                    app.input_english.clear();
                    app.input_russian.clear();
                }
                Err(e) => {
                    app.status_message = format!("Ошибка: {}", e);
                    app.status_is_error = true;
                }
            }
        }
        KeyCode::Char(c) => match app.active_field {
            InputField::English => app.input_english.push(c),
            InputField::Russian => app.input_russian.push(c),
            _ => {}
        },
        KeyCode::Backspace => {
            match app.active_field {
                InputField::English => app.input_english.pop(),
                InputField::Russian => app.input_russian.pop(),
                _ => None,
            };
        }
        _ => {}
    }
}

fn handle_update_search(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            app.clear_inputs();
            app.current_screen = ActiveScreen::Main;
        }
        KeyCode::Enter => {
            let query = app.search_query.trim().to_string();
            if query.is_empty() {
                app.status_message = "Введи слово для поиска".to_string();
                app.status_is_error = true;
                return;
            }

            app.search_words(&query);

            if app.search_results.is_empty() {
                app.status_message = format!("Слово \"{}\" не найдено", query);
                app.status_is_error = true;
                return;
            }

            let query_lower = query.to_lowercase();
            if let Some(pos) = app.search_results.iter().position(|w| {
                w.english.to_lowercase() == query_lower || w.russian.to_lowercase() == query_lower
            }) {
                let word = app.search_results[pos].clone();
                app.input_english = word.english;
                app.input_russian = word.russian;
                app.active_field = InputField::English;
                app.current_screen = ActiveScreen::UpdateEdit;
            } else {
                app.selected = 0;
                app.current_screen = ActiveScreen::Update;
            }
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
        }
        KeyCode::Backspace => {
            app.search_query.pop();
        }
        _ => {}
    }
}

fn handle_update(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            app.clear_inputs();
            app.current_screen = ActiveScreen::UpdateSearch;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if app.selected + 1 < app.search_results.len() {
                app.selected += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.selected > 0 {
                app.selected -= 1;
            }
        }
        KeyCode::Enter => {
            if let Some(word) = app.search_results.get(app.selected) {
                let word = word.clone();
                app.input_english = word.english;
                app.input_russian = word.russian;
                app.active_field = InputField::English;
                app.current_screen = ActiveScreen::UpdateEdit;
            }
        }
        _ => {}
    }
}

fn handle_update_edit(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            app.clear_inputs();
            app.current_screen = ActiveScreen::UpdateSearch;
            app.refresh_words();
        }
        KeyCode::Tab => {
            app.active_field = match app.active_field {
                InputField::English => InputField::Russian,
                _ => InputField::English,
            };
        }
        KeyCode::Enter => {
            let english = remove_spaces(&app.input_english);
            let russian = remove_spaces(&app.input_russian);

            if english.is_empty() || russian.is_empty() {
                app.status_message = "Заполни оба поля".to_string();
                app.status_is_error = true;
                return;
            }

            if let Some(word) = app.search_results.get(app.selected) {
                let id = word.id;
                match db::update_word(&app.db, id, &english, &russian) {
                    Ok(_) => {
                        app.status_message = format!("Обновлено: {} — {}", english, russian);
                        app.status_is_error = false;
                        app.refresh_words();
                        app.current_screen = ActiveScreen::UpdateSearch;
                        app.clear_inputs();
                    }
                    Err(e) => {
                        app.status_message = format!("Ошибка: {}", e);
                        app.status_is_error = true;
                    }
                }
            }
        }
        KeyCode::Char(c) => match app.active_field {
            InputField::English => app.input_english.push(c),
            InputField::Russian => app.input_russian.push(c),
            _ => {}
        },
        KeyCode::Backspace => {
            match app.active_field {
                InputField::English => app.input_english.pop(),
                InputField::Russian => app.input_russian.pop(),
                _ => None,
            };
        }
        _ => {}
    }
}

fn handle_delete(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            app.clear_inputs();
            app.current_screen = ActiveScreen::Main;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if app.selected + 1 < app.words.len() {
                app.selected += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.selected > 0 {
                app.selected -= 1;
            }
        }
        KeyCode::Enter if app.words.get(app.selected).is_some() => {
            app.current_screen = ActiveScreen::DeleteConfirm;
        }
        _ => {}
    }
}

fn handle_delete_confirm(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc | KeyCode::Char('n') => {
            app.current_screen = ActiveScreen::Delete;
        }
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if let Some(word) = app.words.get(app.selected) {
                let id = word.id;
                let name = format!("{} — {}", word.english, word.russian);
                match db::delete_word(&app.db, id) {
                    Ok(_) => {
                        app.status_message = format!("Удалено: {}", name);
                        app.status_is_error = false;
                        app.refresh_words();
                        if app.selected >= app.words.len() && app.selected > 0 {
                            app.selected -= 1;
                        }
                        app.current_screen = ActiveScreen::Delete;
                    }
                    Err(e) => {
                        app.status_message = format!("Ошибка: {}", e);
                        app.status_is_error = true;
                    }
                }
            }
        }
        _ => {}
    }
}

fn handle_import(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            app.clear_inputs();
            app.current_screen = ActiveScreen::Main;
        }
        KeyCode::Enter => {
            let path = app.input_path.trim().to_string();

            if path.is_empty() {
                app.status_message = "Укажи путь к файлу".to_string();
                app.status_is_error = true;
                return;
            }

            match import::read_file(&path) {
                Ok(content) => {
                    let words = import::parse_markdown(&content);
                    if words.is_empty() {
                        app.status_message = "Не найдено слов в файле".to_string();
                        app.status_is_error = true;
                        return;
                    }
                    match db::import_words(&app.db, &words) {
                        Ok(count) => {
                            app.import_count = count;
                            app.refresh_words();
                            app.current_screen = ActiveScreen::ImportResult;
                        }
                        Err(e) => {
                            app.status_message = format!("Ошибка импорта: {}", e);
                            app.status_is_error = true;
                        }
                    }
                }
                Err(e) => {
                    app.status_message = format!("Ошибка чтения файла: {}", e);
                    app.status_is_error = true;
                }
            }
        }
        KeyCode::Char(c) => {
            app.input_path.push(c);
        }
        KeyCode::Backspace => {
            app.input_path.pop();
        }
        _ => {}
    }
}

fn handle_import_result(app: &mut App, key: KeyCode) {
    if key == KeyCode::Esc {
        app.clear_inputs();
        app.current_screen = ActiveScreen::Main;
    }
}
