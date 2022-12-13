use crate::{FromSession, FromSessionPart};

use super::Session;
use std::{future::Future, sync::Arc};

use async_trait::async_trait;
use walle_core::WalleResult;

#[derive(Default, Debug, PartialEq, Eq)]
pub enum Signal {
    MatchAndBlock,
    Matched,
    #[default]
    NotMatch,
}

impl core::ops::BitOr for Signal {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (_, Self::MatchAndBlock) | (Self::MatchAndBlock, _) => Self::MatchAndBlock,
            (_, Self::Matched) | (Self::Matched, _) => Self::Matched,
            _ => Self::NotMatch,
        }
    }
}

impl core::ops::BitAnd for Signal {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::MatchAndBlock, Self::MatchAndBlock) => Self::MatchAndBlock,
            (Self::MatchAndBlock, Self::Matched)
            | (Self::Matched, Self::MatchAndBlock)
            | (Self::Matched, Self::Matched) => Self::Matched,
            _ => Self::NotMatch,
        }
    }
}

impl From<WalleResult<Signal>> for Signal {
    fn from(r: WalleResult<Signal>) -> Self {
        r.unwrap_or(Self::NotMatch)
    }
}

#[async_trait]
pub trait MatcherHandler {
    async fn handle(&self, session: Session) -> Signal;
    fn boxed(self) -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

#[doc(hidden)]
#[async_trait]
pub trait ArcMatcherHandler {
    async fn handle(self: &Arc<Self>, session: Session) -> Signal;
}

impl<T: ArcMatcherHandler> MatcherHandler for Arc<T> {
    fn handle<'life0, 'async_trait>(
        &'life0 self,
        session: Session,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Signal> + core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        self.handle(session)
    }
}

#[async_trait]
pub trait _MatcherHandler<T> {
    async fn _handle(&self, session: Session) -> Signal;
}

pub fn matcher<H, T>(h: H) -> BoxedMatcherHandler<H, T>
where
    H: _MatcherHandler<T>,
{
    BoxedMatcherHandler(h, std::marker::PhantomData::default())
}

pub struct BoxedMatcherHandler<H, T>(H, std::marker::PhantomData<T>);

impl<H, T> MatcherHandler for BoxedMatcherHandler<H, T>
where
    H: _MatcherHandler<T>,
{
    fn handle<'life0, 'async_trait>(
        &'life0 self,
        session: Session,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Signal> + core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        self.0._handle(session)
    }
}

impl<F, Fut> _MatcherHandler<()> for F
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    fn _handle<'a, 't>(
        &'a self,
        _session: Session,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Signal> + core::marker::Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        Box::pin(async move {
            tokio::spawn(self());
            Signal::Matched
        })
    }
}

impl<F, T, Fut> _MatcherHandler<T> for F
where
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
    T: FromSession + Send,
{
    fn _handle<'a, 't>(
        &'a self,
        session: Session,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Signal> + core::marker::Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        Box::pin(async move {
            let t = match T::from_session(session).await {
                Ok(t) => t,
                Err(e) => {
                    tracing::debug!(target: "Walle", "from session failed: {}", e);
                    return Signal::NotMatch;
                }
            };
            tokio::spawn(self(t));
            Signal::Matched
        })
    }
}

macro_rules! impl_matcher_handler {
    ($($ty: ident),*) => {
        #[allow(non_snake_case)]
        impl<F, $($ty,)* T, Fut> _MatcherHandler<($($ty,)* T)> for F
        where
            F: Fn($($ty,)* T) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = ()> + Send + 'static,
            $($ty: FromSessionPart + Send,)*
            T: FromSession + Send,
        {
            fn _handle<'a, 't>(
                &'a self,
                mut session: Session,
            ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Signal> + core::marker::Send + 't>>
            where
                'a: 't,
                Self: 't,
            {
                Box::pin(async move {
                    $(let $ty = match $ty::from_session_part(&mut session).await {
                        Ok(t) => t,
                        Err(e) => {
                            tracing::debug!(target: "Walle", "from session part failed: {}", e);
                            return Signal::NotMatch;
                        }
                    };)*
                    let t = match T::from_session(session).await {
                        Ok(t) => t,
                        Err(e) => {
                            tracing::debug!(target: "Walle", "from session failed: {}", e);
                            return Signal::NotMatch;
                        }
                    };
                    tokio::spawn(self($($ty,)* t));
                    Signal::Matched
                })
            }
        }
    };
}

impl_matcher_handler!(T0);
impl_matcher_handler!(T0, T1);
impl_matcher_handler!(T0, T1, T2);
impl_matcher_handler!(T0, T1, T2, T3);
impl_matcher_handler!(T0, T1, T2, T3, T4);
impl_matcher_handler!(T0, T1, T2, T3, T4, T5);
impl_matcher_handler!(T0, T1, T2, T3, T4, T5, T6);
impl_matcher_handler!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_matcher_handler!(T0, T1, T2, T3, T4, T5, T6, T7, T8);

#[cfg(test)]
mod test {
    pub struct StructMatcher;

    impl StructMatcher {
        async fn method(
            &self,
            _event: crate::walle_core::event::GroupMessageEvent,
            _session: crate::Session,
        ) {
            ()
        }
        #[allow(dead_code)]
        async fn failable_method(
            self: &std::sync::Arc<Self>,
            _event: crate::walle_core::event::GroupMessageEvent,
            _session: crate::Session,
        ) -> crate::walle_core::WalleResult<()> {
            Ok(())
        }
    }

    #[crate::walle_core::prelude::async_trait]
    impl crate::ArcMatcherHandler for StructMatcher {
        async fn handle(self: &std::sync::Arc<Self>, mut session: crate::Session) -> crate::Signal {
            use crate::{FromSession, FromSessionPart};
            let event =
                match walle_core::event::GroupMessageEvent::from_session_part(&mut session).await {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::debug!(target: "Walle", "from session part failed: {}", e);
                        return crate::Signal::NotMatch;
                    }
                };
            let session = match crate::Session::from_session(session).await {
                Ok(e) => e,
                Err(e) => {
                    tracing::debug!(target: "Walle", "from session failed: {}", e);
                    return crate::Signal::NotMatch;
                }
            };
            let new = self.clone();
            crate::tokio::spawn(async move { new.method(event, session).await });
            crate::Signal::Matched
        }
    }
}

#[macro_export]
macro_rules! matcher {
    ($s: ident, $m: ident, $($i: ident: $t: ty),*) => {
        #[walle::walle_core::prelude::async_trait]
        impl walle::ArcMatcherHandler for $s {
            async fn handle(self: &std::sync::Arc<Self>, mut session: walle::Session) -> walle::Signal {
                use walle::{FromSession, FromSessionPart};
                $(let $i =
                    match <$t>::from_session_part(&mut session).await {
                        Ok(e) => e,
                        Err(e) => {
                            walle::tracing::debug!(target: "Walle", "from session part failed: {}", e);
                            return walle::Signal::NotMatch;
                        }
                    };)*
                let session = match walle::Session::from_session(session).await {
                    Ok(e) => e,
                    Err(e) => {
                        walle::tracing::debug!(target: "Walle", "from session failed: {}", e);
                        return walle::Signal::NotMatch;
                    }
                };
                let new = self.clone();
                walle::tokio::spawn(async move { new.$m($($i,)* session).await });
                walle::Signal::Matched
            }
        }
    };
    (failable: $s: ident, $m: ident, $($i: ident: $t: ty),*) => {
        #[walle::walle_core::prelude::async_trait]
        impl walle::ArcMatcherHandler for $s {
            async fn handle(self: &std::sync::Arc<Self>, mut session: walle::Session) -> walle::Signal {
                use walle::{FromSession, FromSessionPart};
                $(let $i =
                    match <$t>::from_session_part(&mut session).await {
                        Ok(e) => e,
                        Err(e) => {
                            walle::tracing::debug!(target: "Walle", "from session part failed: {}", e);
                            return walle::Signal::NotMatch;
                        }
                    };)*
                let session = match walle::Session::from_session(session).await {
                    Ok(e) => e,
                    Err(e) => {
                        walle::tracing::debug!(target: "Walle", "from session failed: {}", e);
                        return walle::Signal::NotMatch;
                    }
                };
                let new = self.clone();
                walle::tokio::spawn(async move { if let Err(e) = new.$m($($i,)* session).await {
                    walle::tracing::warn!(target: "Walle", "matcher failed: {}", e);
                } });
                walle::Signal::Matched
            }
        }
    };
}
