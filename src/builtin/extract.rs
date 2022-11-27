mod test {
    pub struct C(walle_core::segment::Segments);

    #[walle_core::prelude::async_trait]
    impl crate::FromSessionPart for C {
        async fn from_session_part(session: &mut crate::Session) -> walle_core::WalleResult<Self> {
            use walle_core::{segment::MessageExt, util::ValueMapExt};
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
                    while let Some(r2) = rest.strip_prefix(" ") {
                        rest = r2;
                    }
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
}

#[macro_export]
macro_rules! on_command {
    ($cid: ident, $command: expr) => {
        on_command!($cid, $command, walle);
    };
    ($cid: ident, $command: expr, $span: tt) => {
        pub struct $cid(walle_core::segment::Segments);

        #[walle_core::prelude::async_trait]
        impl $span::FromSessionPart for $cid {
            async fn from_session_part(
                session: &mut crate::Session,
            ) -> walle_core::WalleResult<Self> {
                use walle_core::{segment::MessageExt, util::ValueMapExt};
                let mut segs = session
                    .event
                    .extra
                    .try_get_as_mut::<&mut Vec<walle_core::util::Value>>("message")
                    .map(|v| std::mem::take(v))?
                    .into_iter()
                    .map(|seg| seg.downcast())
                    .collect::<walle_core::WalleResult<walle_core::segment::Segments>>()?;
                if let Ok(text) = segs.try_first_text_mut() {
                    if let Some(mut rest) = text.strip_prefix($command) {
                        while let Some(r2) = rest.strip_prefix(' ') {
                            rest = r2;
                        }
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
                    $command
                )))
            }
        }
    };
}
