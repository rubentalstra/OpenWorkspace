# Locales

Fluent (`.ftl`) translation resources, one folder per language. The seven
first-release languages (master plan §4): `en`, `de`, `fr`, `es`, `it`, `nl`, `pl`.

## Layout

- `languages.json5` — supported languages as `[code, endonym]`, in display order.
- `app/<lang>/main.ftl` — UI strings, loaded by the `i18n` crate's `leptos-fluent`
  provider and used through `i18n::tr!` / `i18n::move_tr!` in components.
- `email/<lang>/email.ftl` — notification strings, loaded by the `notify` crate for
  localized emails.

UI and email strings live in **separate subtrees** because the `i18n` crate enables
`leptos-fluent`'s compile-time `check_translations`, which is bidirectional: every
message under `app/` must be used by some `tr!`/`move_tr!` call and every key used in
a component must exist in all languages. Email strings are consumed by `notify`
(not by `tr!`), so they are kept out of `app/` to avoid false "unused message"
failures. Both subtrees still require a key to be present in **all** languages.
