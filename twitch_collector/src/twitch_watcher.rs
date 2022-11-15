use std::fmt::{Debug, Display};

use common::{
    models::{
        message::MsgType,
        resub::{NewResub, Tier},
    },
    services::{
        channels::{create_channel_if_not_exists, get_channel_by_twitch_id},
        messages::create_message,
    },
};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use error_stack::{IntoReport, Report, ResultExt};
use tokio::spawn;
use twitch_api2::{
    twitch_oauth2::{AppAccessToken, ClientId, ClientSecret, Scope},
    HelixClient,
};
use twitch_irc::{
    login::StaticLoginCredentials,
    message::{PrivmsgMessage, ServerMessage, UserNoticeEvent, UserNoticeMessage},
    ClientConfig, SecureTCPTransport, TwitchIRCClient,
};

type DbPool = Pool<ConnectionManager<PgConnection>>;
type PooledConnection = diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Debug)]
pub enum RunError {
    HandleError,
    GetTokenError,
    ApiError,
    ChannelNotExists(String),
    DbPoolError,
    DatabaseError,
}

impl Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunError::HandleError => write!(f, "Unexpected Error!"),
            RunError::GetTokenError => write!(f, "Couldn't get token from twitch"),
            RunError::ApiError => write!(f, "Couldn't get info from twitch api"),
            RunError::ChannelNotExists(name) => {
                write!(f, "Couldn't find channel with name \"{name}\"")
            }
            RunError::DbPoolError => write!(f, "Couldn't get connection to database from pool"),
            RunError::DatabaseError => write!(f, "Database error"),
        }
    }
}

impl std::error::Error for RunError {}

// will return only on error
pub async fn run(
    pool: DbPool,
    helix_client: HelixClient<'static, reqwest::Client>,
) -> error_stack::Result<(), RunError> {
    let client_config = ClientConfig::default();

    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(client_config);

    let create_channels_db = pool.clone();

    let handle = spawn(async move {
        let pool = pool.clone();
        while let Some(message) = incoming_messages.recv().await {
            info!("recieved message {:?}", message);

            let Ok(conn) = pool.get() else {
                println!("Database error for message. Check logs for details");
                error!("Database error while getting connection to database from pool");
                continue;
            };

            tokio::spawn(async move {
                handle_message(message, conn).await;
            });
        }
    });

    let config = get_config_async!().await;

    let token = AppAccessToken::get_app_access_token(
        &helix_client,
        ClientId::new(config.twitchapi.clientid.clone()),
        ClientSecret::new(config.twitchapi.clientsecret.clone()),
        Scope::all(),
    )
    .await
    .into_report()
    .change_context(RunError::GetTokenError)?;

    // TODO maybe get rid of channels in config?
    // or add another method of adding
    // will probably do when I do api crate and there add method of adding channel
    for channel in get_config_async!().await.channels.iter() {
        info!("Joining channel {}", &channel);
        let channel_info = &helix_client
            .get_channel_from_login(&channel[..], &token)
            .await
            .into_report()
            .change_context(RunError::ApiError)?
            .ok_or_else(|| Report::new(RunError::ChannelNotExists(channel.clone())))?;

        let db_conn = &create_channels_db
            .get()
            .into_report()
            .change_context(RunError::DbPoolError)?;

        create_channel_if_not_exists(
            &db_conn,
            channel_info.broadcaster_id.clone().into_string(),
            channel.clone(),
        )
        .change_context(RunError::DatabaseError)?;

        client.join(channel.clone()).expect("lol");
    }

    handle
        .await
        .into_report()
        .change_context(RunError::HandleError)
        .attach_printable("Handle returned an error")?;

    Err(Report::new(RunError::HandleError).attach_printable("Handle run joined to main thread"))
}

async fn handle_message(message: ServerMessage, db_conn: PooledConnection) {
    match message {
        ServerMessage::Privmsg(msg) => {
            handle_priv_msg(msg, db_conn);
        }
        ServerMessage::UserNotice(user_notice) => {
            // user notice can be subs, resubs, raids etc.
            handle_user_notice(user_notice, db_conn);
        }
        _ => {}
    }
}

fn handle_priv_msg(msg: PrivmsgMessage, db_conn: PooledConnection) {
    let channel = get_channel_by_twitch_id(&db_conn, &msg.channel_id);

    let channel = match channel {
        Ok(v) => v,
        Err(err) => {
            log::error!("{err}");
            return;
        }
    };

    let Some(channel) = channel else {
        log::error!("error getting channel");
        return;
    };

    let msg_type = get_msg_type_from_privmsg(&msg);

    let message = create_message(
        &db_conn,
        msg.message_text,
        msg_type,
        channel.id,
        msg.server_timestamp,
        msg.bits.map(|bits| bits as i64),
        None,
        msg.sender.id,
        msg.sender.login,
        msg.sender.name,
    );

    if let Err(err) = message {
        log::error!("couldn't save message!! err: {err}");
    }
}

fn handle_user_notice(user_notice: UserNoticeMessage, db_conn: PooledConnection) {
    match user_notice.event {
        UserNoticeEvent::SubOrResub {
            is_resub,
            cumulative_months,
            sub_plan,
            ..
        } => {
            if !is_resub {
                return;
            }

            let Some(msg) = user_notice.message_text else {
                return;
            };

            let channel = get_channel_by_twitch_id(&db_conn, &user_notice.channel_id);

            let channel = match channel {
                Ok(v) => v,
                Err(err) => {
                    log::error!("{err}");
                    return;
                }
            };

            let Some(channel) = channel else {
                log::error!("error getting channel");
                return;
            };

            let tier = match Tier::try_from(&sub_plan[..]) {
                Ok(tier) => tier,
                Err(err) => {
                    println!(
                        "Error: Either data from twitch is invalid or twitch have updated their api. Data: {}",
                        sub_plan
                    );
                    log::error!(
                        "Error: Either data from twitch is invalid or twitch have updated their api. Data: {}; error: {}",
                        sub_plan,
                        err
                    );
                    return;
                }
            };

            let new_resub = NewResub {
                cumulative_month: cumulative_months as i16,
                tier: tier,
            };

            let message = create_message(
                &db_conn,
                msg,
                MsgType::Sub,
                channel.id,
                user_notice.server_timestamp,
                None,
                Some(new_resub),
                user_notice.sender.id,
                user_notice.sender.login,
                user_notice.sender.name,
            );

            if let Err(err) = message {
                log::error!("couldn't save message!! error: {err}");
            }
        }
        _ => {}
    }
}

fn get_msg_type_from_privmsg(msg: &PrivmsgMessage) -> MsgType {
    if msg.is_action {
        return MsgType::Action;
    }

    if msg.bits.is_some() {
        return MsgType::Bits;
    }

    return MsgType::Message;
}
