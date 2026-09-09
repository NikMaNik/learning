# Архитектура

## Структура файлов

```
src/
├── main.rs      # точка входа: TUI + axum, state machine, keyboard handling
├── models.rs    # Word, App, ActiveScreen, InputField, Levenshtein
├── sm2.rs       # SM-2 алгоритм
├── db.rs        # SQLite: init, CRUD, review queries
├── api.rs       # axum REST handlers
├── tui.rs       # ratatui: все экраны
└── import.rs    # парсинг .md файлов
```

## Зависимости

| Крейт | Назначение |
|-------|------------|
| `ratatui` | TUI отрисовка |
| `crossterm` | backend для ratatui |
| `axum` | HTTP API |
| `tokio` | async runtime |
| `rusqlite` | SQLite |
| `serde` + `serde_json` | сериализация |
| `chrono` | дата/время |
| `dirs` | переход в домашнюю директорию | 
| `anyhow` | для ошибок |


## Схема данных

```
words
├── id              INTEGER PRIMARY KEY
├── english         TEXT
├── russian         TEXT
├── ease_factor     REAL (начальное 2.5)
├── interval        INTEGER (дней)
├── repetitions     INTEGER
├── next_review     TEXT (RFC3339)
├── created_at      TEXT (RFC3339)
└── last_quality    INTEGER (последняя оценка review, NULL если не было)
```

Колонка `last_quality` добавляется миграцией (`ALTER TABLE`) для уже существующих баз.

## State Machine

```
MainScreen
├── [1] → ReviewScreen → ReviewResult (auto)
├── [2] → AddScreen
├── [3] → UpdateSearch → UpdateScreen → UpdateEdit
├── [4] → DeleteScreen → DeleteConfirm
├── [5] → ImportScreen → ImportResult
├── [6] → ForgotScreen (забытые слова)
├── [7] → UpcomingScreen (слова на N дней)
└── [q] → exit
```

`words.db` — в `~/.learning/words.db` (см. `created_learning_direction`).
