# Next Steps

## Goal

仕様を分割し、examples を追加し、実装準備ができる状態にする。

## Tasks

1. mini-language-spec-v1.md を元に spec/001-core-language.md を作成
2. 名前解決仕様を spec/002-name-resolution.md に分離
3. 型推論方針を spec/003-typing.md に記述
4. 関数仕様を spec/004-functions.md に分離
5. examples/valid と examples/invalid を追加
6. invalid examples ごとの期待エラー文言を errors.md にまとめる

## Important constraints

- set は導入しない
- shadowing は禁止
- outer scope の更新は禁止
- 型注釈は必要最小限
- 再帰・相互再帰のみ注釈要件をやや厳しくする
