use twilight_model::{
    application::command::CommandOption,
    id::{
        marker::{AttachmentMarker, ChannelMarker, GenericMarker, RoleMarker, UserMarker},
        Id,
    },
};

use twilight_util::builder::command::{
    AttachmentBuilder, BooleanBuilder, ChannelBuilder, IntegerBuilder, MentionableBuilder,
    NumberBuilder, RoleBuilder, StringBuilder, UserBuilder,
};

pub trait CreateOption {
    fn create_option(
        name: &str,
        description: &str,
        required: bool,
        choices: Vec<(String, Self)>,

    ) -> CommandOption
    where
        Self: Sized;
}

impl CreateOption for Id<AttachmentMarker> {
    fn create_option(
        name: &str,
        description: &str,
        required: bool,
        _: Vec<(String, Self)>,

    ) -> CommandOption {
        AttachmentBuilder::new(name, description)
            .required(required)
            .build()
    }
}

impl CreateOption for bool {
    fn create_option(
        name: &str,
        description: &str,
        required: bool,
        _: Vec<(String, Self)>,

    ) -> CommandOption {
        BooleanBuilder::new(name, description)
            .required(required)
            .build()
    }
}

impl CreateOption for Id<ChannelMarker> {
    fn create_option(
        name: &str,
        description: &str,
        required: bool,
        _: Vec<(String, Self)>,

    ) -> CommandOption {
        ChannelBuilder::new(name, description)
            .required(required)
            .build()
    }
}

impl CreateOption for i64 {
    fn create_option(
        name: &str,
        description: &str,
        required: bool,
        choices: Vec<(String, Self)>,

    ) -> CommandOption {
        IntegerBuilder::new(name, description)
            .required(required)
            .choices(choices)
            .build()
    }
}

impl CreateOption for Id<GenericMarker> {
    fn create_option(
        name: &str,
        description: &str,
        required: bool,
        _: Vec<(String, Self)>,

    ) -> CommandOption {
        MentionableBuilder::new(name, description)
            .required(required)
            .build()
    }
}

impl CreateOption for f64 {
    fn create_option(
        name: &str,
        description: &str,
        required: bool,
        choices: Vec<(String, Self)>,

    ) -> CommandOption {
        NumberBuilder::new(name, description)
            .required(required)
            .choices(choices)
            .build()
    }
}

impl CreateOption for Id<RoleMarker> {
    fn create_option(
        name: &str,
        description: &str,
        required: bool,
        _: Vec<(String, Self)>,

    ) -> CommandOption {
        RoleBuilder::new(name, description)
            .required(required)
            .build()
    }
}

impl CreateOption for String {
    fn create_option(
        name: &str,
        description: &str,
        required: bool,
        choices: Vec<(String, Self)>,

    ) -> CommandOption {
        StringBuilder::new(name, description)
            .required(required)
            .choices(choices)
            .build()
    }
}

impl CreateOption for Id<UserMarker> {
    fn create_option(
        name: &str,
        description: &str,
        required: bool,
        _: Vec<(String, Self)>,

    ) -> CommandOption {
        UserBuilder::new(name, description)
            .required(required)
            .build()
    }
}
