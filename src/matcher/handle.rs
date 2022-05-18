use super::{LayeredPreHandler, LayeredRule, PreHandler, Rule, Session};
use std::{future::Future, pin::Pin};

#[async_trait::async_trait]
pub trait MatcherHandler<C>: Sync {
    fn _match(&self, _session: &Session<C>) -> bool {
        true
    }
    fn _pre_handle(&self, _session: &mut Session<C>) -> bool {
        true
    }
    async fn handle(&self, session: Session<C>);
}

pub trait MatcherHandlerExt<C>: MatcherHandler<C> {
    fn rule<R>(self, rule: R) -> LayeredRule<R, Self>
    where
        Self: Sized,
        R: Rule<C>,
    {
        rule.layer(self)
    }
    fn pre_handle<P>(self, pre: P, as_rule: bool) -> LayeredPreHandler<P, Self>
    where
        Self: Sized,
        P: PreHandler<C>,
    {
        pre.layer(self, as_rule)
    }
    fn pre_handle_before<P>(self, pre: P, as_rule: bool) -> LayeredPreHandler<P, Self>
    where
        Self: Sized,
        P: PreHandler<C>,
    {
        pre.layer_before(self, as_rule)
    }
}

impl<C, H: MatcherHandler<C>> MatcherHandlerExt<C> for H {}

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
