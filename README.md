# slack-sdk

> **Work in progress.** A Slack Socket Mode SDK for Rust, currently developed alongside a sample bot. The SDK pieces live in `src/slack/` and will be extracted into a standalone library crate once the API settles.

This started as a learning project: I'm building a Slack client from the wire protocol up (HTTP Web API, Socket Mode WebSocket, acknowledgements, reconnects) instead of wrapping an existing crate.

## Status

**Working**

- Opens a Socket Mode connection (`apps.connections.open`, then a WebSocket to the returned URL)
- Parses incoming frames into typed envelopes (`hello`, `disconnect`, `events_api`, `interactive`, `slash_commands`), with an `Unknown` fallback for types not yet modeled
- Acknowledges every enveloped event immediately, before any other work
- Forwards events to the bot over a `tokio::sync::mpsc` channel
- Reconnects automatically when a session ends (Slack-requested disconnect or error)
- Posts messages via `chat.postMessage`

**Planned**

- Typed payload structs for common events (`app_mention`, `message`, slash commands), keeping the raw JSON as an escape hatch
- Reconnect backoff instead of a fixed one-second delay
- Move the event loop out of `main` into a handler module
- Redact tokens in `Debug` output
- Better error handling in config and response parsing
- Unit tests built from captured Slack frames
- Split into a Cargo workspace (SDK crate + bot crate) and add `tracing` in place of `println!`

## How it works

```
Slack  <--WebSocket-->  Runner  --mpsc channel-->  bot (event loop)
                          |                             |
                          +---- SlackClient (HTTP) <----+
```

1. **Bootstrap (HTTP):** call `apps.connections.open` with the app-level token to get a temporary `wss://` URL.
2. **Connect:** open the WebSocket; Slack sends `hello`.
3. **Receive:** read frames and deserialize each into an `IncomingEvent`.
4. **Acknowledge:** send an ack with the `envelope_id` right away (Slack expects it within about 3 seconds).
5. **Dispatch:** forward the event through the channel so slow handler work never blocks the read loop.
6. **Act:** the bot replies through `SlackClient` (for example `post_message`).
7. **Reconnect:** when the session ends, the runner waits a second and starts a new one.

## Project layout

```
src/
  main.rs          wires config, client, runner, and the receive loop
  config.rs        loads settings from environment variables
  models.rs        response types
  slack/
    http.rs        Web API error type and response envelope parsing
    client.rs      SlackClient (post_message, open_socket_url)
    socket.rs      Socket Mode wire types (IncomingEvent, AcknowledgeEvent, SocketError)
    runner.rs      connect / read / ack / forward / reconnect loop
```

## Setup

### 1. Create a Slack app

1. Create an app at <https://api.slack.com/apps>.
2. **Socket Mode:** enable it under *Settings → Socket Mode*.
3. **App-level token:** under *Basic Information → App-Level Tokens*, generate a token with the `connections:write` scope. It starts with `xapp-`.
4. **Bot scopes:** under *OAuth & Permissions*, add at least `chat:write`, plus `app_mentions:read` if you want mention events.
5. **Event subscriptions:** under *Event Subscriptions*, enable events and subscribe to the bot events you want (for example `app_mention`).
6. **Install** the app to your workspace. The *Bot User OAuth Token* starts with `xoxb-`. Reinstall whenever you change scopes or events.
7. Invite the bot to the channel you want to use (`/invite @yourbot`).

### 2. Configure environment

Copy the template and fill it in:

```sh
cp .env.example .env
```

| Variable          | Description                                                       |
| ----------------- | ----------------------------------------------------------------- |
| `SLACK_BOT_TOKEN` | Bot token (`xoxb-...`), used for Web API calls                    |
| `SLACK_APP_TOKEN` | App-level token (`xapp-...`), used to open the Socket Mode socket |
| `DEFAULT_CHANNEL` | Channel the bot posts its startup message to                      |
| `PORT`            | Optional, defaults to 8080 (currently unused in Socket Mode)      |

Never commit `.env`. It's listed in `.gitignore`.

### 3. Run

```sh
cargo run
```

You should see the bot post a startup message, and events print as you mention it in Slack.

## Notes

- One @-mention can arrive twice if you subscribe to both `message.*` and `app_mention`: once as each event type.
- The bot also receives its own messages as events; handlers should ignore events authored by the bot's own user id.

## License

TBD
