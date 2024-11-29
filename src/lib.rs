pub mod export;

use async_trait::async_trait;
use deppy::ServiceHandler;
use snafu::Snafu;
use std::any::TypeId;
use std::collections::HashMap;
use std::error::Error as ErrorTrait;
use twilight_model::application::command::CommandOptionType;
use twilight_model::application::interaction::application_command::{
    CommandData, CommandDataOption, CommandOptionValue,
};
use twilight_model::application::interaction::InteractionData;
use twilight_model::gateway::payload::incoming::InteractionCreate;
use twilight_model::id::marker::{AttachmentMarker, ChannelMarker, GenericMarker, RoleMarker};
use twilight_model::id::Id;
use twilight_util::builder::command::CommandBuilder;

#[async_trait]
pub trait CommandController {
    async fn execute_command(
        &self,
        interaction: &InteractionCreate,
        data: &CommandData,
    ) -> Result<(), Error>;

    fn command_names() -> Vec<String>
    where
        Self: Sized,
    {
        vec![]
    }

    #[cfg(feature = "register")]
    fn create_commands() -> Vec<CommandBuilder>
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

pub struct CommandHandler {
    commands: HashMap<String, TypeId>,
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

impl CommandHandler {
    pub async fn handle_command_interaction<T: ServiceHandler>(
        &self,
        interaction: &InteractionCreate,
        handler: &T,
    ) -> Result<(), Error> {
        let data = match &interaction.data {
            Some(InteractionData::ApplicationCommand(ap)) => ap,
            _ => return Err(Error::NotApplicationCommand),
        };

        let type_id = match self.commands.get(&data.name) {
            Some(t) => t,
            None => return Err(Error::CommandNotFound),
        };

        let scope = handler.create_scope();
        let command_controller_arc = match scope.get_service_by_type_id(type_id) {
            Some(c) => c,
            None => return Err(Error::CommandNotFound),
        };

        let command_controller =
            match command_controller_arc.downcast_ref::<&dyn CommandController>() {
                Some(c) => c,
                None => return Err(Error::CommandNotFound), // TODO: Make it into an error
            };

        command_controller.execute_command(interaction, data).await
    }
}
