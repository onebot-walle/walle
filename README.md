<div align=center>
  <img src="https://github.com/abrahum/Walle-core/blob/master/logo.png" height="300" width="300">

# Walle

![OneBot12](https://img.shields.io/badge/OneBot-12-black?logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAHAAAABwCAMAAADxPgR5AAAAGXRFWHRTb2Z0d2FyZQBBZG9iZSBJbWFnZVJlYWR5ccllPAAAAAxQTFRF////29vbr6+vAAAAk1hCcwAAAAR0Uk5T////AEAqqfQAAAKcSURBVHja7NrbctswDATQXfD//zlpO7FlmwAWIOnOtNaTM5JwDMa8E+PNFz7g3waJ24fviyDPgfhz8fHP39cBcBL9KoJbQUxjA2iYqHL3FAnvzhL4GtVNUcoSZe6eSHizBcK5LL7dBr2AUZlev1ARRHCljzRALIEog6H3U6bCIyqIZdAT0eBuJYaGiJaHSjmkYIZd+qSGWAQnIaz2OArVnX6vrItQvbhZJtVGB5qX9wKqCMkb9W7aexfCO/rwQRBzsDIsYx4AOz0nhAtWu7bqkEQBO0Pr+Ftjt5fFCUEbm0Sbgdu8WSgJ5NgH2iu46R/o1UcBXJsFusWF/QUaz3RwJMEgngfaGGdSxJkE/Yg4lOBryBiMwvAhZrVMUUvwqU7F05b5WLaUIN4M4hRocQQRnEedgsn7TZB3UCpRrIJwQfqvGwsg18EnI2uSVNC8t+0QmMXogvbPg/xk+Mnw/6kW/rraUlvqgmFreAA09xW5t0AFlHrQZ3CsgvZm0FbHNKyBmheBKIF2cCA8A600aHPmFtRB1XvMsJAiza7LpPog0UJwccKdzw8rdf8MyN2ePYF896LC5hTzdZqxb6VNXInaupARLDNBWgI8spq4T0Qb5H4vWfPmHo8OyB1ito+AysNNz0oglj1U955sjUN9d41LnrX2D/u7eRwxyOaOpfyevCWbTgDEoilsOnu7zsKhjRCsnD/QzhdkYLBLXjiK4f3UWmcx2M7PO21CKVTH84638NTplt6JIQH0ZwCNuiWAfvuLhdrcOYPVO9eW3A67l7hZtgaY9GZo9AFc6cryjoeFBIWeU+npnk/nLE0OxCHL1eQsc1IciehjpJv5mqCsjeopaH6r15/MrxNnVhu7tmcslay2gO2Z1QfcfX0JMACG41/u0RrI9QAAAABJRU5ErkJggg==)
<a href="https://github.com/abrahum/Walle/blob/master/license">
  <img src="https://img.shields.io/github/license/abrahum/Walle" alt="license">
</a>

</div>



A Onebot application SDK

Onebot 应用端开发框架，基于 [Walle-core](https://github.com/abrahum/walle-core)

## 最小实例

```rust
use walle::{builtin::Echo, Plugins, Walle};
use walle_core::AppConfig;

#[tokio::main]
async fn main() {
    let plugins = Plugins::new().add_message_plugin(Echo::new());
    let walle = Walle::new(AppConfig::default(), plugins);
    walle.start().await.unwrap();
}
```

## Plugin

Walle 以 Plugin 为各个独立组件作为开发，并提供一些常用可复用组件。

一个插件实例

```rust
pub struct Echo;

#[async_trait]
impl Handler<MessageContent> for Echo {
    async fn handle(&self, session: Session<MessageContent>) {
        let _ = session.send(session.event.message().clone()).await;
    }
}

impl Echo {
    pub fn new() -> Plugin<MessageContent> {
        Plugin::new("echo", "echo description", on_command("echo", Echo))
    }
}
```

使用闭包构建插件

```rust
pub fn echo2() -> Plugin<MessageContent> {
    Plugin::new(
        "echo2",
        "echo2 description",
        on_command(
            "echo2",
            handler_fn(|mut session: Session<MessageContent>| async move {
                let _ = session
                    .get("input message", std::time::Duration::from_secs(10))
                    .await;
                let _ = session.send(session.event.message().clone()).await;
            }),
        ),
    )
}
```

## 组件

Walle 使用了类似 Tower 的 Service Layer 模式，提供了一些组件，供开发者使用。

Handler 可以类比为 Service 组件，它是一个消息处理器，接收一个 Session，并返回一个消息。

Rule 和 PreHandler 可以类比为 Layer 组件，它们是一个消息规则匹配和预处理器，前者接收一个  &Session，后者接受一个 &mut Session。

需要注意的是所有以上所有 trait 和 Session 都具有一个泛型参数，用于约定其处理的消息类型。

Handler 最终需要被包裹为一个 Plugin，然后添加至 Plugins 中。

## Handler

```rust
pub struct Echo;

#[async_trait]
impl Handler<MessageContent> for Echo {
    async fn handle(&self, session: Session<MessageContent>) {
        let _ = session.send(session.event.message().clone()).await;
    }
}

// or

let echo2 = handler_fn(|mut session: Session<MessageContent>| async move {
    let _ = session
        .get("input message", std::time::Duration::from_secs(10))
        .await;
    let _ = session.send(session.event.message().clone()).await;
});
```

## Rule

```rust
pub struct UserIdChecker {
    pub user_id: String,
}

impl Rule<MessageContent> for UserIdChecker {
    fn rule(&self, session: &Session<MessageContent>) -> bool {
        session.event.user_id() == self.user_id
    }
}

pub fn user_id_check<S>(user_id: S) -> UserIdChecker
where
    S: ToString,
{
    UserIdChecker {
        user_id: user_id.to_string(),
    }
}

// or

pub fn start_with(word: &str) -> impl Rule<MessageContent> {
    let word = word.to_string();
    rule_fn(move |session: &Session<MessageContent>| {
        session.event.content.alt_message.starts_with(&word)
    })
}
```

## PreHandler

```rust
pub struct StripPrefix {
    pub prefix: String,
}

impl PreHandler<MessageContent> for StripPrefix {
    fn pre_handle(&self, session: &mut Session<MessageContent>) {
        let _ = session.event.content.alt_message.strip_prefix(&self.prefix);
    }
}

pub fn strip_prefix<S>(prefix: S) -> StripPrefix
where
    S: ToString,
{
    StripPrefix {
        prefix: prefix.to_string(),
    }
}
```

## Matcher
    
```rust
pub fn on_command<H>(command: &str, handler: H) -> impl Handler<MessageContent>
where
    H: Handler<MessageContent> + Sync,
{
    handler
        .rule(start_with(command))
        .pre_handle(strip_prefix(command))
}
```