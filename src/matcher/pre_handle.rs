use crate::{MatcherHandler, Session, Signal};
use std::future::Future;
use std::pin::Pin;

pub trait PreHandler<T = (), D = (), S = (), P = (), I = ()> {
    fn pre_handle(&self, session: &mut Session<T, D, S, P, I>) -> Signal;
    fn layer<H>(self, handler: H) -> LayeredPreHandler<Self, H>
    where
        Self: Sized,
        H: MatcherHandler<T, D, S, P, I>,
    {
        LayeredPreHandler {
            pre: self,
            handler,
            before: true,
        }
    }
}

pub struct LayeredPreHandler<P, H> {
    pub pre: P,
    pub handler: H,
    pub before: bool,
}

impl<P, H, C> MatcherHandler<C> for LayeredPreHandler<P, H>
where
    P: PreHandler<C> + Sync,
    H: MatcherHandler<C> + Sync,
    C: 'static,
{
    fn pre_handle(&self, session: &mut Session<C>) -> Signal {
        if self.before {
            self.pre.pre_handle(session) + self.handler.pre_handle(session)
        } else {
            self.handler.pre_handle(session) + self.pre.pre_handle(session)
        }
    }
    fn handle<'a, 't>(
        &'a self,
        session: Session<C>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.handler.handle(session)
    }
}

pub struct PreHandleFn<I>(I);

impl<I, C> PreHandler<C> for PreHandleFn<I>
where
    I: Fn(&mut Session<C>) -> Signal + Sync,
{
    fn pre_handle(&self, session: &mut Session<C>) -> Signal {
        self.0(session)
    }
}

pub fn pre_handle_fn<I, C>(pre: I) -> PreHandleFn<I>
where
    I: Fn(&mut Session<C>) -> Signal + Sync,
{
    PreHandleFn(pre)
}
