## Valkey Info

- [Valkey Site](https://valkey.io/)
- [Stream Guide](https://valkey.io/topics/streams-intro/)
    - [Command Reference](https://valkey.io/commands/)
    - [XADD](https://valkey.io/commands/xadd/)
    - [XREAD](https://valkey.io/commands/xread/)
    - [XGROUP CREATE](https://valkey.io/commands/xgroup-create/)
    - [XREADGROUP](https://valkey.io/commands/xreadgroup/)


### Starting with Docker

```sh
docker run --rm -p 6379:6379 valkey/valkey:8.1.2
```

<br />

[This repo](https://github.com/jeremychone-channel/rust-xp-valkey-stream)