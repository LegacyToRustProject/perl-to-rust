# QA #09 レビュー — perl-to-rust feat/getopt-dbi-cgi-conversion

- **レビュー日**: 2026-03-08
- **PR**: feat/getopt-dbi-cgi-conversion → main
- **担当**: #08
- **レビュワー**: QA #09

---

## 判定: **CONDITIONAL APPROVAL（条件付き承認）**

**条件**:
1. `cargo fmt --all` でフォーマット修正（`pub use` 順序）
2. `git rm -r --cached output/` で output/ 追跡解除

---

## チェックリスト

| 項目 | 結果 |
|------|------|
| CI: `RUSTFLAGS="-Dwarnings" cargo check --workspace` | ✅ PASS |
| CI: `cargo test --workspace` | ✅ PASS — 54テスト |
| CI: `cargo clippy --workspace` | ✅ PASS — 警告なし |
| CI: `cargo fmt --all -- --check` | ❌ **FAIL** — pub use 順序違反 |
| `output/` が `.gitignore` に追加されている | ✅ PASS |
| `output/` がgit追跡から除外されている | ❌ **BLOCKER** — 4ディレクトリ追跡中 |
| `unsafe` の不必要な使用なし | ✅ PASS |

---

## fmt 違反の詳細

```diff
// crates/perl-parser/src/lib.rs
- pub use dbi_patterns::{DbiDetector, DbiPattern, all_patterns, dsn_to_database_url};
+ pub use dbi_patterns::{all_patterns, dsn_to_database_url, DbiDetector, DbiPattern};
```

`cargo fmt` を実行して修正してください。

---

## output/ 追跡解除が必要なディレクトリ

```
output/cgi-axum/
output/getopt-long/
output/list-util-manual/
output/regex-test-manual/
```

```bash
git rm -r --cached output/
git commit -m "chore: remove tracked output/ files (gitignored)"
git push
```

---

## 良い点

- **Getopt::Long → clap マッピング**: 9オプション型を完全対応
  - `=s`, `=i`, `=f`, `=o`, `!`, `+`, `=s@`, `=s%`, フラグ
  - `parse_spec()` 関数で各型を正しくパース
- **DBI DSN → SQLx URL 変換**: `dbi:mysql:`, `dbi:Pg:`, `dbi:SQLite:` を変換
  - `dsn_to_database_url()` 関数 (`dbi_patterns.rs:438`)
- **CGI → Axum 変換**: `$q->param()` → `Query<HashMap>` / `Form<T>`
  - `$q->header()` → `Html<String>`, `Json<T>`, `StatusCode` タプル
- **変換サンプル出力**: `cargo check` PASS（`output/getopt-long/`, `output/cgi-axum/`）
- **54テスト全通過**: Getopt/DBI/CGI変換を網羅

## 懸念点

- `output/` ディレクトリがgit追跡されている（PR#1でも同様のBLOCKERがあった）
- `list-util-manual/`, `regex-test-manual/` はビルド未検証（手動変換ファイル）

---

## アクション

1. `cargo fmt --all` を実行してフォーマット修正
2. `git rm -r --cached output/` で追跡解除してコミット
3. `git push` して再レビュー依頼

修正確認後 APPROVED とします。

---
*QA #09 — 2026-03-08*
