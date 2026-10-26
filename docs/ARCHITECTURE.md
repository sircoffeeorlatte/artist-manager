# Архитектура (черновик)

## Слои

- **UI**: страницы Vue и общие компоненты. Не должны выполнять операции над произвольными путями напрямую.
- **Services**: контракты медиатек, системных диалогов, импорта и управления путями.
- **Persistence**: SQLite, миграции, транзакции и репозитории.
- **Native layer**: команды Rust для длительных и системных операций Windows: сканирование каталога, безопасное копирование/перемещение, вычисление хеша и извлечение системных метаданных.
- **Library storage**: пользовательская папка за пределами каталога установки.

## Структура хранилища в текущем MVP

```text
<выбранная папка>/MediaLibrary/
├─ IRL/
│  ├─ Фото/
│  ├─ Видео/
│  └─ Архивы/
├─ Аниме/
│  ├─ Фото/
│  ├─ Видео/
│  └─ Архивы/
└─ 3D/
   ├─ Фото/
   ├─ Видео/
   └─ Архивы/
```

В текущем этапе папки создаются на выбранном пользователем диске. SQLite, превью-кэш, резервные копии и журнал операций добавим отдельно, не смешивая их с установочными файлами программы.

## Жизненный цикл перемещения

1. Показать пользователю источник, назначение и оценку свободного места.
2. Проверить права доступа, существование путей, конфликты имён и доступное пространство.
3. Для перемещения на другой диск копировать в временное имя рядом с назначением.
4. Проверить размер и, при необходимости, хеш.
5. Завершить копирование, затем транзакционно обновить относительный путь в SQLite.
6. Удалять исходник только после успешной проверки, при отдельном режиме «Переместить» и подтверждении пользователя.
7. При отмене/ошибке журналировать состояние и не удалять источник.

## Безопасность

Для медиапревью asset-protocol scope динамически разрешает только выбранный корень медиатеки и его содержимое. Операции импорта проверяют тип файла и заданную категорию на Rust-стороне. Копирование проверяется по размеру перед возможным удалением исходника.


## Implemented MVP iteration: local folder and galleries

The current storage-root is selected by the user in Settings and remembered in the app-local settings file (with WebView localStorage as a browser-preview fallback). The Rust layer creates only the known category/type directories and never clears an existing directory. The library root is dynamically added to Tauri's asset-protocol scope so media previews work without exposing the whole filesystem.

Photos and videos are imported into `<root>/<category>/<type>/`. Copy mode leaves the source untouched. Move mode first copies and compares file size; only after a successful check does it try to remove the source. Conflicting filenames receive a `(1)`, `(2)`, etc. suffix. This is a basic safety check, not a cryptographic integrity check.

The current gallery scans the known folders on demand. SQLite indexing, EXIF/video metadata extraction, thumbnail generation, and creator-tag UI remain separate future work. The schema reserves `tags.tag_type='creator'` for author names connected through `media_file_tags`.
