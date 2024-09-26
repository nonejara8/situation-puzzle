# ウミガメのスープBot

## 概要

このBotは、Discord上でウミガメのスープを楽しむためのBotです。
Botが参加者の質問に答えたり、回答の正誤チェックをするので、GM不在でゲームを進めることが可能です。

## How to use（WIP）
ローカルで立ち上げる場合

### サーバーにBotを導入する

・・・

### Botを作成する

Discord Developer PortalでBotを作成する。
https://discord.com/developers/applications

New Applicationを押し、任意のアプリケーション名を入力する。

Bot > Token
Reset Tokenを押し、表示されたトークンをコピーする。
このTokenは他の人には教えない。

OAuth2 > OAuth2 URL Generator
SCOPESにあるチェックボックスの中からbotを選択する。

BOT PERMISSIONSにあるチェックボックスの中から、Administratorを選択する。

一番下のGENERATED URLにあるCOPYボタンを押し、URLをコピーする。

1. .Secrets.toml.sampleを.Secrets.tomlにコピーして、各値を設定する
   - **DISCORD_TOKEN**
     - ddd
   - **DISCORD_GUILD_ID**
     - ユーザー設定（歯車アイコン） > 詳細設定 > 開発者モードをON
     - 上部のサーバー名を右クリック > サーバーIDをコピー
   - **OPENAI_API_KEY**


### Rustの実行環境を整える

- shuttleをインストールする
  - `curl -sSfL https://www.shuttle.rs/install | bash
`
  - https://docs.shuttle.rs/getting-started/installation
- `cargo shuttle run`でローカルでBotを起動する


