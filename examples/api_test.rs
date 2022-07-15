use std::collections::HashMap;

use tracing::info;
use walle::{
    builtin::{echo2, strip_prefix},
    ext::DeleteFriend,
    handler_fn, new_walle, Matcher, MatcherHandlerExt, Matchers, MessageContent, MessageMatcher,
    Session,
};
use walle_core::{action::BotActionExt, extended_map, MessageSegment};

#[tokio::main]
async fn main() {
    let matchers = Matchers::default()
        .add_message_matcher(recall_test_plugin())
        // .add_message_matcher(flash_test_plugin())
        .add_message_matcher(reply_test_plugin())
        .add_message_matcher(forward_test_plugin())
        .add_message_matcher(forward_2077_plugin())
        .add_message_matcher(url_image_plugin())
        .add_message_matcher(delete_friend_plugin())
        .add_message_matcher(group_temp_plugin())
        .add_message_matcher(echo2().build())
        .add_message_matcher(voice_test_plugin());
    let walle = new_walle(
        walle_core::config::AppConfig {
            // http: {
            //     let mut map = HashMap::default();
            //     map.insert(
            //         "2431103771".to_string(),
            //         walle_core::config::HttpClient::default(),
            //     );
            //     map
            // },
            // http_webhook: vec![walle_core::config::HttpServer {
            //     host: std::net::IpAddr::from([127, 0, 0, 1]),
            //     port: 6701,
            //     ..Default::default()
            // }],
            // websocket: vec![walle_core::config::WebSocketClient::default()],
            // websocket_rev: vec![],
            ..Default::default()
        },
        matchers,
    );
    walle.run_block().await.unwrap();
}

fn recall_test_plugin() -> Matcher<MessageContent> {
    Matcher::new(
        "recall_test",
        "recall",
        handler_fn(|s: Session<MessageContent>| async move {
            info!(target: "api_test", "start api test");
            if let Ok(m) = s.send("hello world").await {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                s.bot.delete_message(m.data.message_id).await.unwrap();
            }
        })
        .with_pre_handler(strip_prefix("./recall"), true),
    )
}

#[allow(dead_code)]
fn flash_test_plugin() -> Matcher<MessageContent> {
    Matcher::new(
        "flash_test",
        "flash",
        handler_fn(|s| async move {
            let mut messages = s.event.message().into_iter();
            while let Some(MessageSegment::Image { file_id, .. }) = messages.next() {
                s.send(vec![MessageSegment::image_with_extend(
                    file_id.to_string(),
                    extended_map! {"flash":true},
                )])
                .await
                .unwrap();
            }
        }),
    )
}

fn reply_test_plugin() -> Matcher<MessageContent> {
    Matcher::new(
        "reply_test",
        "reply",
        handler_fn(|s: Session<MessageContent>| async move {
            s.send(vec![MessageSegment::Reply {
                message_id: s.event.message_id().to_string(),
                user_id: s.event.user_id().to_string(),
                extra: extended_map! {},
            }])
            .await
            .unwrap();
        })
        .with_pre_handler(strip_prefix("./reply"), true),
    )
}

fn forward_test_plugin() -> MessageMatcher {
    Matcher::new(
        "forward_test",
        "forward",
        handler_fn(|s| async move {
            s.send(MessageSegment::Custom {
                ty: "forward".to_string(),
                data: extended_map! {
                    "nodes": [
                        {
                            "type": "node",
                            "data": {
                                "user_id": "80000000",
                                "time": 1654654105527.0,
                                "user_name": "mht",
                                "message": [
                                    {
                                        "type": "text",
                                        "data": {
                                            "text": "hello world",
                                        },
                                    }
                                ]
                            }
                        }
                    ]
                },
            })
            .await
            .unwrap();
        })
        .with_pre_handler(strip_prefix("./forward"), true),
    )
}

fn forward_2077_plugin() -> MessageMatcher {
    Matcher::new(
        "forward_test",
        "forward",
        handler_fn(|s| async move {
            s.send(MessageSegment::Custom {
                ty: "forward".to_string(),
                data: extended_map! {
                    "nodes": [
                        {
                            "type": "node",
                            "data": {
                                "user_id": "80000000",
                                "time": 3376692505000.0,
                                "user_name": "赛博朋克2077官方",
                                "message": [
                                    {
                                        "type": "text",
                                        "data": {
                                            "text": "还没有出哦",
                                        },
                                    }
                                ]
                            }
                        },
                        {
                            "type": "node",
                            "data": {
                                "user_id": "80000000",
                                "time": 3376692506000.0,
                                "user_name": "赛博朋克2077官方",
                                "message": [
                                    {
                                        "type": "node",
                                        "data": {
                                            "user_id": "80000000",
                                            "time": 3376692505000.0,
                                            "user_name": "赛博朋克2077官方",
                                            "message": [
                                                {
                                                    "type": "text",
                                                    "data": {
                                                        "text": "还没有出哦",
                                                    },
                                                }
                                            ]
                                        }
                                    }
                                ]
                            }
                        }
                    ]
                },
            })
            .await
            .unwrap();
        })
        .with_pre_handler(strip_prefix("2077dlc出了吗"), true),
    )
}

fn url_image_plugin() -> MessageMatcher {
    Matcher::new(
        "url_image",
        "url_image",
        handler_fn(|s| async move {
            let r = s
                .bot
                .upload_file_by_url(
                    "test".to_string(),
                    "https://avatars.githubusercontent.com/u/18395948?s=40&v=4".to_string(),
                    HashMap::default(),
                    None,
                )
                .await
                .unwrap();
            s.send(MessageSegment::image(r.data.file_id)).await.unwrap();
        })
        .with_pre_handler(strip_prefix("./url_image"), true),
    )
}

fn delete_friend_plugin() -> MessageMatcher {
    Matcher::new(
        "delete_friend_test",
        "delete_friend",
        handler_fn(|s: Session<MessageContent>| async move {
            let r = s
                .bot
                .call_action(
                    walle::ext::WalleExtraAction::DeleteFriend(DeleteFriend {
                        user_id: s.event.content.user_id,
                    })
                    .into(),
                )
                .await;
            println!("{r:?}");
        })
        .with_pre_handler(strip_prefix("./delete_me"), true),
    )
}

fn group_temp_plugin() -> MessageMatcher {
    Matcher::new(
        "group_temp_test",
        "group_temp",
        handler_fn(|s| async move {
            let r = s
                .bot
                .send_message_ex(
                    "private".to_string(),
                    s.event.group_id().map(ToString::to_string),
                    Some(s.event.user_id().to_string()),
                    None,
                    None,
                    vec![MessageSegment::text("hello stranger".to_string())],
                    extended_map! {
                        "sub_type": "group_temp",
                    },
                )
                .await;
            println!("{r:?}");
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            s.bot.delete_message(r.unwrap().data.message_id).await.ok();
        })
        .with_pre_handler(strip_prefix("./temp_me"), true),
    )
}

fn voice_test_plugin() -> MessageMatcher {
    Matcher::new(
        "voice_test",
        "voice",
        handler_fn(|s: Session<MessageContent>| async move {
            if let Ok(file) = s
                .bot
                .upload_file_by_path_ex(
                    "name".to_string(),
                    "E:/walle/test/test.mp3".to_string(),
                    None,
                    extended_map! {
                        "file_type": "voice",
                    },
                )
                .await
            {
                s.send(MessageSegment::voice(file.data.file_id))
                    .await
                    .unwrap();
            }
        })
        .with_pre_handler(strip_prefix("./voice"), true),
    )
}
