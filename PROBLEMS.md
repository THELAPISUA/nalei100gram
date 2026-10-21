# Аналіз проблем у коді Nalei100Gram

Дата аналізу: 21.10.2026
Стан: `main` @ `6f4e1e2` (+ незначні незакомічені зміни в `src/ui/ui_components.rs`)

Команди, що виконувались для перевірки:
`cargo build` (3 warnings), `cargo clippy --all-targets` (28 warnings), `cargo fmt --check` (чисто),
`cargo test` (**0 тестів**), аналіз `app.log` (76 КБ runtime-логу) та `data/save.db`.

---

## 0. Коротко (підсумок)

| # | Пріоритет | Проблема | Де |
|---|-----------|----------|-----|
| 1 | **P0** | Токени Telegram-бота зашиті в код і вже лежать у git-історії | `src/logic/backend.rs:38,41` |
| 2 | **P0** | Path traversal: ім'я файлу з Telegram пишеться на диск без перевірки | `src/logic/backend.rs:231-235` |
| 3 | **P0** | Бот **видаляє** повідомлення, які не вміє обробити (голосові, відео, контакти…) | `src/logic/backend.rs:221,268` |
| 4 | **P0** | Зламаний SQL в `edit_message`: 3 плейсхолдери, 2 значення → редагування мовчки не працює | `src/logic/db.rs:151-172` |
| 5 | **P1** | `unwrap()` на `None` у паноці: краш замість обробки помилки | 26 місць, див. §4 |
| 6 | **P1** | Редагування в UI не реалізовано (захардкоджене `"hello"`) | `src/ui/ui_components.rs:92-97` |
| 7 | **P1** | Надіслані фото/файли не зберігаються в БД (втрачаються при рестарті) | `src/logic/logic.rs:49,72` |
| 8 | **P1** | Повідомлення, надіслані з UI, не мають `message_id` → видалити/редагувати неможливо | `src/ui/logic.rs:49-67,141-163` |
| 9 | **P1** | `try_send` мовчки викидає повідомлення при заповненому каналі | `src/ui/logic.rs:62,155` |
| 10 | **P1** | Помилки `unwrap()`/паноти в трейї, звуці, буфері обміну → краш у будь-який момент | `src/ui/tray.rs`, `src/utils/audio.rs`, `src/ui/logic.rs` |
| 11 | **P1** | Tray «Exit» не виходить з програми — лише ховає вікно | `src/ui/tray.rs:110-114` |
| 12 | **P1** | `set_max_vol()` ставить системну гучність на 100% на кожне повідомлення | `src/utils/audio.rs:51-55` |
| 13 | **P1** | Немає вибору чату на старті → помилка Telegram «chat not found» | `src/ui/app.rs:30` |
| 14 | **P2** | Клон усіх чатів з усіма повідомленнями щокадру | `src/ui/app.rs:55` |
| 15 | **P2** | Синхронне читання/декод картинок у UI-потоку + `unwrap()` | `src/ui/ui_components.rs:198-201` |
| 16 | **P2** | Немає міграцій схеми БД → старі БД ламаються (є в лозі) | `src/logic/db.rs:47-113` |
| 17 | **P2** | Немає тестів, немає CI, 28 попереджень clippy | увесь проєкт |

---

## 1. Безпека (P0)

### 1.1 Токени бота у вихідному коді
`src/logic/backend.rs:37-41`
```rust
#[cfg(target_os = "linux")]
let bot = Bot::new("8297296496:AAHqAR8s3BEUX-mQ9KKo6U2Z6hBojSeTRGQ");
#[cfg(target_os = "windows")]
let bot = Bot::new("5649333069:AAH3I94QCOkQZaDQ8rWHvHJWscEe5v2ybYk");
```
* Токени лежать у git-історії (коміти `5c74f11`, `b17ae81`, …) — їх треба **відкликати через BotFather**, а не просто видалити з коду.
* Будь-хто, хто має доступ до репозиторію, повністю контролює бота.
* Ризик: спам від імені бота, підписка на ваші групи, доступ до переписок.

**Рекомендація:** токени в `.env` (через `dotenvy`) або в змінні оточення; у README — інструкція; у `.gitignore` — `.env`.

### 1.2 Path traversal при збереженні документів
`src/logic/backend.rs:228-237`
```rust
let file_name = doc.file_name.clone().unwrap_or(file_format);
let path = format!("data/temp/{}", file_name);
let mut destination_file = TokioFile::create(path.clone()).await?;
```
`doc.file_name` — це **ім'я, яке задає відправник**. Значення `../../../../home/user/.bashrc` (або `../../.config/autostart/x.desktop`) призведе до запису/перезапису довільного файлу поза `data/temp`. Для фото/стікерів проблеми немає (там ім'я = `file_id`), але для документів — так.

**Рекомендація:** `Path::new(&file_name).file_name()` + відкидати шляхові роздільники; далі перевіряти, що `canonicalize()` результату починається з `data/temp`.

### 1.3 `open::that(path)` — запуск довільного файлу з БД
`src/ui/ui_components.rs:135` — шлях береться з БД і передається в системний обробник. Разом з 1.2 це дає можливість «клацнути» на файл, який бот сам поклав у `data/temp` під контрольним ім'ям.

### 1.4 `set_max_vol()` — зміна системної гучності
`src/utils/audio.rs:51-55` викликається на **кожне** вхідне повідомлення (`backend.rs:115,171,218,265`):
```rust
let device = AudioDevice::from_default().unwrap();
let _ = device.set_vol(100);
let _ = device.set_mute(false);
```
Це не звичайна «зручність», а втручання в системні налаштування користувача без згоди (і без можливості вимкнути). Плюс `unwrap()` → панотика, якщо звукового пристрою немає.

### 1.5 Відсутність обмежень на вхідні дані
* Немає ліміту на довжину повідомлення (Telegram має 4096 символів) → `send_message` завжди поверне помилку.
* Немає перевірки `msg.chat_id()` — підставляється `ChatId(1)` (`backend.rs:90`).

---

## 2. Функціональні баги (P1/P2)

### 2.1 Зламаний `edit_message` (SQL)
`src/logic/db.rs:151-172`
```sql
UPDATE messages SET text = ? WHERE chat_id = ? AND telegram_message_id = ?;
```
bind-ів лише два: `msg.chat_id.0` та `msg.id.0`.

Як працює sqlx-sqlite (`sqlx-sqlite-0.9.0/src/arguments.rs:78-113`): він проходить по кількості плейсхолдерів у запиті і, коли аргументів менше, ніж плейсхолдерів, **лишає решту як NULL** («SQLite treats unbound variables as NULL»). Тому:

* `text` = `chat_id` (число!), `chat_id` = `telegram_message_id`, `telegram_message_id` = `NULL`;
* `WHERE telegram_message_id = NULL` → ні один рядок не збігається → `rows_affected = 0`;
* результат — **тихе «успішне» редагування, яке нічого не робить** (жодного логу про 0 рядків).

Правильно: `.bind(new_text).bind(chat_id).bind(message_id)` — і перевіряти `rows_affected()`.

### 2.2 Редагування не реалізовано
* `src/ui/ui_components.rs:92-97` — `new_text: Some("hello".into())` — заглушка, будь-яке редагування замінює текст на «hello».
* `src/ui/widgets.rs:5` — варіант `MessageAction::Edit` **ніколи не створюється** (у меню немає пункту «Редагувати») → clippy: `variant Edit is never constructed`.
* `src/logic/backend.rs:61` — `msg.new_text.clone().unwrap()` → панотика, якщо `new_text == None`.
* Коментар-залог у `src/ui/ui_components.rs:67-72` прямо фіксує незакриті задачі (пошук повідомлення в чаті, зручне редагування, видалення «для себе» і «для всіх», плейсхолдер копіювання).

### 2.3 Видалення «для всіх» для чужих повідомлень
`src/ui/ui_components.rs:101-108` → `ActionPacket::Delete` → `src/logic/backend.rs:68-72`:
```rust
let r = bot_for_gui.delete_message(msg.chat_id, msg.id).await;
```
Команда не перевіряє, чи є поточне повідомлення «своїм». Будь-який користувач чату може видалити повідомлення інших. Потрібен режим «видалити тільки в себе» (видаляти з локальної моделі + `delete_for_everyone` лише для власних повідомлень).

### 2.4 Бот видаляє незрозумілі повідомлення (втрата даних)
`src/logic/backend.rs:221,268`
```rust
} else {
    bot.delete_message(msg.chat.id, msg.id).await?;
}
```
Гілка `else` спрацьовує для **всього**, що не є text/photo/sticker/document: голосові повідомлення, відео, анімації, контакти, опитування, location, video_note тощо. Бот (якщо ввімкнено privacy mode) їх отримує — і **видаляє у групі**. Для повідомлень у каналах `msg.chat.id` — це `ChannelId`, а `delete_message` приймає `ChatId` → помилка API.

### 2.5 Надіслані фото/файли не потрапляють у БД
`src/logic/logic.rs:42-63` (FILE) та `65-87` (IMAGE):
```rust
let r = bot_for_gui.send_document(...).await;
player.play(SoundType::Send);
db_for_gui.clone().add_message(&packet).await;   // ← до перевірки результату
```
`packet.message_id` тут `None` (пакет прийшов з UI), а `Db::add_message` починається з:
```rust
let Some(message_id) = msg.message_id else {
    error!("Cannot save message without Telegram message ID");
    return;
};
```
→ рядок **не пишеться** взагалі. Підтвердження в `app.log`: `ERROR ... Cannot save message without Telegram message ID` (5 випадків).

Наслідок: після перезапуску всі надіслані фото/файли зникають з історії. Для TEXT-гілки id підставляється правильно (`logic.rs:26-30`) — там баг виправлений, а для файлів — ні. Також запис у БД відбувається **до** перевірки `r`, тобто невдала відправка теж фіксується.

### 2.6 Повідомлення з UI не отримують `message_id`
`src/ui/logic.rs:141-163` і `41-70` — локальна копія в UI створюється з `message_id: None` і **ніколи не оновлюється** після успішної відправки. Наслідки:
* `process_message` → `let id = msg.message_id?;` (`ui_components.rs:90,102`) → `None` → дія не формується → **власне повідомлення не можна видалити чи відредагувати**;
* видалення в Telegram неможливе, бо id невідомий.

Потрібен зворотний зв'язок від бекенда (напр. `ActionPacket::Sent(MessageId)`), щоб оновити локальну модель.

### 2.7 Ім'я чату береться з автора повідомлення
`src/ui/logic.rs:30-36`
```rust
let chat = self.get_or_create_chat(chat_id, m.username.clone());
```
У пакеті вже є `m.chat_name` (заповнюється в `backend.rs:98-102,155-159,202-206,249-253`), але він ігнорується. У підвалі новий чат підписується **ім'ям відправника** замість назви групи. Для груп це виглядає як «назва чату = останній хто написав».

### 2.8 Непрочитані не зберігаються
* `src/ui/ui_components.rs:36-38` — `msg.is_read = true` міняється лише в пам'яті, у БД не пишеться.
* `src/logic/backend.rs:94` — вхідні повідомлення одразу пишуться з `is_read: true`.
* `src/logic/db.rs:174` — метод `_mark_readad_all` (з UNDERSCORE = мертвий код) **ніколи не викликається**.

Тобто стан «прочитано/не прочитано» живеє тільки в RAM і зникає при рестарті; бейдж непрочитаних теж.

### 2.9 Немає вибору чату на старті → «chat not found»
`src/ui/app.rs:30` — `current_chat: ChatId(0)`. Чат `0` не існує, тому будь-яке повідомлення/файл, надіслане до вибору чату, падає з помилкою Telegram. Підтвердження в `app.log`:
```
ERROR nalei100gram::logic::logic] Failed to send message, err: A Telegram's error: Bad Request: chat not found
```
Так само `get_or_create_chat(ChatId(0), …)` (`ui/logic.rs:65,157`) створює фантомний чат у списку.

### 2.10 Помилки Telegram «ковтаються»
`src/logic/backend.rs:136,142,184,189,230,236` — оператор `?` прямо в обробнику `teloxide::repl`:
```rust
let file_info = bot.get_file(file_id.clone()).await?;   // помилка обриває обробник
```
Будь-яка помилка завантаження файлу зупиняє обробку повідомлення (у лозі: `ERROR teloxide::error_handlers] Error: MigrateToChatId(...)`), причому без коректного контексту. Треба обробляти помилки локально (`match`/`if let`), а не пропагувати з хендлера.

### 2.11 «Видалити» не працює для повідомлень без `message_id`
`src/ui/ui_components.rs:102` + `src/ui/logic.rs:167`:
```rust
let id = msg.message_id?;   // None → return None → дія не виконується
...
chat.messages.retain(|m| m.message_id != Some(msg_id));
```
`delete_msg` порівнює лише `Some(id)`, тому локальні повідомлення без id ніколи не видаляються, а користувач не отримує жодного фідбека.

### 2.12 Копіювання: дублювання + ризик панотики
`src/ui/widgets.rs:90` вже робить `ui.ctx().copy_text(...)`, а потім `src/ui/ui_components.rs:99` → `copy_to_clipboard` → `src/ui/logic.rs:169-172`:
```rust
let mut clipboard = Clipboard::new().unwrap();   // панотика, якщо буфера немає
clipboard.set_text(text).unwrap();                // панотика на помилці
```
У `app.log` є відповідні попередження:
```
WARN arboard::platform::linux::x11] Could not hand the clipboard contents over to the clipboard manager. The request timed out.
```
Тобто `arboard` тут зайвий (egui вже копіює) і потенційно крашить застосунок.

---

## 3. Надійність і панотики

### 3.1 Перелік небезпечних `unwrap` / `expect` / `panic!`

| Місце | Код | Наслідок |
|---|---|---|
| `src/utils/audio.rs:19` | `open_default_sink().expect(...)` | **застосунок не стартує** без звукового пристрою (контейнер, VM, CI) |
| `src/utils/audio.rs:52` | `AudioDevice::from_default().unwrap()` | панотика в backend-тасці на кожне повідомлення → backend мовчить |
| `src/logic/backend.rs:38,41` | токен у коді | див. §1.1 |
| `src/logic/backend.rs:61` | `msg.new_text.clone().unwrap()` | панотика на `None` |
| `src/logic/backend.rs:129,179,225` | `panic!("{}", e)` при `create_dir_all` | панотика всієї програми через проблему з диском |
| `src/logic/db.rs:25,40,90` | `panic!("{}", e)` | панотика при помилці БД/диска |
| `src/ui/logic.rs:99` | `.to_str().unwrap()` | **панотика на не-UTF8 імені файлу** (можливо на Linux) |
| `src/ui/logic.rs:170,171` | `Clipboard::new().unwrap()`, `set_text().unwrap()` | панотика при помилці буфера |
| `src/ui/ui_components.rs:199` | `load_texture(...).unwrap()` | панотика на битому зображенні |
| `src/ui/tray.rs:19,25,86` | `expect(...)` | панотика в UI-потоці, якщо немає system tray |
| `src/main.rs:23,142` | `expect(...)` | панотика при відсутності `assets/*.png` |

### 3.2 Panic hook не завершує програму
`src/main.rs:39-41`
```rust
std::panic::set_hook(Box::new(|info| { log::error!("PANIC: {info}"); }));
```
* Хук **замінює** стандартний — немає бекатрейсу, помилка невидима користувачеві.
* Паніка в UI-потоці (egui) **не** вбиває процес: вікно «завмирає», а backend продовжує працювати — застосунок виглядає як «звисла» без пояснень.
* У лозі видно реальні випадки:
  ```
  ERROR nalei100gram] PANIC: panicked at src/ui/logic.rs:167:43:
      called `Option::unwrap()` on a `None` value
  ERROR nalei100gram] PANIC: panicked at src/logic/db.rs:142:30:
  ```
  Тобто панотики вже відбувалися в продакшен-використанні.

**Рекомендація:** у хуку писати бекатрейс і викликати `std::process::exit(1)` (або хотя б показувати модальне вікно egui з текстом помилки).

### 3.3 Відсутність обробки помилок каналів
* `src/ui/logic.rs:62,155` і `src/ui/ui_components.rs:44,56` — `try_send(...)` з результатом, який **відкидається** (`let _ =`). Якщо канал (місткість 32) заповнений, повідомлення просто зникає, користувач нічого не бачить. Текст при цьому навіть не очищається (`ui/logic.rs:161`).
* Backend-таска (`backend.rs:48-77`) не має супервізії: панотика всередині → канал UI→Backend більше не читається → всі наступні повідомлення губиться без жодного повідомлення.
* `src/logic/backend.rs:54` — лог `"Received packet for chat {}` пишеться **після** фактичної відправки, тобто текст повідомлення вводить в оману.

### 3.4 Немає коректного завершення роботи
* `src/ui/tray.rs:110-114` — команда «Exit» лише ховає вікно:
  ```rust
  TrayCommand::Exit => { ctx.send_viewport_cmd(ViewportCommand::Visible(false)); }
  ```
  Процес продовжує жити → користувач не може закрити застосунок через трей (тільки `kill`).
* `src/ui/logic.rs:71-86` — блокування закриття вікна реалізоване **лише для Windows**; на Linux закриття вікна вбиває застосунок, на Windows — ховає. Поведінка платформозалежна й неочевидна.
* `src/main.rs:65-67` — `_backend_task` (JoinHandle) не перевіряється; при падінні backend-таски застосунок продовжує показувати UI, але без зв'язку з Telegram.

---

## 4. Продуктивність

### 4.1 Повний клон чатів щокадру
`src/ui/app.rs:55`
```rust
let chats = self.chats.clone();
```
Клонується **весь** `Vec<Chat>` разом з усіма `Vec<MessagePacket>` на **кожному кадрі** (egui ~= 60 FPS). При 1000 повідомлень це десятки тисяч алокацій/сек.

### 4.2 O(n²) пошук
`src/ui/app.rs:72-77` — для кожного чату в списку виконується `self.chats.iter().find(...)`, щоб дістати `unreaded_count`; плюс `src/ui/logic.rs:117-122` — `last_message` ще раз робить лінійний пошук по чатах для кожного рядка списку. Разом — O(n²) на кадр.

### 4.3 Синхронне декодування зображень у UI-потоці
`src/ui/ui_components.rs:198-201`
```rust
let data = std::fs::read(p.clone()).unwrap_or(ERROR_LOAD.to_vec());
let photo_tmp = load_texture(ctx, p, &data).unwrap();
```
Читання файлу + декод PNG/JPEG відбуваються **в середині кадру** → видимі фризи. Крім того:
* кеш `photos: HashMap<String, TextureHandle>` ніколи не очищається → зростання пам'яті (VRAM) необмежене;
* текстура завантажується в **повному** розмірі (Telegram-фото до 4000×3000 = ~48 МБ VRAM на картинку), хоча малюється обмежена розмір 380×300 px.

### 4.4 Завантаження всієї історії в пам'ять
`src/logic/db.rs:251-352` — `load_chats` читає **усі** повідомлення без пагінації і тримає їх у RAM (а потім UI ще раз клонує їх щокадру). Зі зростанням бази старт застосунку буде лінійно довшим.

---

## 5. База даних

### 5.1 Немає міграцій схеми
`src/logic/db.rs:47-113` — лише `CREATE TABLE IF NOT EXISTS`. Якщо схема зміниться (а вона вже змінювалась), стара БД **залишається як є** → runtime-помилки. Це вже підтверджено логом:
```
ERROR ... error returned from database: (code: 1) 8 values for 9 columns
```
Потрібен `sqlx::migrate!` + папка `migrations/`, або принаймні `user_version` і `ALTER TABLE`.

### 5.2 Немає UNIQUE на `(chat_id, telegram_message_id)`
`src/logic/db.rs:68-82` — дублікати повідомлень можливі (повторна обробка апдейту, реплей) і не блокуються. У БД уже видно дублікати тексту (`data/save.db`: `asd` двічі).

### 5.3 `PRAGMA` через пул — працює не для всіх з'єднань
`src/logic/db.rs:104-110`
```rust
sqlx::query("PRAGMA journal_mode=WAL;").execute(&self.pool).await
```
PRAGMA діє лише для **одного** з'єднань пулу. Те саме з `foreign_keys`/`busy_timeout` (яких немає). Правильно: задавати в `SqliteConnectOptions`:
```rust
SqliteConnectOptions::new()
    .filename("data/save.db")
    .create_if_missing(true)
    .journal_mode(SqliteJournalMode::Wal)
    .busy_timeout(Duration::from_secs(5))
    .max_connections(1)   // для SQLite — типово 1..2
```
Зараз пул має до 10 з'єднань до одного SQLite-файлу → ризик `database is locked`.

### 5.4 `add_message(self, …)` замість `&self`
`src/logic/db.rs:115` — метод забирає `Db` **за значенням** (тобто клонує пул) — зайвий клон у кожному виклику; решта методів використовує `&self`. Так само `logic.rs:30,49,72` — `db_for_gui.clone().add_message(...)`.

### 5.5 Ручна синхронізація enum ↔ INTEGER
`src/utils/types.rs:21-28` (порядок варіантів) ↔ `src/logic/db.rs:311-317` (ручний match `0=>TEXT, 1=>IMAGE, 2=>STICKER, 3=>FILE`) ↔ `src/logic/db.rs:136` (`msg_type.clone() as i64`). Три місця, які треба синхронізувати вручну; при додаванні варіанта старі рядки почнуть читатися неправильно. Краще — `#[repr(i64)]` + явні дискримінатори.

### 5.6 Дані в БД не відповідають моделі UI
* `messages.username` — порожній рядок, якщо в користувача немає `full_name` (`backend.rs:276-288`).
* `msg_name` для тексту пишеться як `""` (`db.rs:137`), а `chat_name` — порожній рядок.
* Час зберігається як `%H:%M` без дати (`db.rs` / `backend.rs:91`) → **повідомлення різних днів не можна відрізнити** і сортувати.

---

## 6. UI / UX

### 6.1 Заголовок «Nalei100Gram» не видно
`src/ui/app.rs:50` — `ui.heading("Nalei100Gram")` малюється в кореневому `Ui` (на весь viewport) **до** `Panel::left`, а `CentralPanel` показується пізніше (`app.rs:216`) і перекриває його. Заголовок опиняється під лівою панеллю.

### 6.2 Подвійний виклик overlay-у drag&drop
`src/ui/app.rs:218` і `src/ui/ui_components.rs:62` — `preview_files_being_dropped` викликається двічі в одному кадрі → затемнення малюється двічі в один шар (колір темніший за задумане).

### 6.3 Надсилання файлів через drag&drop — тиха втрата
`src/ui/logic.rs:41-70`
```rust
while let Some(file) = self.pending_files.pop() {       // ← вилучається одразу
    if file.size <= 41943040 { ... }                     // ← інакше просто ігнорується
```
* файл > 40 МБ — **мовчки** видаляется, без повідомлення користувачу;
* при `try_send` помилці файл уже вилучено з черги → втрачено назавжди;
* `std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)` (`ui/logic.rs:93`) — якщо метадані не прочитались, розмір стає 0 → файл проходить перевірку і падає вже на стороні Telegram;
* розширення порівнюються **без урахування регістру** (`ui/logic.rs:45-48`): `photo.PNG` → піде як `FILE`, а не `IMAGE`; `jpeg` не враховується.

### 6.4 Enter не відрізняється від Shift+Enter
`src/ui/app.rs:206-207`
```rust
let enter_pressed = text_input_res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
```
`key_pressed(Enter)` спрацьовує і з `Shift`/`Ctrl` → багаторядковий ввід неможливий. Також `lost_focus()` — крихкий спосіб: якщо фокус з текстового поля зник з іншої причини (клік по чату в сайдбарі), повідомлення теж може надіслатись — уже в **інший** чат.

### 6.5 Фіксовані розміри бульбашок
`src/ui/widgets.rs:57` (`set_max_width(400.0)`), `widgets.rs:151` (`content_width = 380.0`), `widgets.rs:240` (`set_width(300.0)`) — при маленькому вікні текст виходить за межі; при розширенні — бульбашки не розтягуються.

### 6.6 Немає візуального фідбека
Немає ні спінерів, ні статусів «доставлено / не доставлено», ні повідомлення про помилки надсилання. `try_send` та помилки Telegram непомітні користувачеві.

---

## 7. Структура коду та якість

### 7.1 Clippy: 28 попереджень
```
cargo clippy --all-targets  →  28 warnings
```
Найбільш важливі:
* `src/logic/backend.rs:74` — `unreachable_patterns`: `_ => {}` у	match по `ActionPacket` не може спрацювати (усі 3 варіанти вже покриті).
* `src/ui/widgets.rs:5` — `variant Edit is never constructed`.
* `src/utils/types.rs:53` — `field chat_name is never read` (підтверджує баг §2.7).
* `src/logic/backend.rs:280` — `unnecessary_unwrap` (`is_some()` + `unwrap()`).
* `src/logic/db.rs`/`logic.rs` — `needless_borrow`, `unnecessary_cast` (`m.chat.id.0 as i64`).
* `src/ui/logic.rs:146,156,157` та `backend.rs:59,60` — `clone_on_copy` для `ChatId`/`MessageId`.
* `src/utils/types.rs:21-24` — `upper_case_acronyms` для `TEXT`/`IMAGE`/`STICKER`/`FILE`.
* `src/logic/mod.rs:3` — `module_inception` (`logic::logic`).
* `src/ui/ui_components.rs:142` — `useless_format` (`format!("Ошибка рендера неизвестний тип")`).

### 7.2 Дублювання коду
`src/logic/backend.rs:86-269` — чотири майже ідентичні блоки обробки (text / photo / sticker / document): кожен створює `MessagePacket` з 9 полями, дублюючи `chat_name` через `msg.chat.title().unwrap_or(msg.chat.first_name().unwrap_or("Unknown name"))`. Зміна структури пакета = 4 правки.

### 7.3 Плутанина в іменах
* `crate::utils::types::Message` (власний тип) vs `teloxide::types::Message` — два різні `Message` в одному проєкті; у `backend.rs` імпортується teloxide-версія через `prelude::*`.
* `is_bot: true` для **власних** повідомлень (`ui/logic.rs:55,146`) — семантика поля «bot» = «виходяще», плутає.
* `unreaded_count` / `_mark_readad_all` — орфографія (`unread`).
* `chad_id` (`ui/logic.rs:117`) — очевидна помилка у назві параметра.

### 7.4 Немає тестів і CI
* `cargo test` → **0 тестів**. Немає жодного тесту на: мапінг `MsgType` ↔ БД, парсер імен файлів, фільтр розміру файлів, `delete_msg`, побудову `MessagePacket`.
* Немає `.github/workflows`, `rustfmt.toml`, `clippy.toml`, `deny.toml`.
* Немає `rust-toolchain.toml` — при `edition = "2024"` потрібен Rust ≥ 1.85.

### 7.5 `Cargo.toml`
* `default = ["linux-tray"]` — фіча `linux-tray` вмикає `tray-icon/ksni` **навіть на Windows**, де це порожньо; `windows-tray = []` — **порожня фіча**, яка нічого не робить (на Windows не вмикається `tray-icon/muda-common-controls-v6`).
* `eframe` з `features = ["wgpu"]` + `default-features` (які тягнуть і `glow`) — надлишково.
* Немає `[[bin]]`, `profile.release`, `opt-level`, `lto`, `strip` — релізний білд не оптимізований.
* `image` без `default-features` — добре, але `webp` усе одно тягне.

---

## 8. Інфраструктура / запуск

* `src/main.rs:19-23,22` — лог пишеться у `app.log` **відносно поточного каталогу**; `src/logic/db.rs:30` — БД теж `data/save.db` відносно CWD. Запуск з іншого каталогу створює **порожню** БД і новий лог (дані попереднього запуску «зникають»). Треба використовувати шлях від каталогу виконуваного файлу або user-data dir.
* `src/main.rs:19-23` — лог без ротації (вже 76 КБ) і без обмеження розміру.
* `src/main.rs:73` — `setup.instance_descriptor.backends = eframe::wgpu::Backends::VULKAN;` — примусово лише Vulkan. На системах без Vulkan (або з битим ICD — у лозі `Skipping this driver` для `libvulkan_dzn.so`) застосунок не стартує. Доцільніше `Backends::ALL` / `PRIMARY` + fallback.
* `src/main.rs:28-37` — фільтри логів налаштовані, але `env_logger` пише тільки в файл, у консоль нічого не потрапляє (на Windows `#![windows_subsystem = "windows"]` і паніки невидимі).
* `src/main.rs:1` — атрибут `windows_subsystem` діє лише на Windows; на Linux панотики друкуються в stderr, але застосунок падає без красивого повідомлення.
* `src/logic/backend.rs:37-41` — бот визначений лише для `linux` і `windows` → **проєкт не збирається на macOS** (змінна `bot` не існує).

---

## 9. Рекомендований порядок робіт

**Крок 1 — безпека (P0)**
1. Відкликати обидва токени через `@BotFather`, винести в `.env`.
2. Санітизація імен файлів + перевірка, що шлях лишається в `data/temp`.
3. Прибрати `bot.delete_message(...)` з гілки `else`; не видаляти чужі повідомлення за замовчуванням.

**Крок 2 — база даних (P1)**
4. Полагодити `edit_message` (3 плейсхолдери → 3 bind-и + перевірка `rows_affected`).
5. `migrations/` + `sqlx::migrate!`; UNIQUE на `(chat_id, telegram_message_id)`.
6. `SqliteConnectOptions` з `journal_mode(Wal)`, `busy_timeout`, `max_connections(1)`.
7. Зберігати `message_id` для фото/файлів; писати `is_read` у БД.

**Крок 3 — надійність (P1)**
8. Прибрати всі `panic!`/`unwrap()` на шляхах, де можлива помилка (звук, трей, буфер, імена файлів, БД).
9. Panic hook: бекатрейс + `exit(1)`; показати помилку в egui.
10. `await` замість `try_send` (або `blocking_send` з обробкою помилки) + фідбек користувачу.
11. Трей «Exit» → `std::process::exit(0)`; єдиний механізм закриття на всіх ОС.

**Крок 4 — функціонал (P1/P2)**
12. Зворотний зв'язок `Sent(MessageId)` → оновлення локальної моделі → редагування/видалення власних повідомлень.
13. Реалізувати редагування (пункт у меню + поле вводу) замість `"hello"`.
14. Вибір чату за замовчуванням; блокувати надсилання, якщо чат не обрано.
15. Використовувати `m.chat_name` для назви чату.

**Крок 5 — продуктивність і якість (P2)**
16. Прибрати `self.chats.clone()` щокадру; індекси за `chat_id` (HashMap замість Vec).
17. Завантажувати/декодувати зображення у фоні; обмежити розмір текстури; чистити кеш.
18. Пагінація історії в `load_chats`.
19. Тести (хоча б юніт-тести на мапінг типів, фільтри, парсинг імен) + CI з `fmt`/`clippy -D warnings`.
20. Прибрати 28 попереджень clippy, `module_inception`, дублювання пакетів, перейменувати поля.

---

*Документ створено в режимі «лише аналіз» — зміни у вихідний код не вносилися.*
