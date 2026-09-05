# REST API

Базовый URL: `http://localhost:3000`

## Эндпоинты

| Метод | Путь | Описание |
|-------|------|----------|
| `GET` | `/api/words` | все слова |
| `POST` | `/api/add` | добавить слово |
| `PUT` | `/api/update/{id}` | обновить слово |
| `DELETE` | `/api/del/{id}` | удалить слово |
| `GET` | `/api/rev` | слова для ревью |
| `POST` | `/api/review/{id}` | записать результат |
| `POST` | `/api/import` | импорт из markdown |

## Примеры

### Добавить слово

```bash
curl -X POST http://localhost:3000/api/add \
  -H "Content-Type: application/json" \
  -d '{"english":"hello","russian":"привет"}'
```

### Обновить слово

```bash
curl -X PUT http://localhost:3000/api/update/1 \
  -H "Content-Type: application/json" \
  -d '{"english":"hello","russian":"приветствую"}'
```

### Удалить слово

```bash
curl -X DELETE http://localhost:3000/api/del/1
```

### Ревью (оценка 0-5)

```bash
curl -X POST http://localhost:3000/api/review/1 \
  -H "Content-Type: application/json" \
  -d '{"quality":4}'
```

### Импорт из markdown

```bash
curl -X POST http://localhost:3000/api/import \
  -H "Content-Type: application/json" \
  -d '{"content":"- hello — привет\n- world — мир"}'
```

## Формат markdown для импорта

```
- hello — привет
- world — мир
```

или таблица:

```
| english | russian |
| hello   | привет  |
```
