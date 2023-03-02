use std::fmt::Write as _;

use reedline_repl_rs::clap::{value_parser, Arg, ArgMatches, Command};
use reedline_repl_rs::Repl;
use sifis_api::Sifis;

#[derive(Debug, thiserror::Error)]
enum CliError {
    #[error(transparent)]
    Sifis(#[from] sifis_api::Error),
    #[error(transparent)]
    Repl(#[from] reedline_repl_rs::Error),
}

type Result<T> = std::result::Result<T, CliError>;

struct Ctx {
    sifis: Sifis,
}

async fn list_lamps(_args: ArgMatches, context: &mut Ctx) -> Result<Option<String>> {
    let mut out = String::new();

    for lamp in context.sifis.lamps().await? {
        writeln!(out, " {}", lamp).unwrap();
    }

    Ok(Some(out))
}

async fn update_prompt(_context: &mut Ctx) -> Result<Option<String>> {
    let msg = "Ok";
    Ok(Some(msg.to_owned()))
}

async fn light_on(args: ArgMatches, context: &mut Ctx) -> Result<Option<String>> {
    let id = args.get_one::<String>("id").unwrap();

    context.sifis.lamp(&id).await?.turn_on().await?;

    Ok(None)
}

async fn light_off(args: ArgMatches, context: &mut Ctx) -> Result<Option<String>> {
    let id = args.get_one::<String>("id").unwrap();

    context.sifis.lamp(&id).await?.turn_off().await?;

    Ok(None)
}

async fn brightness(args: ArgMatches, context: &mut Ctx) -> Result<Option<String>> {
    let id = args.get_one::<String>("id").unwrap();
    let brightness = args.get_one::<u8>("brightness").unwrap();

    context
        .sifis
        .lamp(&id)
        .await?
        .set_brightness(*brightness)
        .await?;

    Ok(None)
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut repl = Repl::new(Ctx {
        sifis: Sifis::new().await?,
    })
    .with_name("Sifis Third Party API REPL")
    .with_version("v0.1.0")
    .with_command_async(
        Command::new("list_lamps").about("List the available lamps"),
        |args, context| Box::pin(list_lamps(args, context)),
    )
    .with_command_async(
        Command::new("light_on")
            .arg(Arg::new("id").required(true))
            .about("Turn the lamp on."),
        |args, context| Box::pin(light_on(args, context)),
    )
    .with_command_async(
        Command::new("light_off")
            .arg(Arg::new("id").required(true))
            .about("Turn the lamp off."),
        |args, context| Box::pin(light_off(args, context)),
    )
    .with_command_async(
        Command::new("brightness")
            .arg(Arg::new("id").required(true))
            .arg(
                Arg::new("brightness")
                    .value_parser(value_parser!(u8).range(0..=100))
                    .required(true),
            )
            .about("Set the lamp brightness"),
        |args, context| Box::pin(brightness(args, context)),
    )
    .with_stop_on_ctrl_c(true)
    .with_on_after_command_async(|context| Box::pin(update_prompt(context)));
    repl.run_async().await?;

    Ok(())
}
