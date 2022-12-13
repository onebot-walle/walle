use crate::{JoinedRulePreHandler, MatcherHandler, PreHandler, Session, Signal};

use walle_core::{prelude::async_trait, WalleResult};

pub trait Rule {
    fn rule(&self, session: &Session) -> Signal;
    fn layer<H>(self, handler: H) -> LayeredRule<Self, H>
    where
        Self: Sized,
        H: MatcherHandler,
    {
        LayeredRule {
            rule: self,
            handler,
        }
    }
    fn with<R>(self, rule: R) -> JoinedRule<Self, R>
    where
        Self: Sized,
        R: Rule,
    {
        JoinedRule(self, rule)
    }
    fn with_pre_handler<PH>(self, pre_handler: PH) -> JoinedRulePreHandler<Self, PH>
    where
        Self: Sized,
        PH: PreHandler,
    {
        JoinedRulePreHandler(self, pre_handler)
    }
}

impl Rule for () {
    fn rule(&self, _: &Session) -> Signal {
        Signal::Matched
    }
}

pub struct LayeredRule<R, H> {
    pub rule: R,
    pub handler: H,
}

#[async_trait]
impl<R, H> MatcherHandler for LayeredRule<R, H>
where
    R: Rule + Send + Sync,
    H: MatcherHandler + Send + Sync,
{
    async fn handle(&self, session: Session) -> Signal {
        let mut sig = self.rule.rule(&session);
        if sig != Signal::NotMatch {
            sig = self.handler.handle(session).await & sig
        }
        sig
    }
}

pub struct JoinedRule<R0, R1>(pub R0, pub R1);

impl<R0, R1> Rule for JoinedRule<R0, R1>
where
    R0: Rule + Send + Sync,
    R1: Rule + Send + Sync,
{
    fn rule(&self, session: &Session) -> Signal {
        self.0.rule(session) & self.1.rule(session)
    }
}

pub struct RuleFn<F>(F);

impl<F> Rule for RuleFn<F>
where
    F: Fn(&Session) -> Signal + Send + Sync,
{
    fn rule(&self, session: &Session) -> Signal {
        self.0(session)
    }
}

pub fn rule_fn<F>(rule: F) -> RuleFn<F>
where
    F: Fn(&Session) -> Signal + 'static,
{
    RuleFn(rule)
}

pub struct RuleFnUnwarp<F>(F);

pub fn rule_fn_unwarp<F>(rule: F) -> RuleFnUnwarp<F>
where
    F: Fn(&Session) -> WalleResult<Signal> + 'static,
{
    RuleFnUnwarp(rule)
}

impl<F> Rule for RuleFnUnwarp<F>
where
    F: Fn(&Session) -> WalleResult<Signal> + Send + Sync,
{
    fn rule(&self, session: &Session) -> Signal {
        self.0(session).into()
    }
}
