use crate::{MatcherHandler, Rule, Session, Signal};

use walle_core::{prelude::async_trait, WalleResult};

pub trait PreHandler {
    fn pre_handle(&self, session: &mut Session) -> Signal;
    fn layer<H>(self, handler: H) -> LayeredPreHandler<Self, H>
    where
        Self: Sized,
        H: MatcherHandler,
    {
        LayeredPreHandler { pre: self, handler }
    }
    fn with<PR>(self, pr: PR) -> JoinedPreHandler<Self, PR>
    where
        Self: Sized,
        PR: PreHandler,
    {
        JoinedPreHandler(self, pr)
    }
    fn with_rule<R>(self, rule: R) -> JoinedPreHandlerRule<Self, R>
    where
        Self: Sized,
        R: Rule,
    {
        JoinedPreHandlerRule(self, rule)
    }
}

pub struct LayeredPreHandler<PR, H> {
    pub pre: PR,
    pub handler: H,
}

#[async_trait]
impl<PR, H> MatcherHandler for LayeredPreHandler<PR, H>
where
    PR: PreHandler + Send + Sync,
    H: MatcherHandler + Send + Sync,
{
    async fn handle(&self, mut session: Session) -> Signal {
        let mut sig = self.pre.pre_handle(&mut session);
        if sig != Signal::NotMatch {
            sig = self.handler.handle(session).await & sig;
        }
        sig
    }
}

pub struct JoinedPreHandler<PR0, PR1>(pub PR0, pub PR1);

impl<PR0, PR1> PreHandler for JoinedPreHandler<PR0, PR1>
where
    PR0: PreHandler + Sync,
    PR1: PreHandler + Sync,
{
    fn pre_handle(&self, session: &mut Session) -> Signal {
        self.0.pre_handle(session) & self.1.pre_handle(session)
    }
}

pub struct JoinedPreHandlerRule<PH, R>(pub PH, pub R);

impl<PH, R> PreHandler for JoinedPreHandlerRule<PH, R>
where
    PH: PreHandler + Sync,
    R: Rule + Sync,
{
    fn pre_handle(&self, session: &mut Session) -> Signal {
        self.0.pre_handle(session) & self.1.rule(session)
    }
}

pub struct JoinedRulePreHandler<R, PH>(pub R, pub PH);

impl<R, PH> PreHandler for JoinedRulePreHandler<R, PH>
where
    R: Rule + Sync,
    PH: PreHandler + Sync,
{
    fn pre_handle(&self, session: &mut Session) -> Signal {
        self.0.rule(session) & self.1.pre_handle(session)
    }
}

pub struct PreHandleFn<F>(F);

impl<F> PreHandler for PreHandleFn<F>
where
    F: Fn(&mut Session) -> Signal + Sync,
{
    fn pre_handle(&self, session: &mut Session) -> Signal {
        self.0(session)
    }
}

pub fn pre_handle_fn<F>(pre: F) -> PreHandleFn<F>
where
    F: Fn(&mut Session) -> Signal + Sync,
{
    PreHandleFn(pre)
}

pub struct PreHandleFnUnwarp<F>(F);

impl<F> PreHandler for PreHandleFnUnwarp<F>
where
    F: Fn(&mut Session) -> WalleResult<Signal> + Sync,
{
    fn pre_handle(&self, session: &mut Session) -> Signal {
        self.0(session).into()
    }
}

pub fn pre_handle_fn_unwarp<F>(pre: F) -> PreHandleFnUnwarp<F>
where
    F: Fn(&mut Session) -> WalleResult<Signal> + Sync,
{
    PreHandleFnUnwarp(pre)
}
