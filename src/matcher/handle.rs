use crate::{ActionCaller, Matcher, MatchersConfig, ReplyAbleSession, TempMatchers};

use super::{LayeredPreHandler, LayeredRule, PreHandler, Rule, Session};
use std::{future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;
use walle_core::{
    event::{DetailTypeDeclare, Event, ImplDeclare, PlatformDeclare, SubTypeDeclare, TypeDeclare},
    prelude::WalleError,
    segment::IntoMessage,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Signal {
    MatchAndBlock,
    Matched,
    NotMatch,
}

impl core::ops::Add for Signal {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (_, Self::MatchAndBlock) | (Self::MatchAndBlock, _) => Self::MatchAndBlock,
            (_, Self::Matched) | (Self::Matched, _) => Self::Matched,
            _ => Self::NotMatch,
        }
    }
}

pub trait RawMatcherHandler {
    fn call(
        &self,
        event: Event,
        config: &Arc<MatchersConfig>,
        caller: &Arc<dyn ActionCaller + Send + 'static>,
        temp: &TempMatchers,
    ) -> Signal;
}

pub(crate) struct BoxedHandler<H, T, D, S, P, I>(
    pub Arc<H>,
    pub std::marker::PhantomData<(T, D, S, P, I)>,
);

impl<H, T, D, S, P, I> RawMatcherHandler for BoxedHandler<H, T, D, S, P, I>
where
    H: MatcherHandler<T, D, S, P, I> + Send + 'static,
    T: for<'a> TryFrom<&'a mut Event, Error = WalleError>
        + std::fmt::Debug
        + TypeDeclare
        + Send
        + 'static,
    D: for<'a> TryFrom<&'a mut Event, Error = WalleError>
        + std::fmt::Debug
        + DetailTypeDeclare
        + Send
        + 'static,
    S: for<'a> TryFrom<&'a mut Event, Error = WalleError>
        + std::fmt::Debug
        + SubTypeDeclare
        + Send
        + 'static,
    I: for<'a> TryFrom<&'a mut Event, Error = WalleError>
        + std::fmt::Debug
        + ImplDeclare
        + Send
        + 'static,
    P: for<'a> TryFrom<&'a mut Event, Error = WalleError>
        + std::fmt::Debug
        + PlatformDeclare
        + Send
        + 'static,
{
    fn call(
        &self,
        event: Event,
        config: &Arc<MatchersConfig>,
        caller: &Arc<dyn ActionCaller + Send + 'static>,
        temp: &TempMatchers,
    ) -> Signal {
        match event.try_into() {
            Ok(event) => {
                let mut session = Session::<T, D, S, P, I>::new(
                    event,
                    caller.clone(),
                    config.clone(),
                    temp.clone(),
                );
                let signal = self.0.pre_handle(&mut session);
                println!("{}:{:?}", session.event.detail_type.detail_type(), signal);
                let handler = self.0.clone();
                if signal != Signal::NotMatch {
                    tokio::spawn(async move {
                        handler.handle(session).await;
                    });
                }
                signal
            }
            Err(_) => Signal::NotMatch,
        }
    }
}

/// Matcher Handler
#[async_trait]
pub trait MatcherHandler<T = (), D = (), S = (), P = (), I = ()>: Sync {
    fn pre_handle(&self, _session: &mut Session<T, D, S, P, I>) -> Signal {
        Signal::NotMatch
    }
    async fn handle(&self, session: Session<T, D, S, P, I>);
}

pub trait MatcherHandlerExt<T = (), D = (), S = (), P = (), I = ()>:
    MatcherHandler<T, D, S, P, I>
{
    fn with_rule<R>(self, rule: R) -> LayeredRule<R, Self>
    where
        Self: Sized,
        R: Rule<T, D, S, P, I>,
    {
        LayeredRule {
            rule,
            handler: self,
            before: false,
        }
    }
    fn with_pre_handler<PR>(self, pre: PR) -> LayeredPreHandler<PR, Self>
    where
        Self: Sized,
        PR: PreHandler<T, D, S, P, I>,
    {
        LayeredPreHandler {
            pre,
            handler: self,
            before: false,
        }
    }
    fn with_extra_handler<H>(self, handler: H) -> LayeredHandler<H, Self>
    where
        Self: Sized,
        H: ExtraHandler<T, D, S, P, I>,
    {
        LayeredHandler {
            extra: handler,
            handler: self,
        }
    }
    fn boxed(self) -> Matcher
    where
        Self: Send + Sync + Sized + 'static,
        T: for<'a> TryFrom<&'a mut Event, Error = WalleError>
            + std::fmt::Debug
            + TypeDeclare
            + Send
            + Sync
            + 'static,
        D: for<'a> TryFrom<&'a mut Event, Error = WalleError>
            + std::fmt::Debug
            + DetailTypeDeclare
            + Send
            + Sync
            + 'static,
        S: for<'a> TryFrom<&'a mut Event, Error = WalleError>
            + std::fmt::Debug
            + SubTypeDeclare
            + Send
            + Sync
            + 'static,
        P: for<'a> TryFrom<&'a mut Event, Error = WalleError>
            + std::fmt::Debug
            + PlatformDeclare
            + Send
            + Sync
            + 'static,
        I: for<'a> TryFrom<&'a mut Event, Error = WalleError>
            + std::fmt::Debug
            + ImplDeclare
            + Send
            + Sync
            + 'static,
    {
        Box::new(BoxedHandler(
            Arc::new(self),
            std::marker::PhantomData::default(),
        ))
    }
}

impl<T, D, S, P, I, H: MatcherHandler<T, D, S, P, I>> MatcherHandlerExt<T, D, S, P, I> for H {}

pub struct HandlerFn<H>(H);

pub fn handler_fn<H, T, D, S, P, I, Fut>(inner: H) -> HandlerFn<H>
where
    H: Fn(Session<T, D, S, P, I>) -> Fut + Send,
    Fut: Future<Output = ()> + Send,
    T: Send + 'static,
    D: Send + 'static,
    S: Send + 'static,
    P: Send + 'static,
    I: Send + 'static,
{
    HandlerFn(inner)
}

impl<T, D, S, P, I, H, Fut> MatcherHandler<T, D, S, P, I> for HandlerFn<H>
where
    H: Fn(Session<T, D, S, P, I>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
    T: Sync + Send + 'static,
    D: Sync + Send + 'static,
    S: Sync + Send + 'static,
    P: Sync + Send + 'static,
    I: Sync + Send + 'static,
{
    fn handle<'a, 'b>(
        &'a self,
        session: Session<T, D, S, P, I>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
        Self: 'b,
    {
        Box::pin(self.0(session))
    }
}

pub struct MayFailHandlerFn<H, M>(H, std::marker::PhantomData<M>);

pub fn may_fail_handler_fn<H, T, D, S, P, I, Fut, M>(inner: H) -> MayFailHandlerFn<H, M>
where
    H: Fn(Session<T, D, S, P, I>) -> Fut + Send + Sync,
    Fut: Future<Output = Result<(), M>> + Send,
    T: Clone + Send + Sync + 'static,
    D: Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    P: Clone + Send + Sync + 'static,
    I: Clone + Send + Sync + 'static,
    M: IntoMessage + Send + Sync + 'static,
    Session<T, D, S, P, I>: ReplyAbleSession,
{
    MayFailHandlerFn(inner, std::marker::PhantomData::default())
}

#[async_trait]
impl<T, D, S, P, I, H, Fut, M> MatcherHandler<T, D, S, P, I> for MayFailHandlerFn<H, M>
where
    H: Fn(Session<T, D, S, P, I>) -> Fut + Send + Sync,
    Fut: Future<Output = Result<(), M>> + Send,
    T: Clone + Send + Sync + 'static,
    D: Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    P: Clone + Send + Sync + 'static,
    I: Clone + Send + Sync + 'static,
    M: IntoMessage + Send + Sync + 'static,
    Session<T, D, S, P, I>: ReplyAbleSession,
{
    async fn handle(&self, session: Session<T, D, S, P, I>) {
        let s = session.clone();
        if let Err(e) = self.0(s).await {
            session.send("Matcher Error:").await.ok();
            session.send(e.into_message()).await.ok();
        }
    }
}

#[async_trait]
pub trait ExtraHandler<T = (), D = (), S = (), P = (), I = ()> {
    async fn handle(&self, session: Session<T, D, S, P, I>);
    fn layer<H>(self, handler: H) -> LayeredHandler<Self, H>
    where
        Self: Sized,
        H: MatcherHandler<T, D, S, P, I>,
    {
        LayeredHandler {
            extra: self,
            handler,
        }
    }
}

pub struct LayeredHandler<E, H> {
    pub extra: E,
    pub handler: H,
}

impl<E, H, C> MatcherHandler<C> for LayeredHandler<E, H>
where
    E: ExtraHandler<C> + Send + Sync,
    H: MatcherHandler<C> + Send + Sync,
    C: Clone + Send + Sync + 'static,
{
    fn pre_handle(&self, session: &mut Session<C>) -> Signal {
        self.handler.pre_handle(session)
    }
    fn handle<'a, 't>(
        &'a self,
        session: Session<C>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        Box::pin(async move {
            self.extra.handle(session.clone()).await;
            self.handler.handle(session).await;
        })
    }
}

impl<C, I, Fut> ExtraHandler<C> for HandlerFn<I>
where
    C: Sync + Send + 'static,
    I: Fn(Session<C>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    fn handle<'a, 'b>(
        &'a self,
        session: Session<C>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
        Self: 'b,
    {
        Box::pin(self.0(session))
    }
}
