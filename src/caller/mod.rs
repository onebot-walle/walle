use std::{future::Future, pin::Pin, sync::Arc};
use walle_core::{
    action::Action,
    event::Event,
    prelude::{async_trait, GetSelfs, ToAction},
    resp::Resp,
    structs::Selft,
    util::{GetSelf, Value},
    ActionHandler, EventHandler, OneBot, WalleError, WalleResult,
};

use crate::{Bot, Session};

#[async_trait]
pub trait ActionCaller: GetSelfs + Sync {
    async fn call_action(&self, action: Action) -> WalleResult<Resp>;
    async fn get_bots(&self) -> Vec<Bot>;
}

#[async_trait]
impl<AH, EH> ActionCaller for Arc<OneBot<AH, EH>>
where
    AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
    EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
{
    async fn call_action(&self, action: Action) -> WalleResult<Resp> {
        self.handle_action(action).await
    }

    async fn get_bots(&self) -> Vec<Bot> {
        self.get_selfs()
            .await
            .into_iter()
            .map(|id| Bot {
                selft: id,
                caller: Arc::new(self.clone()),
            })
            .collect()
    }
}

impl GetSelfs for Bot {
    fn get_impl<'life0, 'life1, 'async_trait>(
        &'life0 self,
        selft: &'life1 Selft,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = String> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        self.caller.get_impl(selft)
    }
    fn get_selfs<'life0, 'async_trait>(
        &'life0 self,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Vec<Selft>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        self.caller.get_selfs()
    }
}

impl ActionCaller for Bot {
    fn call_action<'a, 't>(
        &'a self,
        action: Action,
    ) -> Pin<Box<dyn Future<Output = WalleResult<Resp>> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.caller.call_action(action)
    }
    fn get_bots<'a, 't>(&'a self) -> Pin<Box<dyn Future<Output = Vec<Bot>> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.caller.get_bots()
    }
}

impl<T, D, S, P, I> GetSelfs for Session<T, D, S, P, I> {
    fn get_impl<'life0, 'life1, 'async_trait>(
        &'life0 self,
        selft: &'life1 Selft,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = String> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        self.caller.get_impl(selft)
    }
    fn get_selfs<'life0, 'async_trait>(
        &'life0 self,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Vec<Selft>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        self.caller.get_selfs()
    }
}

impl<T, D, S, P, I> ActionCaller for Session<T, D, S, P, I>
where
    T: GetSelf + Sync,
    D: Sync,
    S: Sync,
    P: Sync,
    I: Sync,
{
    fn call_action<'a, 't>(
        &'a self,
        mut action: Action,
    ) -> Pin<Box<dyn Future<Output = WalleResult<Resp>> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        action.selft = Some(self.event.ty.get_self());
        self.caller.call_action(action)
    }
    fn get_bots<'a, 't>(&'a self) -> Pin<Box<dyn Future<Output = Vec<Bot>> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.caller.get_bots()
    }
}

#[async_trait]
pub trait ActionCallerExt: ActionCaller {
    async fn call<A, R>(&self, action: A) -> WalleResult<R>
    where
        A: ToAction + Send,
        R: TryFrom<Value, Error = WalleError>,
    {
        self.call_action(action.to_action())
            .await?
            .as_result()
            .map_err(WalleError::RespError)?
            .try_into()
    }
}

impl<T: ActionCaller> ActionCallerExt for T {}
