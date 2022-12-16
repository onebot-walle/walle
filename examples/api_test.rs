use tracing::info;
use walle::{
    matcher, on_command, ActionCaller, ActionCallerExt, MatcherHandler, Matchers, MatchersConfig,
    Session,
};
use walle_core::{
    event::GroupMessageEvent,
    prelude::Action,
    segment::{Mention, MessageExt},
    value_map,
};

#[tokio::main]
async fn main() {
    let matchers = Matchers::default()
        .add_matcher(recall_test_plugin().boxed())
        .add_matcher(mute_test().boxed())
        .add_matcher(unmute_test().boxed());
    // .add_matcher(member_test())
    // .add_matcher(forward_test_plugin());
    let walle = walle::new_walle(matchers, "debug");
    let joins = walle
        .start(
            walle::config::AppConfig::default(),
            MatchersConfig::default(),
            true,
        )
        .await
        .unwrap();
    // let walle = test_walle(matchers);
    // let joins = walle
    //     .start((), MatchersConfig::default(), true)
    //     .await
    //     .unwrap();
    // walle
    //     .handle_event(Event {
    //         id: "".to_owned(),
    //         time: 0.0,
    //         ty: "message".to_owned(),
    //         detail_type: "group".to_owned(),
    //         sub_type: "".to_owned(),
    //         extra: value_map! {
    //             "group_id": "",
    //             "user_id": "",
    //             "message": [{
    //                 "type": "text",
    //                 "data": {
    //                     "text": "呼叫全体干员"
    //                 }
    //             }]
    //         },
    //     })
    //     .await
    //     .ok();
    for join in joins {
        join.await.ok();
    }
}

fn recall_test_plugin() -> impl MatcherHandler {
    on_command!(Recall, "./recall");
    matcher(|Recall(_): Recall, s: Session| async move {
        info!(target: "api_test", "recall test");
        let m = s.reply("hello world").await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        s.delete_message(m.message_id).await.unwrap();
    })
}

fn mute_test() -> impl MatcherHandler {
    on_command!(Mute, "./mute");
    matcher(
        |Mute(segs): Mute, event: GroupMessageEvent, s: Session| async move {
            for seg in segs.extract::<Mention>() {
                s.call_action(Action {
                    action: "ban_group_member".to_string(),
                    params: value_map! {
                        "group_id": event.detail_type.group_id,
                        "user_id": seg.user_id,
                        "duration": 60,
                    },
                    selft: None,
                })
                .await
                .unwrap()
                .as_result()
                .unwrap();
            }
        },
    )
}

fn unmute_test() -> impl MatcherHandler {
    on_command!(Unmute, "./unmute");
    matcher(
        |Unmute(segs): Unmute, event: GroupMessageEvent, s: Session| async move {
            for seg in segs.extract::<Mention>() {
                s.call_action(Action {
                    action: "unban_group_member".to_string(),
                    params: value_map! {
                        "group_id": event.detail_type.group_id,
                        "user_id": seg.user_id,
                    },
                    selft: None,
                })
                .await
                .unwrap();
            }
        },
    )
}

// fn member_test() -> Matcher {
//     strip_prefix("./get_no_member")
//         .layer(handler_fn(|s: Session<Message, Group>| async move {
//             let r = s
//                 .call_action(walle_core::action::Action {
//                     action: "get_group_member_info".to_string(),
//                     selft: Some(s.event.ty.selft.clone()),
//                     params: value_map! {
//                         "group_id": s.event.detail_type.group_id,
//                         "user_id": "80000001"
//                     },
//                 })
//                 .await;
//             println!("{:?}", r);
//         }))
//         .boxed()
// }

// #[allow(dead_code)]
// fn flash_test_plugin() -> Matcher {
//     handler_fn(|s: Session<Message, MessageDeatilTypes>| async move {
//         let mut messages = s.event.message().into_iter();
//         while let Some(MessageSegment::Image { file_id, .. }) = messages.next() {
//             s.send(vec![MessageSegment::image_with_extend(
//                 file_id.to_string(),
//                 extended_map! {"flash":true},
//             )])
//             .await
//             .unwrap();
//         }
//     })
//     .boxed()
// }

// fn reply_test_plugin() -> Matcher {
//     handler_fn(|s: Session<Message, MessageDeatilTypes>| async move {
//         s.send(vec![MessageSegment::Reply {
//             message_id: s.event.message_id().to_string(),
//             user_id: s.event.user_id().to_string(),
//             extra: extended_map! {},
//         }])
//         .await
//         .unwrap();
//     })
//     .with_rule(strip_prefix("./reply"))
//     .boxed()
// }

// fn forward_test_plugin() -> Matcher {
//     handler_fn(|s: Session<Message, MessageDeatilTypes>| async move {
//         s.send(vec![
//             MsgSegment {
//                 ty: "node".to_string(),
//                 data: value_map! {
//                     "user_id": "80000000",
//                     "time": 1654654105527.0,
//                     "user_name": "mht",
//                     "message": [
//                         {
//                             "type": "text",
//                             "data": {
//                                 "text": "hello world"
//                             }
//                         }
//                     ]
//                 },
//             },
//             // MsgSegment {
//             //     ty: "text".to_string(),
//             //     data: value_map! {
//             //         "text": "this segemnt will break the nodes"
//             //     },
//             // },
//             MsgSegment {
//                 ty: "node".to_string(),
//                 data: value_map! {
//                     "user_id": "80000001",
//                     "time": 1654654190000.0,
//                     "user_name": "mht2",
//                     "message": [
//                         {
//                             "type": "node",
//                             "data": {
//                                 "user_id": "80000000",
//                                 "time": 1654654105527.0,
//                                 "user_name": "mht",
//                                 "message": [
//                                     {
//                                         "type": "text",
//                                         "data": {
//                                             "text": "hello world"
//                                         }
//                                     }
//                                 ]
//                             }
//                         }
//                     ]
//                 },
//             },
//         ])
//         .await
//         .unwrap();
//     })
//     .with_pre_handler(strip_prefix("./forward"))
//     .boxed()
// }

// fn url_image_plugin() -> MessageMatcher {
//     handler_fn(|s| async move {
//         let r = s
//             .bot
//             .upload_file_by_url(
//                 "test".to_string(),
//                 "https://avatars.githubusercontent.com/u/18395948?s=40&v=4".to_string(),
//                 HashMap::default(),
//                 None,
//             )
//             .await
//             .unwrap();
//         s.send(MessageSegment::image(r.data.file_id)).await.unwrap();
//     })
//     .with_pre_handler(strip_prefix("./url_image"))
//     .boxed()
// }

// fn delete_friend_plugin() -> MessageMatcher {
//     handler_fn(|s: Session<MessageContent>| async move {
//         let r = s
//             .bot
//             .call_action(
//                 walle::ext::WalleExtraAction::DeleteFriend(DeleteFriend {
//                     user_id: s.event.content.user_id,
//                 })
//                 .into(),
//             )
//             .await;
//         println!("{r:?}");
//     })
//     .with_pre_handler(strip_prefix("./delete_me"))
//     .boxed()
// }

// fn group_temp_plugin() -> MessageMatcher {
//     handler_fn(|s| async move {
//         let r = s
//             .bot
//             .send_message_ex(
//                 "private".to_string(),
//                 s.event.group_id().map(ToString::to_string),
//                 Some(s.event.user_id().to_string()),
//                 None,
//                 None,
//                 vec![MessageSegment::text("hello stranger".to_string())],
//                 extended_map! {
//                     "sub_type": "group_temp",
//                 },
//             )
//             .await;
//         println!("{r:?}");
//         tokio::time::sleep(std::time::Duration::from_secs(2)).await;
//         s.bot.delete_message(r.unwrap().data.message_id).await.ok();
//     })
//     .with_pre_handler(strip_prefix("./temp_me"))
//     .boxed()
// }

// fn voice_test_plugin() -> MessageMatcher {
//     handler_fn(|s: Session<MessageContent>| async move {
//         if let Ok(file) = s
//             .bot
//             .upload_file_by_path_ex(
//                 "name".to_string(),
//                 "E:/walle/test/test.mp3".to_string(),
//                 None,
//                 extended_map! {
//                     "file_type": "voice",
//                 },
//             )
//             .await
//         {
//             s.send(MessageSegment::voice(file.data.file_id))
//                 .await
//                 .unwrap();
//         }
//     })
//     .with_pre_handler(strip_prefix("./voice"))
//     .boxed()
// }
