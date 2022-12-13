use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use walle::builtin::{mention_user, start_with, trim};
use walle::walle_core::event::GroupMessageEvent;
use walle::walle_core::util::ValueMapExt;
use walle::walle_core::WalleResult;
use walle::{matcher, on_command, MatcherHandler, PreHandler, Session};

on_command!(Roulette, Start => "轮盘赌", Shot => "shot");

pub struct RouletteMatcher(Mutex<HashMap<String, Vec<(String, String, u8, u8, u8)>>>);

impl RouletteMatcher {
    pub async fn roalette(
        &self,
        ro: Roulette,
        event: GroupMessageEvent,
        mut s: Session,
    ) -> WalleResult<()> {
        match ro {
            Roulette::Start(_seg) => {
                s.get_with_pre_handler(
                    "开始轮盘赌局，哪位英雄接受挑战？",
                    mention_user(event.ty.user_id.clone())
                        .with(trim(true))
                        .with_rule(start_with("接受赌局")),
                    false,
                )
                .await?;
                self.0
                    .lock()
                    .await
                    .entry(event.detail_type.group_id)
                    .or_default()
                    .push((
                        event.ty.user_id,
                        s.event.extra.get_downcast("user_id")?,
                        0,
                        6,
                        {
                            use rand::Rng;
                            let mut rng = rand::thread_rng();
                            rng.gen_range(0..6)
                        },
                    ));
                s.reply("开始赌局，发送shot发射子弹").await?;
            }
            Roulette::Shot(_seg) => {
                let mut locked = self.0.lock().await;
                if let Some(v) = locked.get_mut(&event.detail_type.group_id) {
                    let mut need_remove = None;
                    for (index, (a, b, count, all, shot)) in v.into_iter().enumerate() {
                        if a == event.ty.user_id.as_str() || b == event.ty.user_id.as_str() {
                            if count == shot {
                                s.reply("嘣！正中靶心！").await?;
                                need_remove = Some(index);
                            } else {
                                *count += 1;
                                s.reply(format!("咔哒，逃过一劫\n剩余子弹{}发", *all - *count))
                                    .await?;
                            }
                        }
                    }
                    if let Some(index) = need_remove {
                        v.remove(index);
                        if v.is_empty() {
                            drop(v);
                            locked.remove(&event.detail_type.group_id);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

matcher!(
    failable: RouletteMatcher,
    roalette,
    ro: Roulette,
    event: GroupMessageEvent
);

pub fn roulette() -> impl MatcherHandler {
    Arc::new(RouletteMatcher(Mutex::default()))
}

#[tokio::test]
async fn t() {
    let matchers = walle::Matchers::default().add_matcher(roulette().boxed());
    let walle = walle::new_walle(matchers);
    for join in walle
        .start(
            walle::config::AppConfig::default(),
            walle::MatchersConfig::default(),
            true,
        )
        .await
        .unwrap()
    {
        join.await.ok();
    }
}
