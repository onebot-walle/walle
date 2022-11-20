use std::{future::Future, pin::Pin, sync::Arc};
use walle_core::{
    action::Action,
    event::Event,
    prelude::{async_trait, GetSelfs},
    resp::Resp,
    structs::Selft,
    util::{GetSelf, Value},
    ActionHandler, EventHandler, OneBot, WalleError, WalleResult,
};

use crate::{Bot, Session};

#[async_trait]
pub trait ActionCaller: GetSelfs + Sync {
    async fn call_action(&self, action: Action) -> WalleResult<Resp>;
    async fn get_bots(&self) -> Vec<Bot>;
}

#[async_trait]
impl<AH, EH> ActionCaller for Arc<OneBot<AH, EH>>
where
    AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
    EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
{
    async fn call_action(&self, action: Action) -> WalleResult<Resp> {
        self.handle_action(action).await
    }

    async fn get_bots(&self) -> Vec<Bot> {
        self.get_selfs()
            .await
            .into_iter()
            .map(|id| Bot {
                selft: id,
                caller: Arc::new(self.clone()),
            })
            .collect()
    }
}

impl GetSelfs for Bot {
    fn get_impl<'life0, 'life1, 'async_trait>(
        &'life0 self,
        selft: &'life1 Selft,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = String> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        self.caller.get_impl(selft)
    }
    fn get_selfs<'life0, 'async_trait>(
        &'life0 self,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Vec<Selft>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        self.caller.get_selfs()
    }
}

impl ActionCaller for Bot {
    fn call_action<'a, 't>(
        &'a self,
        action: Action,
    ) -> Pin<Box<dyn Future<Output = WalleResult<Resp>> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.caller.call_action(action)
    }
    fn get_bots<'a, 't>(&'a self) -> Pin<Box<dyn Future<Output = Vec<Bot>> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.caller.get_bots()
    }
}

impl<T, D, S, P, I> GetSelfs for Session<T, D, S, P, I> {
    fn get_impl<'life0, 'life1, 'async_trait>(
        &'life0 self,
        selft: &'life1 Selft,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = String> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        self.caller.get_impl(selft)
    }
    fn get_selfs<'life0, 'async_trait>(
        &'life0 self,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Vec<Selft>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        self.caller.get_selfs()
    }
}

impl<T, D, S, P, I> ActionCaller for Session<T, D, S, P, I>
where
    T: GetSelf + Sync,
    D: Sync,
    S: Sync,
    P: Sync,
    I: Sync,
{
    fn call_action<'a, 't>(
        &'a self,
        mut action: Action,
    ) -> Pin<Box<dyn Future<Output = WalleResult<Resp>> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        action.selft = Some(self.event.ty.get_self());
        self.caller.call_action(action)
    }
    fn get_bots<'a, 't>(&'a self) -> Pin<Box<dyn Future<Output = Vec<Bot>> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.caller.get_bots()
    }
}

macro_rules! action_ext {
    ($fname: ident, $aty: expr => $rty: ty) => {
        fn $fname<'a, 't>(&'a self) -> Pin<Box<dyn Future<Output = WalleResult<Vec<Event>>> + 't>>
        where
            'a: 't,
            Self: 't,
        {
            self.call(walle_core::action::Action {
                action: $aty.to_string(),
                params: walle_core::prelude::ValueMap::default(),
                selft: None,
            })
        }
    };
    ($fname: ident, $a: expr => $rty: ty, $($f: ident: $fty: ty),*) => {
        fn $fname<'a, 't>(&'a self, $($f: $fty),*) -> Pin<Box<dyn Future<Output = WalleResult<Vec<Event>>> + 't>>
        where
            'a: 't,
            Self: 't,
        {
            self.call($a)
        }
    };
}

#[async_trait]
pub trait ActionCallerExt: ActionCaller {
    async fn call<A, R>(&self, action: A) -> WalleResult<R>
    where
        A: Into<Action> + Send,
        R: TryFrom<Value, Error = WalleError>,
    {
        self.call_action(action.into())
            .await?
            .as_result()
            .map_err(WalleError::RespError)?
            .try_into()
    }
    fn get_latest_events<'a, 't>(
        &'a self,
        limit: i64,
        timeout: i64,
    ) -> Pin<Box<dyn Future<Output = WalleResult<Vec<Event>>> + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.call(walle_core::action::GetLatestEvents { limit, timeout })
    }
    action_ext!(get_supported_actions, "get_supported_actions" => Vec<String>);
    action_ext!(get_status, "get_status" => walle_core::structs::Status);
    action_ext!(get_version, "get_version" => walle_core::structs::Version);
    fn send_message<'a, 't, M>(
        &'a self,
        detail_type: String,
        user_id: Option<String>,
        group_id: Option<String>,
        guild_id: Option<String>,
        channel_id: Option<String>,
        message: M,
    ) -> Pin<Box<dyn Future<Output = WalleResult<walle_core::structs::SendMessageResp>> + 't>>
    where
        'a: 't,
        Self: 't,
        M: walle_core::segment::IntoMessage,
    {
        self.call(walle_core::action::SendMessage { 
            detail_type,
            user_id,
            group_id,
            guild_id,
            channel_id,
            message:message.into_message()
        })
    }
    fn send_private_message<'a, 't, M>(
        &'a self,
        user_id: String,
        message: M,
    ) -> Pin<Box<dyn Future<Output = WalleResult<walle_core::structs::SendMessageResp>> + 't>>
    where
        'a: 't,
        Self: 't,
        M: walle_core::segment::IntoMessage,
    {
        self.call(walle_core::action::SendMessage { 
            detail_type: "private".to_owned(),
            user_id: Some(user_id),
            group_id: None,
            guild_id: None,
            channel_id: None,
            message:message.into_message()
        })
    }
    fn send_group_message<'a, 't, M>(
        &'a self,
        group_id: String,
        message: M,
    ) -> Pin<Box<dyn Future<Output = WalleResult<walle_core::structs::SendMessageResp>> + 't>>
    where
        'a: 't,
        Self: 't,
        M: walle_core::segment::IntoMessage,
    {
        self.call(walle_core::action::SendMessage { 
            detail_type: "group".to_owned(),
            user_id: None,
            group_id: Some(group_id),
            guild_id: None,
            channel_id: None,
            message:message.into_message()
        })
    }
    fn send_channel_message<'a, 't, M>(
        &'a self,
        guild_id: String,
        channel_id: String,
        message: M,
    ) -> Pin<Box<dyn Future<Output = WalleResult<walle_core::structs::SendMessageResp>> + 't>>
    where
        'a: 't,
        Self: 't,
        M: walle_core::segment::IntoMessage,
    {
        self.call(walle_core::action::SendMessage { 
            detail_type: "channel".to_owned(),
            user_id: None,
            group_id: None,
            guild_id: Some(guild_id),
            channel_id: Some(channel_id),
            message:message.into_message()
        })
    }
    action_ext!(
        delete_message,
        walle_core::action::DeleteMessage { message_id } => (),
        message_id: String
    );
    action_ext!(
        get_self_info,
        "get_self_info" => walle_core::structs::UserInfo
    );
    action_ext!(
        get_user_info,
        walle_core::action::GetUserInfo { user_id } => walle_core::structs::UserInfo,
        user_id: String
    );
    action_ext!(
        get_friend_list,
        "get_friend_list" => Vec<walle_core::structs::UserInfo>
    );
    action_ext!(
        get_group_info,
        walle_core::action::GetGroupInfo { group_id } => walle_core::structs::GroupInfo,
        group_id: String
    );
    action_ext!(
        get_group_list,
        "get_group_list" => Vec<walle_core::structs::GroupInfo>
    );
    action_ext!(
        get_group_member_info,
        walle_core::action::GetGroupMemberInfo { group_id, user_id } =>
        walle_core::structs::UserInfo,
        group_id: String,
        user_id: String
    );
    action_ext!(
        get_group_member_list,
        walle_core::action::GetGroupMemberList {group_id} => 
        Vec<walle_core::structs::UserInfo>, 
        group_id: String);
    action_ext!(
        set_group_name,
        walle_core::action::SetGroupName {group_id, group_name} => (),
        group_id: String,
        group_name: String
    );
    action_ext!(
        leave_group,
        walle_core::action::LeaveGroup {group_id} => (),
        group_id: String
    );
    action_ext!(
        get_guild_info, 
        walle_core::action::GetGuildInfo { guild_id } => 
        walle_core::structs::GuildInfo,
        guild_id: String
    );
    action_ext!(
        get_guild_list,
        "get_guild_list" => Vec<walle_core::structs::GuildInfo>
    );
    action_ext!(
        set_guild_name,
        walle_core::action::SetGuildName { guild_id, guild_name } => (),
        guild_id: String,
        guild_name: String
    );
    action_ext!(
        get_guild_member_info,
        walle_core::action::GetGuildMemberInfo { guild_id, user_id } => 
        walle_core::structs::UserInfo,
        guild_id: String,
        user_id: String
    );
    action_ext!(
        get_guild_member_list,
        walle_core::action::GetGuildMemberList { guild_id } => 
        Vec<walle_core::structs::UserInfo>,
        guild_id: String
    );
    action_ext!(
        leave_guild,
        walle_core::action::LeaveGuild { guild_id } => (),
        guild_id: String
    );
    action_ext!(
        get_channel_info,
        walle_core::action::GetChannelInfo { guild_id, channel_id } => 
        walle_core::structs::ChannelInfo,
        guild_id: String,
        channel_id: String
    );
    action_ext!(
        get_channel_list,
        walle_core::action::GetChannelList { guild_id } => 
        Vec<walle_core::structs::ChannelInfo>,
        guild_id: String
    );
    action_ext!(
        set_channel_name,
        walle_core::action::SetChannelName { guild_id, channel_id, channel_name } => (),
        guild_id: String,
        channel_id: String,
        channel_name: String
    );
    // action_ext!(
    //     get_channel_member_info,
    //     walle_core::action::GetChannelMemberInfo { guild_id, channel_id, user_id } => 
    //     walle_core::structs::UserInfo,
    //     guild_id: String,
    //     channel_id: String,
    //     user_id: String
    // );
    // action_ext!(
    //     get_channel_member_list,
    //     walle_core::action::GetChannelMemberList { guild_id, channel_id } => 
    //     Vec<walle_core::structs::UserInfo>,
    //     guild_id: String,
    //     channel_id: String
    // );
    // action_ext!(
    //     leave_channel,
    //     walle_core::action::LeaveChannel { guild_id, channel_id } => (),
    //     guild_id: String,
    //     channel_id: String
    // );

    // file
    action_ext!(
        upload_file,
        walle_core::action::UploadFile { ty, name, url, headers, path, data, sha256 } => 
        walle_core::structs::FileId,
        ty: String,
        name: String,
        url: Option<String>,
        headers: Option<std::collections::HashMap<String, String>>,
        path: Option<String>,
        data: Option<walle_core::util::OneBotBytes>,
        sha256: Option<String>
    );
    action_ext!(
        upload_file_by_url,
        walle_core::action::UploadFile { 
            ty: "url".to_owned(),
            name,
            url: Some(url),
            headers,
            path: None,
            data: None,
            sha256
        } => walle_core::structs::FileId,
        name: String,
        url: String,
        headers: Option<std::collections::HashMap<String, String>>,
        sha256: Option<String>
    );
    action_ext!(
        upload_file_by_path,
        walle_core::action::UploadFile { 
            ty: "path".to_owned(),
            name,
            url: None,
            headers: None,
            path: Some(path),
            data: None,
            sha256
        } => walle_core::structs::FileId,
        name: String,
        path: String,
        sha256: Option<String>
    );
    action_ext!(
        upload_file_by_data,
        walle_core::action::UploadFile { 
            ty: "data".to_owned(),
            name,
            url: None,
            headers: None,
            path: None,
            data: Some(walle_core::util::OneBotBytes(data)),
            sha256
        } => walle_core::structs::FileId,
        name: String,
        data: Vec<u8>,
        sha256: Option<String>
    );
    // async fn upload_file_fragmented_by_path(
    //     &self,
    //     path: &str,
    //     size: Option<u32>,
    // ) -> WalleResult<walle_core::structs::FileId>
    // {
    //     let path = std::path::PathBuf::from(path);
    //     let file = std::fs::File::open(&path)?;
    //     let size = file.metadata()?.len();
    //     self.call(walle_core::action::UploadFileFragmented::Prepare { 
    //         name: path
    //             .file_name()
    //             .unwrap_or_default(),
    //         total_size: size as i64 }).await?;
    //     todo!()
    // }
}

impl<T: ActionCaller> ActionCallerExt for T {}
