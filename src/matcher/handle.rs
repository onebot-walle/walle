use crate::{ActionCaller, MatchersConfig, TempMatchers};

use super::{LayeredPreHandler, LayeredRule, PreHandler, Rule, Session};
use std::{future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;
use walle_core::{
    event::{DetailTypeDeclare, Event, ImplDeclare, PlatformDeclare, SubTypeDeclare, TypeDeclare},
    prelude::WalleError,
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
            (Self::MatchAndBlock, Self::MatchAndBlock)
            | (Self::Matched, Self::MatchAndBlock)
            | (Self::MatchAndBlock, Self::Matched) => Self::MatchAndBlock,
            (Self::Matched, Self::Matched) => Self::Matched,
            (_, _) => Self::NotMatch,
        }
    }
}

pub trait _MatcherHandler {
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

impl<H, T, D, S, P, I> _MatcherHandler for BoxedHandler<H, T, D, S, P, I>
where
    H: MatcherHandler<T, D, S, P, I> + Send + 'static,
    T: for<'a> TryFrom<&'a mut Event, Error = WalleError> + TypeDeclare + Send + 'static,
    D: for<'a> TryFrom<&'a mut Event, Error = WalleError> + DetailTypeDeclare + Send + 'static,
    S: for<'a> TryFrom<&'a mut Event, Error = WalleError> + SubTypeDeclare + Send + 'static,
    I: for<'a> TryFrom<&'a mut Event, Error = WalleError> + ImplDeclare + Send + 'static,
    P: for<'a> TryFrom<&'a mut Event, Error = WalleError> + PlatformDeclare + Send + 'static,
{
    fn call(
        &self,
        event: Event,
        config: &Arc<MatchersConfig>,
        caller: &Arc<dyn ActionCaller + Send + 'static>,
        temp: &TempMatchers,
    ) -> Signal {
        if let Ok(event) = event.try_into() {
            let mut session =
                Session::<T, D, S, P, I>::new(event, caller.clone(), config.clone(), temp.clone());
            let signal = self.0.pre_handle(&mut session);
            let handler = self.0.clone();
            if signal != Signal::NotMatch {
                tokio::spawn(async move {
                    handler.handle(session).await;
                });
            }
            signal
        } else {
            Signal::NotMatch
        }
    }
}

/// Matcher Handler
#[async_trait]
pub trait MatcherHandler<T = (), D = (), S = (), P = (), I = ()>: Sync {
    fn pre_handle(&self, _session: &mut Session<T, D, S, P, I>) -> Signal {
        Signal::Matched
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
    fn boxed(self) -> BoxedHandler<Self, T, D, S, P, I>
    where
        Self: Sized,
    {
        BoxedHandler(Arc::new(self), std::marker::PhantomData::default())
    }
}

impl<T, D, S, P, I, H: MatcherHandler<T, D, S, P, I>> MatcherHandlerExt<T, D, S, P, I> for H {}

pub struct HandlerFn<I>(I);

pub fn handler_fn<I, C, Fut>(inner: I) -> HandlerFn<I>
where
    I: Fn(Session<C>) -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send,
    C: Sync + Send + 'static,
{
    HandlerFn(inner)
}

impl<C, I, Fut> MatcherHandler<C> for HandlerFn<I>
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
