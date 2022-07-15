use crate::{MatcherHandler, Session, Signal};
use std::future::Future;
use std::pin::Pin;

pub trait Rule<T = (), D = (), S = (), P = (), I = ()> {
    fn rule(&self, session: &Session<T, D, S, P, I>) -> Signal;
    fn layer<H>(self, handler: H) -> LayeredRule<Self, H>
    where
        Self: Sized,
        H: MatcherHandler<T, D, S, P, I>,
    {
        LayeredRule {
            rule: self,
            handler,
            before: true,
        }
    }
}

pub struct LayeredRule<R, H> {
    pub rule: R,
    pub handler: H,
    pub before: bool,
}

impl<R, H, C> MatcherHandler<C> for LayeredRule<R, H>
where
    R: Rule<C> + Sync,
    H: MatcherHandler<C> + Sync,
    C: 'static,
{
    fn pre_handle(&self, session: &mut Session<C>) -> Signal {
        if self.before {
            self.rule.rule(session) + self.handler.pre_handle(session)
        } else {
            self.handler.pre_handle(session) + self.rule.rule(session)
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

pub struct RuleFn<I>(I);

impl<I, C> Rule<C> for RuleFn<I>
where
    I: Fn(&Session<C>) -> Signal,
{
    fn rule(&self, session: &Session<C>) -> Signal {
        self.0(session)
    }
}

pub fn rule_fn<I, C>(rule: I) -> RuleFn<I>
where
    I: Fn(&Session<C>) -> Signal,
{
    RuleFn(rule)
}
