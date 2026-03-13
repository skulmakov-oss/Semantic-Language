# Semantic Roadmap v0.3

Этот план переводит `EXO_DNA` в исполнимые задачи, привязанные к текущей кодовой базе.

## Track A: Frontend & Parser

Цель: завершить arena-first frontend и единообразную диагностику.

- [ ] Удалить остатки legacy-веток, где `Expr/Stmt` ещё интерпретируются как owned-деревья.
  - Модуль: `src/frontend.rs`
- [ ] Завершить переход на `ExprId/StmtId/SymbolId` во всех публичных API frontend.
  - Модуль: `src/frontend.rs`
- [ ] Ввести metadata/doc-comment узлы для `Law/Entity`.
  - Модуль: `src/frontend.rs`
- [ ] Унифицировать ошибки parser/type/lowering в rustc-style формат.
  - Модули: `src/frontend.rs`, `src/semantics/mod.rs`
- [ ] Подготовить REPL parser mode (single-input incremental parse).
  - Модуль: `src/bin/smc.rs` (+ новый REPL модуль при необходимости)

Acceptance:

- `cargo test` green;
- diagnostics содержат `line:col`, context и caret во всех критических путях.

## Track B: Semantics & Type System

Цель: формализовать строгую семантику Logos + типовую политику.

- [ ] Зафиксировать типовую решётку: `Int`, `Fx`, `QVec<N>`, `Mask`, `Str`, `Bool`, `Quad`, `Unit`.
  - Модуль: `src/semantics/mod.rs`
- [ ] Довести проверку совместимости `QVec<N>` по размерности (операции, присваивания, вызовы).
  - Модуль: `src/semantics/mod.rs`
- [ ] Формально отделить implicit/explicit cast policy (`Int -> Fx` only implicit).
  - Модуль: `src/semantics/mod.rs`
- [ ] Проверка уникальности `Law` в `Entity`, duplicate `Entity`, shadowing policy внутри `Law`.
  - Модуль: `src/semantics/mod.rs`
- [ ] Dead law branch detection (warning) и стабильный law scheduling по priority.
  - Модуль: `src/semantics/mod.rs`

Acceptance:

- `smc check` даёт стабильный отчёт;
- негативные тесты на mismatch/shadowing/duplicate покрыты.

## Track C: IR, Bytecode, VM

Цель: стабилизировать контракт выполнения и подготовить эволюцию формата.

- [ ] Ввести capability/version таблицу в SemCode header.
  - Модуль: `src/semcode_format.rs`
- [ ] Зафиксировать immutable IR boundary после lowering.
  - Модули: `src/frontend.rs`, `src/semantics/mod.rs`
- [ ] Поддержка gate surface в pipeline (`GateRead/GateWrite/PulseEmit`) с явной политикой encode/decode.
  - Модули: `src/frontend.rs`, `src/semcode_format.rs`, `src/semcode_vm.rs`
- [ ] Расширить VM validation на новые секции формата.
  - Модуль: `src/semcode_vm.rs`
- [ ] Подготовить compatibility tests между версиями bytecode.
  - Тесты: `tests/golden_semcode.rs` + новые golden-наборы по версиям.

Acceptance:

- golden tests стабильны;
- разбор заголовка и версий детерминирован, с корректными ошибками.

## Track D: no_std Readiness

Цель: снизить зависимости от std-контейнеров в критических местах.

- [ ] Локализовать std-only код за feature gates.
  - Модули: `src/lib.rs`, `src/frontend.rs`, `src/semantics/mod.rs`
- [ ] Подготовить no_std-friendly коллекции/аллокаторы для frontend/semantics.
  - Модули: `src/frontend.rs`, `src/semantics/mod.rs`
- [ ] Отдельный CI профиль `--no-default-features` для smoke-check.
  - Конфиг/CI

Acceptance:

- минимальная сборка no_std проходит для core слоёв;
- std-only части чётко изолированы.

## CLI Milestones

- [x] `smc check <input.sm>` — семантический анализ без записи `.smc`.
  - Модуль: `src/bin/smc.rs`
- [ ] `smc repl` — интерактивный режим.
- [ ] `smc explain <error-code>` — справка по диагностике.

## Технический долг (приоритет высокий)

- [ ] Стандартизовать функцию вывода имён по `SymbolId` (одна точка преобразования).
- [ ] Сократить дублирование между classic parser и Logos parser.
- [ ] Выделить frontend arena/types в отдельный модуль (`src/frontend/ast.rs`) для читаемости.

## Definition of Done для v0.3

- `cargo check` и `cargo test` green.
- golden-наборы обновлены и стабильны.
- публичные API frontend/semantics согласованы на ID-модели.
- документация (`EXO_DNA`, roadmap, diagnostics) синхронизирована с кодом.
