# muga

"Muga" is a Japanese term meaning "selflessness" or "transcendence of self," referring to a state of being beyond personal limitations or free from self-centered thinking.

This programming language incorporates the concept of muga, featuring a simple and intuitive syntax designed to immerse developers in coding while letting go of self-consciousness. Muga emphasizes both code aesthetics and efficiency, providing an environment where developers can freely express their creative ideas.

---

現状このリポジトリは、言語仕様の草案のみを含んでいる。今後、仕様を分割し、例を追加し、実装準備ができる状態にする予定である。

## 現在の方針

- let を使わない
- immutable がデフォルト
- mut で mutable
- `x = e` は現在スコープで未定義なら新規 immutable 束縛
- `x = e` は現在スコープの mutable 既存名なら更新
- `x = e` は現在スコープの immutable 既存名ならエラー
- shadowing 禁止
- outer scope の更新は禁止
- 型注釈は原則省略し、推論不能な場合のみ必須

## 仕様ドキュメント

- 正本: [mini-language-spec-v1.md](./mini-language-spec-v1.md)
- 分割仕様:
  - [spec/001-core-language.md](./spec/001-core-language.md)
  - [spec/002-name-resolution.md](./spec/002-name-resolution.md)
  - [spec/003-typing.md](./spec/003-typing.md)
  - [spec/004-functions.md](./spec/004-functions.md)
- エラー一覧: [errors.md](./errors.md)

## Examples

### Valid

- [examples/valid/001-basic-bindings.md](./examples/valid/001-basic-bindings.md)
- [examples/valid/002-read-from-outer-scope.md](./examples/valid/002-read-from-outer-scope.md)
- [examples/valid/003-local-mutable-loop.md](./examples/valid/003-local-mutable-loop.md)
- [examples/valid/004-inferred-parameter-type.md](./examples/valid/004-inferred-parameter-type.md)
- [examples/valid/005-recursive-function.md](./examples/valid/005-recursive-function.md)
- [examples/valid/006-mutual-recursion.md](./examples/valid/006-mutual-recursion.md)

### Invalid

- [examples/invalid/001-immutable-update.md](./examples/invalid/001-immutable-update.md)
- [examples/invalid/002-duplicate-mutable-binding.md](./examples/invalid/002-duplicate-mutable-binding.md)
- [examples/invalid/003-shadowing-in-block.md](./examples/invalid/003-shadowing-in-block.md)
- [examples/invalid/004-outer-scope-mutation.md](./examples/invalid/004-outer-scope-mutation.md)
- [examples/invalid/005-ambiguous-identity.md](./examples/invalid/005-ambiguous-identity.md)
- [examples/invalid/006-unannotated-recursion.md](./examples/invalid/006-unannotated-recursion.md)
- [examples/invalid/007-unannotated-mutual-recursion.md](./examples/invalid/007-unannotated-mutual-recursion.md)
