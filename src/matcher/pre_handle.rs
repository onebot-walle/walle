use crate::{MatcherHandler, Session};
use std::future::Future;
use std::pin::Pin;

pub trait PreHandler<C> {
    fn pre_handle(&self, session: &mut Session<C>);
    fn layer<H>(self, handler: H) -> LayeredPreHandler<Self, H>
    where
        Self: Sized,
        H: MatcherHandler<C>,
    {
        LayeredPreHandler {
            pre: self,
            handler,
            before: false,
        }
    }
    fn layer_before<H>(self, handler: H) -> LayeredPreHandler<Self, H>
    where
        Self: Sized,
        H: MatcherHandler<C>,
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
    before: bool,
}

impl<P, H, C> MatcherHandler<C> for LayeredPreHandler<P, H>
where
    P: PreHandler<C> + Sync,
    H: MatcherHandler<C> + Sync,
    C: 'static,
{
    fn _pre_handle(&self, session: &mut Session<C>) {
        if self.before {
            self.pre.pre_handle(session);
            self.handler._pre_handle(session);
        } else {
            self.handler._pre_handle(session);
            self.pre.pre_handle(session);
        }
    }
    fn _match(&self, session: &Session<C>) -> bool {
        self.handler._match(session)
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
    I: Fn(&mut Session<C>) + Sync,
{
    fn pre_handle(&self, session: &mut Session<C>) {
        self.0(session);
    }
}

pub fn pre_handle_fn<I, C>(pre: I) -> PreHandleFn<I>
where
    I: Fn(&mut Session<C>) + Sync,
{
    PreHandleFn(pre)
}
