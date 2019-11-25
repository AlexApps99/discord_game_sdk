use crate::{across_ffi, event, sys, to_result::ToResult, CreateFlags, Discord, Result};
use std::ffi::c_void;

/// # Core
///
/// <https://discordapp.com/developers/docs/game-sdk/discord>
impl<'a> Discord<'a> {
    /// Calls [`with_create_flags`](#method.with_create_flags)
    /// with [`CreateFlags::Default`](enum.CreateFlags.html#variant.Default).
    pub fn new(client_id: i64) -> Result<Self> {
        Self::with_create_flags(client_id, CreateFlags::Default)
    }

    /// Creates an instance of the main interface with the Discord Game SDK.
    /// It also forwards all log messages to [`log`](https://docs.rs/log)
    /// and kickstarts all managers that produce events when started.
    ///
    /// <https://discordapp.com/developers/docs/game-sdk/discord#create>  
    /// <https://discordapp.com/developers/docs/game-sdk/discord#setloghook>
    pub fn with_create_flags(client_id: i64, flags: CreateFlags) -> Result<Self> {
        let (senders, receivers) = event::create_channels();
        let senders_ptr = Box::into_raw(Box::new(senders));
        let senders = unsafe { Box::from_raw(senders_ptr) };

        let mut params = create_params(client_id, flags, senders_ptr as *mut _);

        let mut core = std::ptr::null_mut();

        unsafe { sys::DiscordCreate(sys::DISCORD_VERSION, &mut params, &mut core) }.to_result()?;

        log::trace!("received pointer to {:p}", core);

        let instance = Self {
            core,
            client_id,
            senders,
            receivers,
            callbacks: Vec::new(),
        };

        instance.set_log_hook();
        instance.kickstart_managers();

        Ok(instance)
    }

    fn set_log_hook(&self) {
        unsafe {
            ffi!(self.set_log_hook(
                sys::DiscordLogLevel_Debug,
                std::ptr::null_mut(),
                Some(across_ffi::callbacks::log),
            ))
        };
    }

    fn kickstart_managers(&self) {
        unsafe {
            // In this order to prioritize managers that instantly generate events
            ffi!(self.get_network_manager());
            ffi!(self.get_overlay_manager());
            ffi!(self.get_relationship_manager());
            ffi!(self.get_user_manager());

            ffi!(self.get_achievement_manager());
            ffi!(self.get_activity_manager());
            ffi!(self.get_lobby_manager());
            ffi!(self.get_store_manager());
            ffi!(self.get_voice_manager());
        }
    }

    /// Runs all pending SDK callbacks.
    ///
    /// Will return [`Error::NotRunning`](enum.Error.html#variant.NotRunning)
    /// if the Discord client was closed.
    ///
    /// ## Attention
    /// Event buffers will grow large if `run_callbacks` is not ran often, it is recommended to run
    /// this in the game loop.
    pub fn run_callbacks(&mut self) -> Result<()> {
        unsafe { ffi!(self.run_callbacks()) }.to_result()?;

        // https://github.com/rust-lang/rust/issues/43244
        let mut i = 0;
        while i < self.callbacks.len() {
            if self.callbacks[i].is_ready() {
                let mut callback = self.callbacks.remove(i);
                callback.run(self);
            } else {
                i += 1;
            }
        }

        Ok(())
    }

    /// This will discard all events that have not been used.
    ///
    /// ## Attention
    /// Event buffers will grow large if not all used or emptied.
    ///
    /// As a rule of thumb, call `empty_event_buffers` before every `run_callbacks`.
    pub fn empty_event_buffers(&self) {
        self.receivers.empty_channels()
    }
}

impl<'a> Drop for Discord<'a> {
    fn drop(&mut self) {
        unsafe { ffi!(self.destroy()) }
    }
}

fn create_params(
    client_id: i64,
    flags: CreateFlags,
    event_data: *mut c_void,
) -> sys::DiscordCreateParams {
    let flags: sys::EDiscordCreateFlags = flags.into();

    sys::DiscordCreateParams {
        client_id,
        flags: flags as u64,

        events: std::ptr::null_mut(),
        event_data,

        application_events: ACHIEVEMENT as *const _ as *mut _,
        application_version: sys::DISCORD_APPLICATION_MANAGER_VERSION,

        user_events: USER as *const _ as *mut _,
        user_version: sys::DISCORD_USER_MANAGER_VERSION,

        image_events: std::ptr::null_mut(),
        image_version: sys::DISCORD_IMAGE_MANAGER_VERSION,

        activity_events: ACTIVITY as *const _ as *mut _,
        activity_version: sys::DISCORD_ACTIVITY_MANAGER_VERSION,

        relationship_events: RELATIONSHIP as *const _ as *mut _,
        relationship_version: sys::DISCORD_RELATIONSHIP_MANAGER_VERSION,

        lobby_events: LOBBY as *const _ as *mut _,
        lobby_version: sys::DISCORD_LOBBY_MANAGER_VERSION,

        network_events: NETWORK as *const _ as *mut _,
        network_version: sys::DISCORD_NETWORK_MANAGER_VERSION,

        overlay_events: OVERLAY as *const _ as *mut _,
        overlay_version: sys::DISCORD_OVERLAY_MANAGER_VERSION,

        storage_events: std::ptr::null_mut(),
        storage_version: sys::DISCORD_STORAGE_MANAGER_VERSION,

        store_events: STORE as *const _ as *mut _,
        store_version: sys::DISCORD_STORE_MANAGER_VERSION,

        voice_events: VOICE as *const _ as *mut _,
        voice_version: sys::DISCORD_VOICE_MANAGER_VERSION,

        achievement_events: std::ptr::null_mut(),
        achievement_version: sys::DISCORD_ACHIEVEMENT_MANAGER_VERSION,
    }
}

const ACHIEVEMENT: &sys::IDiscordAchievementEvents = &sys::IDiscordAchievementEvents {
    on_user_achievement_update: Some(across_ffi::achievements::on_user_achievement_update),
};

const ACTIVITY: &sys::IDiscordActivityEvents = &sys::IDiscordActivityEvents {
    on_activity_join: Some(across_ffi::activities::on_activity_join),
    on_activity_spectate: Some(across_ffi::activities::on_activity_spectate),
    on_activity_join_request: Some(across_ffi::activities::on_activity_join_request),
    on_activity_invite: Some(across_ffi::activities::on_activity_invite),
};

const LOBBY: &sys::IDiscordLobbyEvents = &sys::IDiscordLobbyEvents {
    on_lobby_update: Some(across_ffi::lobbies::on_lobby_update),
    on_lobby_delete: Some(across_ffi::lobbies::on_lobby_delete),
    on_member_connect: Some(across_ffi::lobbies::on_member_connect),
    on_member_update: Some(across_ffi::lobbies::on_member_update),
    on_member_disconnect: Some(across_ffi::lobbies::on_member_disconnect),
    on_lobby_message: Some(across_ffi::lobbies::on_lobby_message),
    on_speaking: Some(across_ffi::lobbies::on_speaking),
    on_network_message: Some(across_ffi::lobbies::on_network_message),
};

const NETWORK: &sys::IDiscordNetworkEvents = &sys::IDiscordNetworkEvents {
    on_message: Some(across_ffi::networking::on_message),
    on_route_update: Some(across_ffi::networking::on_route_update),
};

const OVERLAY: &sys::IDiscordOverlayEvents = &sys::IDiscordOverlayEvents {
    on_toggle: Some(across_ffi::overlay::on_toggle),
};

const RELATIONSHIP: &sys::IDiscordRelationshipEvents = &sys::IDiscordRelationshipEvents {
    on_refresh: Some(across_ffi::relationships::on_refresh),
    on_relationship_update: Some(across_ffi::relationships::on_relationship_update),
};

const STORE: &sys::IDiscordStoreEvents = &sys::IDiscordStoreEvents {
    on_entitlement_create: Some(across_ffi::store::on_entitlement_create),
    on_entitlement_delete: Some(across_ffi::store::on_entitlement_delete),
};

const USER: &sys::IDiscordUserEvents = &sys::IDiscordUserEvents {
    on_current_user_update: Some(across_ffi::users::on_current_user_update),
};

const VOICE: &sys::IDiscordVoiceEvents = &sys::IDiscordVoiceEvents {
    on_settings_update: Some(across_ffi::voice::on_settings_update),
};
