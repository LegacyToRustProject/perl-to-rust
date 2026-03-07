# perl-to-rust OSS変換テスト結果

実施日: 2026-03-08
作業者: #08

---

## 環境

| 項目 | 値 |
|------|-----|
| Perl バージョン | v5.38.2 |
| Rust エディション | 2021 |
| perl-to-rust バージョン | 0.1.0 (build: release) |
| LLM プロバイダー | mock (ANTHROPIC_API_KEY 未設定のためClaude API不使用) |
| テスト方式 | analyze コマンド + 手動変換 + cargo check/test |

---

## サマリー

| プロジェクト | Perl Ver | 行数 | 解析完走 | 手動変換 cargo check | 出力一致 | TODO数 |
|---|---|---|---|---|---|---|
| List::Util (minimal) | 5.x | 50 | ✅ | ✅ (6/6テスト通過) | ✅ | 0 |
| Scalar-List-Utils-1.63 | 5.x | 4177 | ✅ | N/A (全体変換不要) | N/A | — |
| Getopt::Long-2.57 | 5.x | 3947 | ✅ | N/A (analyze のみ) | N/A | — |
| regex-test.pl | 5.x | 21 | ✅ | ✅ (Rust出力完全一致) | ✅ | 0 |

**注**: Claude API キーが未設定のため `convert --llm claude` は実施不可。`convert --llm mock` でパイプラインのE2E動作を確認、手動Rust変換でコンパイル可能性を検証した。

---

## 解析結果詳細

### List::Util (Scalar-List-Utils-1.63)

```
Total files: 40 | Total lines: 4177
Regex patterns: 21 | OOP modules: 2 | Complexity: Medium
```

主要モジュール:
- `List::Util` (836行) — `sum`, `min`, `max`, `first`, `any`, `all`, `reduce`, `product`, `uniq` など
- `Scalar::Util` (377行) — `blessed`, `reftype`, `looks_like_number`, `weaken` など
- `Sub::Util` (153行) — `set_subname`, `prototype` など

未マッピングのCPAN依存: `List::Util::XS`, `Sub::Util`, `Tie::Scalar`, `Math::BigInt`, `Symbol`
→ 今回のマッピング追加で `List::Util::XS`, `Sub::Util` を追加済み

### Getopt::Long-2.57

```
Total files: 16 | Total lines: 3947
Regex patterns: 52 | OOP modules: 2 | Complexity: Medium
```

変換の焦点: 52個の正規表現パターン（`=~` 演算子使用）がある高正規表現密度プロジェクト。
- `GetOptions("verbose!" => \$verbose)` → `clap::Parser` derive が対応
- 参照引数 `\$variable` → `&mut T` 変換が必要

### regex-test.pl (正規表現変換ベンチマーク)

```
Total files: 1 | Total lines: 21
Regex patterns: 4 | Complexity: Low
```

---

## 正規表現変換精度

| テストケース | Perl出力 | Rust出力 | 一致 |
|---|---|---|---|
| 日付キャプチャ `/^(\d{4})-(\d{2})-(\d{2})$/` | `year=2024, month=03, day=08` | `year=2024, month=03, day=08` | ✅ |
| グローバル置換 `s/Hello/Goodbye/g` | `Goodbye World Goodbye` | `Goodbye World Goodbye` | ✅ |
| 名前付きキャプチャ `(?P<level>\w+)` | `level=ERROR, line=42` | `level=ERROR, line=42` | ✅ |
| メッセージキャプチャ | `message=Connection failed` | `message=Connection failed` | ✅ |

### 変換パターンの詳細

| Perlパターン | Rust変換 | ノート |
|---|---|---|
| `/pattern/` | `Regex::new(r"pattern").unwrap()` | LazyLock化推奨 |
| `=~ /^(\d{4})-(\d{2})-(\d{2})$/` | `re.captures(s)` | `&caps[1]` でグループアクセス |
| `s/Hello/Goodbye/g` | `re.replace_all(s, "Goodbye")` | 非破壊的、`Cow<str>` を返す |
| `(?P<name>...)` | `(?P<name>...)` | **完全互換** — 変換不要 |
| `$+{name}` | `&caps["name"]` | Perlの `%+` ハッシュ → 名前付きキャプチャ |
| `$1, $2` | `&caps[1], &caps[2]` | 番号付きキャプチャ |
| `/pattern/i` | `(?i)pattern` | インラインフラグ変換 |
| `/pattern/g` | `re.find_iter(s)` or `.replace_all()` | globalフラグ → イテレータ |
| `=~ tr/a-z/A-Z/` | `.to_uppercase()` / `chars().map()` | 文字変換 |
| `(?<=prefix)` | `fancy_regex::Regex` | lookbehind → fancy-regex必須 |

### 正規表現エンジン選択ロジック（実装確認済み）

```
regex crate:    標準的なパターン（デフォルト）
fancy_regex:    後方参照 (\1, \2)、lookbehind ((?<=, (?<!))、
                条件パターン (?(DEFINE)、再帰 (?1), (?R) を含む場合
```

---

## List::Util 手動変換結果

`output/list-util-manual/` に変換コード生成 → **cargo test 6/6 通過**

### 変換マッピング

| Perl | Rust | 備考 |
|------|------|------|
| `sub sum { $t += $_ for @_ }` | `nums.iter().sum()` | `std::iter::Sum` |
| `sub min { ... $m = $_ if $_ < $m }` | `nums.iter().cloned().reduce(f64::min)` | `Option<f64>` を返す |
| `sub max { ... $m = $_ if $_ > $m }` | `nums.iter().cloned().reduce(f64::max)` | `Option<f64>` を返す |
| `sub first (&@) { ... return $_ if $code->($_) }` | `slice.iter().find(\|x\| predicate(x))` | `Option<&T>` |
| `sub any (&@) { !!grep { $code->($_) } @_ }` | `slice.iter().any(predicate)` | `bool` |
| `sub all (&@) { !grep { !$code->($_) } @_ }` | `slice.iter().all(predicate)` | `bool` |
| `grep { $_ > 0 } @list` | `.iter().filter(\|&&x\| x > 0)` | closureのパターン注意 |
| `map { $_ * 2 } @list` | `.iter().map(\|&x\| x * 2)` | |
| `$_` 暗黙変数 | 明示的クロージャ引数 | `\|x\|` or `\|&x\|` |
| `@_` 引数配列 | 関数パラメータ `&[T]` | スライス参照 |

### 発見したコンパイルエラーパターン

LLM変換で頻出しそうなエラー:

```rust
// ❌ よくある誤変換: 二重参照クロージャ
any(&nums, |&&x| x > 5.0)  // E0308 mismatched types

// ✅ 正しい変換
any(&nums, |&x| x > 5.0)   // &[f64] の要素は &f64

// ❌ sum0 の -0.0 表示問題
println!("{}", 0.0_f64 * -1.0)  // → -0

// ✅ 修正
println!("{}", 0.0_f64.abs())   // → 0
```

---

## CPANマッピング追加分

今回のOSS検証で **38件** の新規マッピングを追加（旧: 83件 → 新: 121件）。

| カテゴリ | 追加数 | 代表例 |
|---------|--------|--------|
| 標準ライブラリ / OS | 5 | `Cwd`→`std::env::current_dir`, `Errno`→`std::io::ErrorKind` |
| IO | 4 | `IO::File`→`std::fs::File`, `IO::String`→`std::io::Cursor` |
| Sub / Symbol | 4 | `Sub::Util`, `List::Util::XS`, `Hash::Util` |
| テスト | 5 | `Test::MockObject`→`mockall`, `Test::Fatal` |
| 数学 | 5 | `Math::BigInt`→`num-bigint`, POSIX数学関数 |
| 文字列 | 5 | `Text::Wrap`→`textwrap`, `Unicode::Normalize` |
| ターミナル | 3 | `Term::ANSIColor`→`colored`, `Term::ReadLine`→`rustyline` |
| シリアライズ | 4 | `Data::MessagePack`→`rmp-serde`, `HTML::Entities`→`html-escape` |
| Getopt拡張 | 3 | `Getopt::Long::Parser`→`clap::Command`, `Pod::Usage` |

---

## 未対応パターン一覧

| Perlパターン | 出現頻度 | 対応難度 | 対応方針 |
|---|---|---|---|
| `$_` 暗黙変数 | 非常に高 | 低 | 明示的クロージャ引数に変換（実装済み） |
| `@_` 引数配列 | 非常に高 | 低 | `&[T]` スライスに変換（実装済み） |
| ヒアドキュメント `<<EOF` | 高 | 低 | `r#"..."#` 生文字列リテラル（実装済み） |
| `grep { } @list` | 高 | 低 | `.iter().filter()` に変換（実装済み） |
| `map { } @list` | 高 | 低 | `.iter().map()` に変換（実装済み） |
| Perl OOP (`bless`) | 高 | 中 | `struct + impl` に変換（実装済み） |
| `eval { }` (例外捕捉) | 高 | 中 | `Result<T, E>` + `std::panic::catch_unwind` |
| `wantarray` (コンテキスト判定) | 中 | 高 | 関数を2つに分割 or TODOコメント（スプリント指示でスキップ可） |
| `local $_` | 中 | 中 | スコープ付き変数に変換 |
| `sub first (&@)` プロトタイプ | 中 | 中 | 高階関数 `fn<F: Fn(&T) -> bool>` に変換 |
| `sprintf "%s", $val` | 中 | 低 | `format!("{}", val)` |
| `(my $x = $y) =~ s/a/b/` | 中 | 低 | `let x = re.replace_all(&y, "b")` |
| `AUTOLOAD` | 低 | 非常に高 | TODOコメントを生成 |
| `tie` / `untie` | 低 | 非常に高 | TODOコメントを生成 |
| XS モジュール (C拡張) | 低 | 非常に高 | 対応するRustクレートに置換 or TODOコメント |
| 正規表現修飾子 `/e` (eval) | 低 | 高 | 警告 + 手動変換必須のコメント |
| `\$variable` 参照渡し | 高 | 低 | `&mut T` 参照に変換 |

---

## 変換エンジン改善提案

### 優先度: 高

**1. クロージャのパターン生成改善**

現在の問題: `|&&x|` と `|&x|` を誤って生成する場合がある。

```rust
// 現在の誤生成パターン
slice.iter().any(|&&x| x > 5.0)  // コンパイルエラー

// 正しいパターン — スライス &[T] のイテレータは &T を返す
slice.iter().any(|&x| x > 5.0)
```

改善案: `rust-generator/src/prompt.rs` のプロンプトに以下を追加:
```
IMPORTANT: When iterating over &[T], the iterator yields &T, not &&T.
Use |&x| not |&&x| in closures.
```

**2. `sum0` の浮動小数点表示**

```rust
// 0.0_f64 が -0 と表示される場合の対策
format!("{}", 0.0_f64)  // "0" ✓
format!("{}", -0.0_f64) // "-0" ← 注意
```

改善案: `sum0` / `product` の空リスト処理に `+ 0.0` を加える。

### 優先度: 中

**3. `wantarray` 変換戦略**

指示書通り、関数を2つに分割するパターンをプロンプトに明示:

```perl
# Perl: wantarray 対応関数
sub context_sensitive {
    return wantarray ? (1, 2, 3) : "scalar";
}
```

```rust
// Rust: 2関数に分割
fn context_sensitive_list() -> Vec<i32> { vec![1, 2, 3] }
fn context_sensitive_scalar() -> &'static str { "scalar" }
// TODO: Perl wantarray had context-sensitive behavior - split into two functions
```

**4. `eval {}` → `std::panic::catch_unwind` の自動変換**

```perl
# Perl
eval { some_operation() };
if ($@) { handle_error($@) }
```

```rust
// Rust
match std::panic::catch_unwind(|| some_operation()) {
    Ok(result) => result,
    Err(e) => handle_error(e),
}
// または anyhow::Result を使う場合:
some_operation()?;
```

### 優先度: 低

**5. `AUTOLOAD` への対処**

出現頻度は低いが、変換不可能なパターン。現状の「TODOコメント生成」が適切。
改善案: TODOコメントに「Rustでは proc macro や enum dispatch が代替手段」のリンクを追加。

---

## pipeline E2E テスト結果 (mock LLM)

```
$ perl-to-rust convert ./test-projects/list-util-minimal/ \
    --output ./output/list-util/ --llm mock

INFO Project analyzed modules=0 scripts=1
INFO Starting conversion...
INFO Converting script script=list_util.pl
INFO Written file=list_util.rs

Conversion complete! Output: ./output/list-util/
```

パイプライン動作確認:
- [x] `analyze` フェーズ: Perlファイル解析、サブルーチン抽出、CPAN依存収集
- [x] `convert` コマンド: Cargo.toml スキャフォールド生成
- [x] ファイル出力: `list_util.rs` + `Cargo.toml` 生成
- [ ] LLM変換: Claude APIキーが必要（mock返答は `// No mock response available`）

---

## cargo test / clippy 結果

```
$ cargo test --workspace
test result: ok. 17 passed; 0 failed (cli)
test result: ok. 10 passed; 0 failed (perl-parser)
test result: ok. 9 passed; 0 failed  (rust-generator)
Total: 36 tests, 0 failures

$ cargo clippy --workspace -- -D warnings
(警告なし)
```

---

## 結論

### 動作確認済み
- **解析パイプライン**: 40ファイル/4177行のList::Utilを正常解析
- **正規表現変換**: 4/4テストケースでPerl出力と完全一致
- **List::Util手動変換**: cargo check通過、6/6テスト通過、Perl出力一致
- **CPANマッピング**: 83件 → 121件 (+38件)
- **ワークスペーステスト**: 36テスト全通過

### 次フェーズ課題（Beta向け）
1. Claude APIキー設定後に実際のLLM変換をE2Eテスト
2. Getopt::Long の実変換 (52個の正規表現パターン処理が鍵)
3. CGI.pm → Axum 変換パターン確立 (~4000行)
4. クロージャパターン (`|&x|` vs `|&&x|`) の自動修正

---

*レポート生成: 作業者 #08 — 2026-03-08*
