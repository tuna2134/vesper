mod parse_impl;

pub mod argument;
pub mod builder;
pub mod command;
pub mod context;
pub mod framework;
pub mod group;
pub mod hook;
pub mod iter;
pub mod message;
pub mod parse;
pub mod waiter;

pub use zephyrus_macros as macros;

type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

/// Useful exports to get started quickly
pub mod prelude {
    pub use crate::{
        builder::{FrameworkBuilder, WrappedClient},
        command::CommandResult,
        context::{AutocompleteContext, SlashContext},
        framework::Framework,
        parse::{Parse, ParseError},
        waiter::WaiterReceiver,
    };
    pub use async_trait::async_trait;
    pub use zephyrus_macros::*;
}

pub mod twilight_exports {
    pub use twilight_http::{
        client::{Client, InteractionClient},
        request::application::interaction::UpdateResponse,
    };
    pub use twilight_model::{
        application::{
            command::{
                BaseCommandOptionData, ChannelCommandOptionData, ChoiceCommandOptionData, Command,
                CommandOption, CommandOptionChoice, CommandOptionType, NumberCommandOptionData,
                OptionsCommandOptionData,
            },
            interaction::{
                application_command::{
                    ApplicationCommand, CommandData, CommandDataOption, CommandOptionValue,
                },
                application_command_autocomplete::{
                    ApplicationCommandAutocomplete, ApplicationCommandAutocompleteDataOptionType,
                },
                message_component::MessageComponentInteraction,
                Interaction,
            },
        },
        channel::Message,
        gateway::payload::incoming::InteractionCreate,
        guild::Permissions,
        http::interaction::{
            InteractionResponse, InteractionResponseData, InteractionResponseType,
        },
        id::{
            marker::{
                ApplicationMarker, ChannelMarker, GenericMarker, GuildMarker, MessageMarker,
                RoleMarker, UserMarker,
            },
            Id,
        },
    };
}
