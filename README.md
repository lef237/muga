# muga

"Muga" is a Japanese term meaning "selflessness" or "transcendence of self," referring to a state of being beyond personal limitations or free from self-centered thinking.

This programming language incorporates the concept of muga, featuring a simple and intuitive syntax designed to immerse developers in coding while letting go of self-consciousness. Muga emphasizes both code aesthetics and efficiency, providing an environment where developers can freely express their creative ideas.

---

現状このリポジトリは、言語仕様の草案のみを含んでいる。今後、仕様を分割し、例を追加し、実装準備ができる状態にする予定である。

## 現在の方針

- let を使わない
- immutable がデフォルト
- mut で mutable
- `x = e` は未定義なら束縛、mutable 既存名なら更新
- shadowing 禁止
- 型注釈は原則省略し、推論不能な場合のみ必須
