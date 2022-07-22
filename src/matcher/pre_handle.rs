use crate::{MatcherHandler, Rule, Session, Signal};
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
    fn with<PR>(self, pr: PR) -> JoinedPreHandler<Self, PR>
    where
        Self: Sized,
        PR: PreHandler<T, D, S, P, I>,
    {
        JoinedPreHandler(self, pr)
    }
    fn with_rule<R>(self, rule: R) -> JoinedRulePreHandler<R, Self>
    where
        Self: Sized,
        R: Rule<T, D, S, P, I>,
    {
        JoinedRulePreHandler(rule, self, false)
    }
}

pub struct LayeredPreHandler<PR, H> {
    pub pre: PR,
    pub handler: H,
    pub before: bool,
}

impl<PR, H, T, D, S, P, I> MatcherHandler<T, D, S, P, I> for LayeredPreHandler<PR, H>
where
    PR: PreHandler<T, D, S, P, I> + Sync,
    H: MatcherHandler<T, D, S, P, I> + Sync,
{
    fn pre_handle(&self, session: &mut Session<T, D, S, P, I>) -> Signal {
        if self.before {
            self.pre.pre_handle(session) + self.handler.pre_handle(session)
        } else {
            self.handler.pre_handle(session) + self.pre.pre_handle(session)
        }
    }
    fn handle<'a, 't>(
        &'a self,
        session: Session<T, D, S, P, I>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.handler.handle(session)
    }
}

pub struct JoinedPreHandler<PR0, PR1>(pub PR0, pub PR1);

impl<PR0, PR1, T, D, S, P, I> PreHandler<T, D, S, P, I> for JoinedPreHandler<PR0, PR1>
where
    PR0: PreHandler<T, D, S, P, I> + Sync,
    PR1: PreHandler<T, D, S, P, I> + Sync,
{
    fn pre_handle(&self, session: &mut Session<T, D, S, P, I>) -> Signal {
        self.0.pre_handle(session) + self.1.pre_handle(session)
    }
}

pub struct JoinedRulePreHandler<R, PR>(pub R, pub PR, pub bool);

impl<R, PR, T, D, S, P, I> PreHandler<T, D, S, P, I> for JoinedRulePreHandler<R, PR>
where
    R: Rule<T, D, S, P, I> + Sync,
    PR: PreHandler<T, D, S, P, I> + Sync,
{
    fn pre_handle(&self, session: &mut Session<T, D, S, P, I>) -> Signal {
        if self.2 {
            self.1.pre_handle(session) + self.0.rule(session)
        } else {
            self.0.rule(session) + self.1.pre_handle(session)
        }
    }
}

pub struct PreHandleFn<F>(F);

impl<F, T, D, S, P, I> PreHandler<T, D, S, P, I> for PreHandleFn<F>
where
    F: Fn(&mut Session<T, D, S, P, I>) -> Signal + Sync,
{
    fn pre_handle(&self, session: &mut Session<T, D, S, P, I>) -> Signal {
        self.0(session)
    }
}

pub fn pre_handle_fn<F, T, D, S, P, I>(pre: F) -> PreHandleFn<F>
where
    F: Fn(&mut Session<T, D, S, P, I>) -> Signal + Sync,
{
    PreHandleFn(pre)
}
