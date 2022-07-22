use crate::{JoinedRulePreHandler, MatcherHandler, PreHandler, Session, Signal};
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
    fn with<R>(self, rule: R) -> JoinedRule<Self, R>
    where
        Self: Sized,
        R: Rule<T, D, S, P, I>,
    {
        JoinedRule(self, rule)
    }
    fn with_pre_handler<PR>(self, pre_handler: PR) -> JoinedRulePreHandler<Self, PR>
    where
        Self: Sized,
        PR: PreHandler<T, D, S, P, I>,
    {
        JoinedRulePreHandler(self, pre_handler, true)
    }
}

pub struct LayeredRule<R, H> {
    pub rule: R,
    pub handler: H,
    pub before: bool,
}

impl<R, H, T, D, S, P, I> MatcherHandler<T, D, S, P, I> for LayeredRule<R, H>
where
    R: Rule<T, D, S, P, I> + Sync,
    H: MatcherHandler<T, D, S, P, I> + Sync,
{
    fn pre_handle(&self, session: &mut Session<T, D, S, P, I>) -> Signal {
        if self.before {
            self.rule.rule(session) + self.handler.pre_handle(session)
        } else {
            self.handler.pre_handle(session) + self.rule.rule(session)
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

pub struct JoinedRule<R0, R1>(pub R0, pub R1);

impl<R0, R1, T, D, S, P, I> Rule<T, D, S, P, I> for JoinedRule<R0, R1>
where
    R0: Rule<T, D, S, P, I> + Sync,
    R1: Rule<T, D, S, P, I> + Sync,
{
    fn rule(&self, session: &Session<T, D, S, P, I>) -> Signal {
        self.0.rule(session) + self.1.rule(session)
    }
}

pub struct RuleFn<F>(F);

impl<F, T, D, S, P, I> Rule<T, D, S, P, I> for RuleFn<F>
where
    F: Fn(&Session<T, D, S, P, I>) -> Signal,
{
    fn rule(&self, session: &Session<T, D, S, P, I>) -> Signal {
        self.0(session)
    }
}

pub fn rule_fn<F, T, D, S, P, I>(rule: F) -> RuleFn<F>
where
    F: Fn(&Session<T, D, S, P, I>) -> Signal,
{
    RuleFn(rule)
}
