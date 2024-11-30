pub mod export;
pub mod register;
#[cfg(feature = "services")]
pub mod services;

use async_trait::async_trait;
use deppy::ServiceHandler;
use snafu::Snafu;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::error::Error as ErrorTrait;
use std::sync::Arc;
use twilight_model::application::command::{Command, CommandOptionType};
use twilight_model::application::interaction::application_command::{
    CommandData, CommandDataOption, CommandOptionValue,
};
use twilight_model::application::interaction::InteractionData;
use twilight_model::gateway::payload::incoming::InteractionCreate;
use twilight_model::id::marker::{
    AttachmentMarker, ChannelMarker, GenericMarker, RoleMarker, UserMarker,
};
use twilight_model::id::Id;

#[async_trait]
pub trait CommandController {
    async fn execute_command(
        &self,
        interaction: &InteractionCreate,
        data: &CommandData,
    ) -> Result<(), Error>;

    fn get_command_names<'a>() -> &'a [&'static str]
    where
        Self: Sized,
    {
        &[]
    }

    fn build_commands() -> Vec<Command>
    where
        Self: Sized,
    {
        vec![]
    }
}

pub trait FromOption {
    fn from_option(value: CommandOptionValue) -> Option<Self>
    where
        Self: Sized;
}

impl FromOption for Id<AttachmentMarker> {
    fn from_option(value: CommandOptionValue) -> Option<Self> {
        if let CommandOptionValue::Attachment(v) = value {
            Some(v)
        } else {
            None
        }
    }
}

impl FromOption for bool {
    fn from_option(value: CommandOptionValue) -> Option<Self> {
        if let CommandOptionValue::Boolean(v) = value {
            Some(v)
        } else {
            None
        }
    }
}

impl FromOption for Id<ChannelMarker> {
    fn from_option(value: CommandOptionValue) -> Option<Self> {
        if let CommandOptionValue::Channel(v) = value {
            Some(v)
        } else {
            None
        }
    }
}

impl FromOption for (String, CommandOptionType) {
    fn from_option(value: CommandOptionValue) -> Option<Self> {
        if let CommandOptionValue::Focused(v, v2) = value {
            Some((v, v2))
        } else {
            None
        }
    }
}

impl FromOption for i64 {
    fn from_option(value: CommandOptionValue) -> Option<Self> {
        if let CommandOptionValue::Integer(v) = value {
            Some(v)
        } else {
            None
        }
    }
}

impl FromOption for Id<GenericMarker> {
    fn from_option(value: CommandOptionValue) -> Option<Self> {
        if let CommandOptionValue::Mentionable(v) = value {
            Some(v)
        } else {
            None
        }
    }
}

impl FromOption for f64 {
    fn from_option(value: CommandOptionValue) -> Option<Self> {
        if let CommandOptionValue::Number(v) = value {
            Some(v)
        } else {
            None
        }
    }
}

impl FromOption for Id<RoleMarker> {
    fn from_option(value: CommandOptionValue) -> Option<Self> {
        if let CommandOptionValue::Role(v) = value {
            Some(v)
        } else {
            None
        }
    }
}

impl FromOption for String {
    fn from_option(value: CommandOptionValue) -> Option<Self> {
        if let CommandOptionValue::String(v) = value {
            Some(v)
        } else {
            None
        }
    }
}

impl FromOption for Vec<CommandDataOption> {
    fn from_option(value: CommandOptionValue) -> Option<Self> {
        match value {
            CommandOptionValue::SubCommand(v) => Some(v),
            CommandOptionValue::SubCommandGroup(v) => Some(v),
            _ => None,
        }
    }
}

impl FromOption for Id<UserMarker> {
    fn from_option(value: CommandOptionValue) -> Option<Self> {
        if let CommandOptionValue::User(v) = value {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("The interaction is not an application command"))]
    NotApplicationCommand,
    #[snafu(display("Could not find a command for the interaction"))]
    CommandNotFound,
    #[snafu(display("Failed to bind options to the command"))]
    OptionBindingFailed,
    #[snafu(display("The command failed to execute"))]
    CommandError { error: Box<dyn ErrorTrait> },
}

type ConvertFn<T> = fn(&<T as ServiceHandler>::ScopeType) -> Arc<dyn CommandController + 'static>; 

#[derive(Debug)]
pub struct CommandHandler<T: ServiceHandler> {
    commands: HashMap<String, ConvertFn<T>>,
}

impl<T: ServiceHandler> CommandHandler<T> {
    pub fn new() -> Self {
        CommandHandler {
            commands: Default::default(),
        }
    }

    pub fn add_command<C: CommandController + Any + Send + Sync>(mut self) -> Self {
        for name in C::get_command_names() {
            self.commands.insert(name.to_string(), |h: &T::ScopeType| {
                h.get_service_by_type_id(&TypeId::of::<C>())
                    .unwrap()
                    .downcast::<C>()
                    .unwrap() as Arc<dyn CommandController>
            });
        }

        self
    }

    pub async fn handle_command_interaction(
        &self,
        interaction: &InteractionCreate,
        handler: &T,
    ) -> Result<(), Error> {
        let data = match &interaction.data {
            Some(InteractionData::ApplicationCommand(ap)) => ap,
            _ => return Err(Error::NotApplicationCommand),
        };

        let fn_ = match self.commands.get(&data.name) {
            Some(t) => t,
            None => return Err(Error::CommandNotFound),
        };

        let scope = handler.create_scope();
        let command_controller = fn_(&scope);

        command_controller.execute_command(interaction, data).await
    }
}

impl<T: ServiceHandler> Default for CommandHandler<T> {
    fn default() -> Self {
        Self::new()
    }
}
