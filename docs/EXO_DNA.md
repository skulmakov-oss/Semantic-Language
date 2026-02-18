# EXOcode DNA

Этот документ фиксирует архитектурную ДНК EXOcode: что мы заимствуем, что сознательно не берём, и что остаётся уникальным слоем EXO.

## Принципы

- Не копирование синтаксиса, а заимствование сильных инженерных решений.
- Приоритет: читаемость Logos + строгая семантика + предсказуемая VM.
- Любая фича должна укладываться в цепочку: `Source -> AST -> IR -> Bytecode -> VM`.

## Заимствования

### Python

- `INDENT/DEDENT` как дисциплина блоков для Logos frontend.
- `logical line + continuation depth` для читаемых многострочных выражений.
- дружелюбные диагностики: `Expected X, got Y`, caret, hints.
- docstring-подобные комментарии для метаданных `Law/Entity`.
- REPL-культура для короткого цикла разработки.

Не берём:

- динамическую типизацию;
- runtime monkey-patching;
- неявную магию выполнения.

### Rust

- `Span/SourceMark` на всех этапах.
- string interning (`SymbolId`) вместо сырого `String` в AST/таблицах.
- parser discipline (Pratt для выражений).
- диагностическая модель с label/context.
- no_std-готовность и zero-cost подход.
- `enum + match` как фундамент AST/IR/VM.
- RAII guard-паттерны и feature-gated архитектура.

Не берём:

- сложность lifetime-модели в пользовательской поверхности языка;
- macro-heavy стиль как основу DSL.

### Java

- bytecode как стабильный контракт.
- versioned binary format.
- magic header + constant pool.
- строгая валидация сигнатур и этапность пайплайна.

Не берём:

- OOP-наследование как каркас языка;
- verbosity и checked-exception стиль.

### C++

- контроль layout (repr(C)-подход там, где это нужно ABI/VM).
- минимальный runtime overhead.
- compile-time folding как оптимизационная дисциплина.
- минимум heap, предсказуемая память.

Не берём:

- template/meta сложность ради самой сложности;
- operator overloading как источник двусмысленности.

### ML/Haskell

- алгебраические типы и total pattern coverage.
- явная модель данных в семантике.

Не берём:

- ленивость и монады как обязательную модель языка.

### Lisp (позже)

- AST как данные;
- аккуратный, ограниченный macro-layer (поздний этап).

### Erlang/Elixir (для VectorOS позже)

- supervisor mindset;
- fault isolation;
- event-driven orchestration.

### ECS/Engine world

- `Entity = data`, `Law = systems`;
- deterministic tick;
- fixed timestep.

## Что остаётся уникальным EXO

- Квадро-логика (`N/F/T/S`) как первичная семантика.
- Merge как базовая философия вычислений.
- Runtime semantic profiles.
- EXO-T как тензорный IR-слой.
- Нативная интеграция с VectorOS/Transjector.
- Semantic VM поверх kernel-уровня.

## Архитектурные инварианты

- Любое новое расширение должно сохранять детерминизм compilation/runtime.
- Диагностики должны быть source-anchored (`SourceMark`) и воспроизводимы.
- no_std совместимость учитывается на этапе проектирования, а не постфактум.
- IR и bytecode версии эволюционируют только с обратимой миграцией формата.
