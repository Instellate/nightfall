use anyhow::anyhow;
use deppy::{Dep, ServiceCollectionBuilder, ServiceHandler};
use deppy_macros::Injectable;
use nightfall::services::AddTwilightServices;
use nightfall::{CommandController, CommandHandler};
use nightfall_macros::{command, command_controller};
use serde::Deserialize;
use std::env;
use std::error::Error;
use std::fs::File;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::{Event, Shard};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::payload::incoming::InteractionCreate;
use twilight_model::gateway::{Intents, ShardId};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::marker::UserMarker;
use twilight_model::id::Id;
use twilight_util::builder::InteractionResponseDataBuilder;

#[derive(Injectable)]
struct Test {
    client: Dep<HttpClient>,
    cache: Dep<InMemoryCache>,
}

#[command_controller]
impl Test {
    #[command(
        description = "User command, it's funny",
        option(name = "user", description = "The user you wanna funny to")
    )]
    async fn user(
        &self,
        interaction: &InteractionCreate,
        user_id: Id<UserMarker>,
    ) -> Result<(), Box<dyn Error>> {
        let user = self
            .cache
            .user(user_id)
            .ok_or(anyhow!("Couldn't find user"))?;

        let data = InteractionResponseDataBuilder::new()
            .content(format!("User {} is super funny today", user.name))
            .build();

        let response: InteractionResponse = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };

        self.client
            .interaction(Id::new(813708786493161523))
            .create_response(interaction.id, &interaction.token, &response)
            .await?;

        Ok(())
    }

    #[command(description = "User command, it's funny")]
    async fn member(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    #[command(
        name = "echo",
        description = "Echos a given set of choices",
        option(
            description = "The message to echo",
            choice(name = "Hello", value = "Hi"),
            choice(name = "World", value = "Heaven")
        )
    )]
    async fn echo(
        &self,
        interaction: &InteractionCreate,
        message: String,
    ) -> Result<(), Box<dyn Error>> {
        let data = InteractionResponseDataBuilder::new()
            .content(message)
            .build();

        let response: InteractionResponse = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };

        self.client
            .interaction(Id::new(813708786493161523))
            .create_response(interaction.id, &interaction.token, &response)
            .await?;
        Ok(())
    }
}

#[derive(Injectable)]
struct TestSub {
    client: Dep<HttpClient>,
}

#[command_controller(sub = "paru", sub_description = "Emulates paru")]
impl TestSub {
    #[command(
        name = "install",
        description = "Emulate installing a package",
        option(name = "name", description = "The package name to install")
    )]
    async fn install(
        &self,
        interaction: &InteractionCreate,
        name: String,
    ) -> Result<(), Box<dyn Error>> {
        let data = InteractionResponseDataBuilder::new()
            .content(format!("Installing package {}...", name))
            .build();

        let response_msg: InteractionResponse = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };

        self
            .client
            .interaction(Id::new(813708786493161523))
            .create_response(interaction.id, &interaction.token, &response_msg)
            .await?;

        let sleep_time = tokio::time::Duration::from_secs(5);
        tokio::time::sleep(sleep_time).await;

        self.client
            .interaction(Id::new(813708786493161523))
            .create_followup(&interaction.token)
            .content(&format!("Installed package {}!", name))?
            .await?;

        Ok(())
    }
}

#[derive(Deserialize)]
struct Config {
    token: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config: Config = {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| String::from("./config.json"));
        let f = File::open(config_path)?;
        serde_json::from_reader(f)?
    };

    let collection = ServiceCollectionBuilder::default()
        .add_http_client(config.token.clone())
        .add_in_memory_cache()
        .add_scoped::<Test>()
        .add_scoped::<TestSub>()
        .build();

    let command_handler = CommandHandler::new()
        .add_command::<Test>()
        .add_command::<TestSub>();

    let mut shard = Shard::new(ShardId::ONE, config.token.clone(), Intents::GUILDS);

    let application_id = {
        let client = collection.get_required_service::<HttpClient>();
        let response = client.current_user_application().await?;

        response.model().await?.id
    };

    {
        let mut commands = Test::build_commands();
        let mut sub_commands = TestSub::build_commands();
        commands.append(&mut sub_commands);

        let client = collection.get_required_service::<HttpClient>();
        client
            .interaction(application_id)
            .set_global_commands(&commands)
            .await?;
    }

    loop {
        let event = shard.next_event().await?;
        let cache = collection.get_required_service::<InMemoryCache>();
        cache.update(&event);

        let Event::InteractionCreate(interacton) = event else {
            continue;
        };

        let err = command_handler
            .handle_command_interaction(&interacton, &collection)
            .await;
        if err.is_err() {
            let client = collection.get_required_service::<HttpClient>();

            println!("Error: {:#?}", err);

            let response_data = InteractionResponseDataBuilder::new()
                .content("Uh oh, something happened...")
                .build();

            let response: InteractionResponse = InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(response_data),
            };

            client
                .interaction(application_id)
                .create_response(interacton.id, &interacton.token, &response)
                .await?;
        }
    }
}
