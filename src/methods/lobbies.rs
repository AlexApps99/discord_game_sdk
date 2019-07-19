use crate::{
    callbacks::{ResultCallback, ResultFromPtrCallback},
    sys,
    to_result::ToResult,
    utils::cstr_to_str,
    Discord, Lobby, LobbyMemberTransaction, LobbyTransaction, Result, SearchQuery,
};
use std::collections::HashMap;
use std::ffi::CStr;
use std::mem::size_of;

/// # Lobbies
///
/// <https://discordapp.com/developers/docs/game-sdk/lobbies>
impl<'a> Discord<'a> {
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#createlobby>
    pub fn create_lobby(
        &mut self,
        builder: LobbyTransaction,
        mut callback: impl FnMut(&mut Discord, Result<Lobby>) + 'a,
    ) {
        let mut ptr = std::ptr::null_mut();

        if let Err(e) = unsafe {
            ffi!(self
                .get_lobby_manager()
                .get_lobby_create_transaction(&mut ptr))
            .to_result()
        } {
            return callback(self, Err(e));
        }

        if let Err(e) = unsafe { builder.process(ptr) } {
            return callback(self, Err(e));
        }

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .create_lobby(ptr)
                .and_then(ResultFromPtrCallback::new(callback)))
        }
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#updatelobby>
    pub fn update_lobby(
        &mut self,
        lobby_id: i64,
        builder: LobbyTransaction,
        mut callback: impl FnMut(&mut Discord, Result<()>) + 'a,
    ) {
        let mut ptr = std::ptr::null_mut();

        if let Err(e) = unsafe {
            ffi!(self
                .get_lobby_manager()
                .get_lobby_update_transaction(lobby_id, &mut ptr))
            .to_result()
        } {
            return callback(self, Err(e));
        }

        if let Err(e) = unsafe { builder.process(ptr) } {
            return callback(self, Err(e));
        }

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .update_lobby(lobby_id, ptr)
                .and_then(ResultCallback::new(callback)))
        }
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#deletelobby>
    pub fn delete_lobby(
        &mut self,
        lobby_id: i64,
        callback: impl FnMut(&mut Discord, Result<()>) + 'a,
    ) {
        unsafe {
            ffi!(self
                .get_lobby_manager()
                .delete_lobby(lobby_id)
                .and_then(ResultCallback::new(callback)))
        }
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#connectlobby>
    pub fn connect_lobby(
        &mut self,
        lobby_id: i64,
        secret: impl AsRef<CStr>,
        callback: impl FnMut(&mut Discord, Result<Lobby>) + 'a,
    ) {
        unsafe {
            ffi!(self
                .get_lobby_manager()
                .connect_lobby(lobby_id, secret.as_ref().as_ptr() as *mut _)
                .and_then(ResultFromPtrCallback::new(callback)))
        }
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#connectlobbywithactivitysecret>
    pub fn connect_lobby_with_activity_secret(
        &mut self,
        activity_secret: impl AsRef<CStr>,
        callback: impl FnMut(&mut Discord, Result<Lobby>) + 'a,
    ) {
        unsafe {
            ffi!(self
                .get_lobby_manager()
                .connect_lobby_with_activity_secret(activity_secret.as_ref().as_ptr() as *mut _)
                .and_then(ResultFromPtrCallback::new(callback)))
        }
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#disconnectlobby>
    pub fn disconnect_lobby(
        &mut self,
        lobby_id: i64,
        callback: impl FnMut(&mut Discord, Result<()>) + 'a,
    ) {
        unsafe {
            ffi!(self
                .get_lobby_manager()
                .disconnect_lobby(lobby_id)
                .and_then(ResultCallback::new(callback)))
        }
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getlobby>
    pub fn lobby(&mut self, lobby_id: i64) -> Result<Lobby> {
        let mut lobby = sys::DiscordLobby::default();
        unsafe {
            ffi!(self
                .get_lobby_manager()
                .get_lobby(lobby_id, &mut lobby as *mut _))
        }
        .to_result()?;

        Ok(Lobby::from(lobby))
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getlobbyactivitysecret>
    pub fn lobby_activity_secret(&mut self, lobby_id: i64) -> Result<String> {
        let mut secret: sys::DiscordLobbySecret = [0; size_of::<sys::DiscordLobbySecret>()];

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .get_lobby_activity_secret(lobby_id, &mut secret as *mut _))
        }
        .to_result()?;

        Ok(cstr_to_str(&secret[..]).to_string())
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getlobbymetadatavalue>
    pub fn lobby_metadata(&mut self, lobby_id: i64, key: impl AsRef<CStr>) -> Result<String> {
        let mut value: sys::DiscordMetadataValue = [0; size_of::<sys::DiscordMetadataValue>()];

        unsafe {
            ffi!(self.get_lobby_manager().get_lobby_metadata_value(
                lobby_id,
                key.as_ref().as_ptr() as *mut _,
                &mut value as *mut _
            ))
        }
        .to_result()?;

        Ok(cstr_to_str(&value[..]).to_string())
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#lobbymetadatacount>  
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getlobbymetadatakey>  
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getlobbymetadatavalue>
    pub fn all_lobby_metadata(&mut self, lobby_id: i64) -> Result<HashMap<String, String>> {
        let mut count: i32 = 0;

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .lobby_metadata_count(lobby_id, &mut count as *mut _))
        }
        .to_result()?;

        let mut res = HashMap::with_capacity(count as usize);
        let mut key: sys::DiscordMetadataKey = [0; size_of::<sys::DiscordMetadataKey>()];
        let mut value: sys::DiscordMetadataValue = [0; size_of::<sys::DiscordMetadataValue>()];

        for index in 0..count {
            unsafe {
                ffi!(self.get_lobby_manager().get_lobby_metadata_key(
                    lobby_id,
                    index as i32,
                    &mut key as *mut _
                ))
            }
            .to_result()?;

            unsafe {
                ffi!(self.get_lobby_manager().get_lobby_metadata_value(
                    lobby_id,
                    key.as_mut_ptr(),
                    &mut value as *mut _
                ))
            }
            .to_result()?;

            let _ = res.insert(
                cstr_to_str(&key[..]).to_string(),
                cstr_to_str(&value[..]).to_string(),
            );
        }

        Ok(res)
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#updatemember>
    pub fn update_member(
        &mut self,
        lobby_id: i64,
        user_id: i64,
        builder: LobbyMemberTransaction,
        mut callback: impl FnMut(&mut Discord, Result<()>) + 'a,
    ) {
        let mut ptr = std::ptr::null_mut();

        if let Err(e) = unsafe {
            ffi!(self
                .get_lobby_manager()
                .get_member_update_transaction(lobby_id, user_id, &mut ptr))
            .to_result()
        } {
            return callback(self, Err(e));
        }

        if let Err(e) = unsafe { builder.process(ptr) } {
            return callback(self, Err(e));
        }

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .update_member(lobby_id, user_id, ptr)
                .and_then(ResultCallback::new(callback)))
        }
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#membercount>  
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getmemberuserid>
    pub fn all_lobby_member_ids(&mut self, lobby_id: i64) -> Result<Vec<i64>> {
        let mut count = 0;

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .member_count(lobby_id, &mut count as *mut _))
        }
        .to_result()?;

        let mut result = Vec::with_capacity(count as usize);
        let mut user_id = 0;

        for index in 0..count {
            unsafe {
                ffi!(self.get_lobby_manager().get_member_user_id(
                    lobby_id,
                    index,
                    &mut user_id as *mut _
                ))
            }
            .to_result()?;

            result.push(user_id)
        }

        Ok(result)
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#membermetadatacount>  
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getmembermetadatakey>  
    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#getmembermetadatavalue>
    pub fn all_lobby_member_metadata(
        &mut self,
        lobby_id: i64,
        user_id: i64,
    ) -> Result<HashMap<String, String>> {
        let mut count: i32 = 0;

        unsafe {
            ffi!(self.get_lobby_manager().member_metadata_count(
                lobby_id,
                user_id,
                &mut count as *mut _
            ))
        }
        .to_result()?;

        let mut res = HashMap::with_capacity(count as usize);
        let mut key: sys::DiscordMetadataKey = [0; size_of::<sys::DiscordMetadataKey>()];
        let mut value: sys::DiscordMetadataValue = [0; size_of::<sys::DiscordMetadataValue>()];

        for index in 0..count {
            unsafe {
                ffi!(self.get_lobby_manager().get_member_metadata_key(
                    lobby_id,
                    user_id,
                    index as i32,
                    &mut key as *mut _
                ))
            }
            .to_result()?;

            unsafe {
                ffi!(self.get_lobby_manager().get_member_metadata_value(
                    lobby_id,
                    user_id,
                    key.as_mut_ptr(),
                    &mut value as *mut _
                ))
            }
            .to_result()?;

            let _ = res.insert(
                cstr_to_str(&key[..]).to_string(),
                cstr_to_str(&value[..]).to_string(),
            );
        }

        Ok(res)
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#sendlobbymessage>
    pub fn send_lobby_message(
        &mut self,
        lobby_id: i64,
        buf: impl AsRef<[u8]>,
        callback: impl FnMut(&mut Discord, Result<()>) + 'a,
    ) {
        let buf = buf.as_ref();
        assert!(buf.len() <= u32::max_value() as usize);

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .send_lobby_message(lobby_id, buf.as_ptr() as *mut _, buf.len() as u32)
                .and_then(ResultCallback::new(callback)))
        }
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#search>
    pub fn lobby_search(
        &mut self,
        builder: SearchQuery,
        mut callback: impl FnMut(&mut Discord, Result<Vec<i64>>) + 'a,
    ) {
        let mut ptr = std::ptr::null_mut();

        if let Err(e) =
            unsafe { ffi!(self.get_lobby_manager().get_search_query(&mut ptr)).to_result() }
        {
            return callback(self, Err(e));
        }

        if let Err(e) = unsafe { builder.process(ptr) } {
            return callback(self, Err(e));
        }

        let inner = move |gsdk: &mut Discord, res: Result<()>| {
            if let Err(e) = res {
                return callback(gsdk, Err(e));
            }

            let mut count = 0;

            unsafe { ffi!(gsdk.get_lobby_manager().lobby_count(&mut count)) }

            let mut vec = Vec::with_capacity(count as usize);
            let mut lobby_id = 0;

            for index in 0..count {
                let res =
                    unsafe { ffi!(gsdk.get_lobby_manager().get_lobby_id(index, &mut lobby_id)) }
                        .to_result();

                if let Err(e) = res {
                    return callback(gsdk, Err(e));
                }

                vec.push(lobby_id);
            }

            callback(gsdk, Ok(vec))
        };

        unsafe {
            ffi!(self
                .get_lobby_manager()
                .search(ptr)
                .and_then(ResultCallback::new(inner)))
        }
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#connectvoice>
    pub fn connect_lobby_voice(
        &mut self,
        lobby_id: i64,
        callback: impl FnMut(&mut Discord, Result<()>) + 'a,
    ) {
        unsafe {
            ffi!(self
                .get_lobby_manager()
                .connect_voice(lobby_id)
                .and_then(ResultCallback::new(callback)))
        }
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#disconnectvoice>
    pub fn disconnect_lobby_voice(
        &mut self,
        lobby_id: i64,
        callback: impl FnMut(&mut Discord, Result<()>) + 'a,
    ) {
        unsafe {
            ffi!(self
                .get_lobby_manager()
                .disconnect_voice(lobby_id)
                .and_then(ResultCallback::new(callback)))
        }
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#connectnetwork>
    pub fn connect_lobby_network(&mut self, lobby_id: i64) -> Result<()> {
        unsafe { ffi!(self.get_lobby_manager().connect_network(lobby_id,)) }.to_result()
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#disconnectnetwork>
    pub fn disconnect_lobby_network(&mut self, lobby_id: i64) -> Result<()> {
        unsafe { ffi!(self.get_lobby_manager().disconnect_network(lobby_id,)) }.to_result()
    }

    pub fn flush_lobby_network(&mut self) -> Result<()> {
        unsafe { ffi!(self.get_lobby_manager().flush_network()) }.to_result()
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#opennetworkchannel>
    pub fn open_lobby_network_channel(
        &mut self,
        lobby_id: i64,
        channel_id: u8,
        reliable: bool,
    ) -> Result<()> {
        unsafe {
            ffi!(self
                .get_lobby_manager()
                .open_network_channel(lobby_id, channel_id, reliable))
        }
        .to_result()
    }

    /// <https://discordapp.com/developers/docs/game-sdk/lobbies#sendnetworkmessage>
    pub fn send_lobby_network_message(
        &mut self,
        lobby_id: i64,
        user_id: i64,
        channel_id: u8,
        buf: &[u8],
    ) -> Result<()> {
        assert!(buf.len() <= u32::max_value() as usize);

        unsafe {
            ffi!(self.get_lobby_manager().send_network_message(
                lobby_id,
                user_id,
                channel_id,
                buf.as_ptr() as *mut _,
                buf.len() as u32
            ))
        }
        .to_result()
    }
}
