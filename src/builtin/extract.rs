#[cfg(test)]
mod test {
    pub struct Command(walle_core::segment::Segments);

    #[walle_core::prelude::async_trait]
    impl crate::FromSessionPart for Command {
        async fn from_session_part(session: &mut crate::Session) -> walle_core::WalleResult<Self> {
            use walle_core::{segment::MessageMutExt, util::ValueMapExt};
            let mut segs = session
                .event
                .extra
                .try_get_as_mut::<&mut Vec<walle_core::util::Value>>("message")
                .map(|v| std::mem::take(v))?
                .into_iter()
                .map(|seg| seg.downcast())
                .collect::<walle_core::WalleResult<walle_core::segment::Segments>>()?;
            if let Ok(text) = segs.try_first_text_mut() {
                if let Some(mut rest) = text.strip_prefix("command") {
                    rest = rest.trim_start();
                    if !rest.is_empty() {
                        *text = rest.to_string();
                    } else {
                        segs.remove(0);
                    }
                    return Ok(Self(segs));
                }
            }
            Err(walle_core::WalleError::Other(format!(
                "Command not match with {}",
                "command"
            )))
        }
    }

    pub enum Commands {
        A(crate::walle_core::segment::Segments),
        B(crate::walle_core::segment::Segments),
    }

    #[crate::walle_core::prelude::async_trait]
    impl crate::FromSessionPart for Commands {
        async fn from_session_part(
            session: &mut crate::Session,
        ) -> crate::walle_core::WalleResult<Self> {
            use crate::walle_core::{segment::MessageMutExt, util::ValueMapExt};

            let mut segs = session
                .event
                .extra
                .try_get_as_mut::<&mut Vec<crate::walle_core::util::Value>>("message")
                .map(std::mem::take)?
                .into_iter()
                .map(|seg| seg.downcast())
                .collect::<walle_core::WalleResult<walle_core::segment::Segments>>()?;
            if let Ok(text) = segs.try_first_text_mut() {
                if let Some(mut rest) = text.strip_prefix("a") {
                    rest = rest.trim_start();
                    if !rest.is_empty() {
                        *text = rest.to_string();
                    } else {
                        segs.remove(0);
                    }
                    return Ok(Self::A(segs));
                } else if let Some(mut rest) = text.strip_prefix("b") {
                    rest = rest.trim_start();
                    if !rest.is_empty() {
                        *text = rest.to_string();
                    } else {
                        segs.remove(0);
                    }
                    return Ok(Self::B(segs));
                }
            }
            Err(crate::walle_core::WalleError::Other(format!(
                "Command not match with {}",
                ["a", "b"].join(" or ")
            )))
        }
    }

    pub struct StartWith(walle_core::segment::Segments);

    #[walle_core::prelude::async_trait]
    impl crate::FromSessionPart for StartWith {
        async fn from_session_part(session: &mut crate::Session) -> walle_core::WalleResult<Self> {
            use walle_core::util::ValueMapExt;
            let segs = session
                .event
                .extra
                .try_get_as_mut::<&mut Vec<walle_core::util::Value>>("message")
                .map(|v| std::mem::take(v))?
                .into_iter()
                .map(|seg| seg.downcast())
                .collect::<walle_core::WalleResult<walle_core::segment::Segments>>()?;
            if let Some(Ok(text)) = segs
                .first()
                .map(|seg| seg.data.try_get_as_ref::<&str>("text"))
            {
                if text.starts_with("started") {
                    return Ok(Self(segs));
                }
            }
            Err(walle_core::WalleError::Other(format!(
                "Message not start with {}",
                "started"
            )))
        }
    }
}

#[macro_export]
macro_rules! on_command {
    ($cid: ident, $command: expr) => {
        on_command!($cid, $command, walle);
    };
    ($cid: ident, $command: expr, $span: tt) => {
        pub struct $cid($span::walle_core::segment::Segments);

        #[$span::walle_core::prelude::async_trait]
        impl $span::FromSessionPart for $cid {
            async fn from_session_part(
                session: &mut $span::Session,
            ) -> $span::walle_core::WalleResult<Self> {
                use $span::walle_core::{segment::MessageMutExt, util::ValueMapExt};
                let mut segs = session
                    .event
                    .extra
                    .try_get_as_mut::<&mut Vec<$span::walle_core::util::Value>>("message")
                    .map(|v| std::mem::take(v))?
                    .into_iter()
                    .map(|seg| seg.downcast())
                    .collect::<$span::walle_core::WalleResult<$span::walle_core::segment::Segments>>()?;
                if let Ok(text) = segs.try_first_text_mut() {
                    if let Some(mut rest) = text.strip_prefix($command) {
                        rest = rest.trim_start();
                        if !rest.is_empty() {
                            *text = rest.to_string();
                        } else {
                            segs.remove(0);
                        }
                        return Ok(Self(segs));
                    }
                }
                Err($span::walle_core::WalleError::Other(format!(
                    "Command not match with {}",
                    $command
                )))
            }
        }
    };
    ($cid: ident, $($subids: ident => $commands: expr),*) => {
        pub enum $cid {
            $($subids(walle::walle_core::segment::Segments)),*
        }

        #[walle::walle_core::prelude::async_trait]
        impl walle::FromSessionPart for $cid {
            async fn from_session_part(
                session: &mut walle::Session,
            ) -> walle::walle_core::WalleResult<Self> {
                use walle::walle_core::{segment::MessageMutExt, util::ValueMapExt};

                let mut segs = session
                    .event
                    .extra
                    .try_get_as_mut::<&mut Vec<walle::walle_core::util::Value>>("message")
                    .map(std::mem::take)?
                    .into_iter()
                    .map(|seg| seg.downcast())
                    .collect::<walle::walle_core::WalleResult<walle::walle_core::segment::Segments>>()?;
                if let Ok(text) = segs.try_first_text_mut() {
                    $(if let Some(mut rest) = text.strip_prefix($commands) {
                        rest = rest.trim_start();
                        if !rest.is_empty() {
                            *text = rest.to_string();
                        } else {
                            segs.remove(0);
                        }
                        return Ok(Self::$subids(segs));
                    })*
                }
                Err(walle::walle_core::WalleError::Other(format!(
                    "Command not match with {}",
                    [$($commands,)*].join(" or ")
                )))
            }
        }
    }
}
