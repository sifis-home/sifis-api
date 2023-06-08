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
    #[error("Quit requested")]
    Quit,
}

type Result<T> = std::result::Result<T, CliError>;

struct Ctx {
    sifis: Sifis,
}

async fn list_lamps(_args: ArgMatches, context: &mut Ctx) -> Result<Option<String>> {
    let mut out = String::new();

    writeln!(out, "{:<15} {:<7} {:<5}", "Lamp id", "Status", "Brightness").unwrap();
    for lamp in context.sifis.lamps().await? {
        let on_off = if lamp.get_on_off().await? {
            "On"
        } else {
            "Off"
        };
        let brightness = lamp.get_brightness().await?;
        writeln!(out, "{:<15} {:<7} {:<5} ", lamp.id, on_off, brightness).unwrap();
    }

    Ok(Some(out))
}

async fn update_prompt(_context: &mut Ctx) -> Result<Option<String>> {
    let msg = "Ok";
    Ok(Some(msg.to_owned()))
}

async fn light_on(args: ArgMatches, context: &mut Ctx) -> Result<Option<String>> {
    let id = args.get_one::<String>("id").unwrap();

    context.sifis.lamp(id).await?.turn_on().await?;

    Ok(None)
}

async fn light_off(args: ArgMatches, context: &mut Ctx) -> Result<Option<String>> {
    let id = args.get_one::<String>("id").unwrap();

    context.sifis.lamp(id).await?.turn_off().await?;

    Ok(None)
}

async fn brightness(args: ArgMatches, context: &mut Ctx) -> Result<Option<String>> {
    let id = args.get_one::<String>("id").unwrap();
    let brightness = args.get_one::<u8>("brightness").unwrap();

    context
        .sifis
        .lamp(id)
        .await?
        .set_brightness(*brightness)
        .await?;

    Ok(None)
}

async fn list_sinks(_args: ArgMatches, context: &mut Ctx) -> Result<Option<String>> {
    let mut out = String::new();

    writeln!(
        out,
        "{:<15} {:<4} {:<11} {:<11}",
        "Sink id", "Flow", "Water level", "Temperature"
    )
    .unwrap();
    for sink in context.sifis.sinks().await? {
        let flow = sink.get_flow().await?;
        let water_level = sink.get_water_level().await?;
        let temperature = sink.get_temperature().await?;
        writeln!(
            out,
            "{:<15} {flow:<4} {water_level:<11} {temperature:<11}",
            sink.id
        )
        .unwrap();
    }

    Ok(Some(out))
}

async fn set_sink_flow(args: ArgMatches, context: &mut Ctx) -> Result<Option<String>> {
    let id = args.get_one::<String>("id").unwrap();
    let flow = args.get_one::<u8>("flow").unwrap();

    context.sifis.sink(id).await?.set_flow(*flow).await?;

    Ok(None)
}

async fn open_sink_drain(args: ArgMatches, context: &mut Ctx) -> Result<Option<String>> {
    let id = args.get_one::<String>("id").unwrap();

    context.sifis.sink(id).await?.open_drain().await?;

    Ok(None)
}

async fn close_sink_drain(args: ArgMatches, context: &mut Ctx) -> Result<Option<String>> {
    let id = args.get_one::<String>("id").unwrap();

    context.sifis.sink(id).await?.close_drain().await?;

    Ok(None)
}

async fn set_sink_temperature(args: ArgMatches, context: &mut Ctx) -> Result<Option<String>> {
    let id = args.get_one::<String>("id").unwrap();
    let flow = args.get_one::<u8>("temperature").unwrap();

    context.sifis.sink(id).await?.set_temperature(*flow).await?;

    Ok(None)
}

async fn list_doors(_args: ArgMatches, context: &mut Ctx) -> Result<Option<String>> {
    let mut out = String::new();

    writeln!(
        out,
        "{:<15} {:<5} {:<11}",
        "Door id", "Open?", "Lock status"
    )
    .unwrap();
    for door in context.sifis.doors().await? {
        let is_open = door.is_open().await?;
        let lock_status = door.lock_status().await?;
        writeln!(out, "{:<15} {is_open:<5} {lock_status:<11}", door.id).unwrap();
    }

    Ok(Some(out))
}

async fn lock_door(args: ArgMatches, context: &mut Ctx) -> Result<Option<String>> {
    let id = args.get_one::<String>("id").unwrap();

    context.sifis.door(id).await?.lock().await?;

    Ok(None)
}

async fn unlock_door(args: ArgMatches, context: &mut Ctx) -> Result<Option<String>> {
    let id = args.get_one::<String>("id").unwrap();

    context.sifis.door(id).await?.unlock().await?;

    Ok(None)
}

async fn list_fridges(_args: ArgMatches, context: &mut Ctx) -> Result<Option<String>> {
    let mut out = String::new();

    writeln!(
        out,
        "{:<15} {:<5} {:<11} {:<12}",
        "Fridge id", "Open?", "Temperature", "Target Temp."
    )
    .unwrap();
    for fridge in context.sifis.fridges().await? {
        let is_open = fridge.is_open().await?;
        let temperature = fridge.temperature().await?;
        let target_temperature = fridge.target_temperature().await?;
        writeln!(
            out,
            "{:<15} {is_open:<5} {temperature:<11} {target_temperature:<12}",
            fridge.id
        )
        .unwrap();
    }

    Ok(Some(out))
}

async fn set_fridge_target_temperature(
    args: ArgMatches,
    context: &mut Ctx,
) -> Result<Option<String>> {
    let id = args.get_one::<String>("id").unwrap();
    let temperature = args.get_one::<i8>("temperature").unwrap();

    context
        .sifis
        .fridge(id)
        .await?
        .set_target_temperature(*temperature)
        .await?;

    Ok(None)
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut repl = Repl::new(Ctx {
        sifis: Sifis::new().await?,
    })
    .with_name("Sifis developer API REPL")
    .with_version("v0.1.0")
    .with_command_async(
        Command::new("list_lamps").about("List the available lamps"),
        |args, context| Box::pin(list_lamps(args, context)),
    )
    .with_command_async(
        Command::new("turn_light_on")
            .arg(Arg::new("id").required(true))
            .about("Turn the lamp on."),
        |args, context| Box::pin(light_on(args, context)),
    )
    .with_command_async(
        Command::new("turn_light_off")
            .arg(Arg::new("id").required(true))
            .about("Turn the lamp off."),
        |args, context| Box::pin(light_off(args, context)),
    )
    .with_command_async(
        Command::new("set_lamp_brightness")
            .arg(Arg::new("id").required(true))
            .arg(
                Arg::new("brightness")
                    .value_parser(value_parser!(u8).range(0..=100))
                    .required(true),
            )
            .about("Set the lamp brightness"),
        |args, context| Box::pin(brightness(args, context)),
    )
    .with_command_async(
        Command::new("list_sinks").about("List the available sinks"),
        |args, context| Box::pin(list_sinks(args, context)),
    )
    .with_command_async(
        Command::new("set_sink_flow")
            .arg(Arg::new("id").required(true))
            .arg(
                Arg::new("flow")
                    .value_parser(value_parser!(u8).range(0..=100))
                    .required(true),
            )
            .about("Set the flow of the sink."),
        |args, context| Box::pin(set_sink_flow(args, context)),
    )
    .with_command_async(
        Command::new("close_sink_drain")
            .arg(Arg::new("id").required(true))
            .about("Close the drain of the sink."),
        |args, context| Box::pin(close_sink_drain(args, context)),
    )
    .with_command_async(
        Command::new("open_sink_drain")
            .arg(Arg::new("id").required(true))
            .about("Open the drain of the sink."),
        |args, context| Box::pin(open_sink_drain(args, context)),
    )
    .with_command_async(
        Command::new("set_sink_temperature")
            .arg(Arg::new("id").required(true))
            .arg(
                Arg::new("temperature")
                    .value_parser(value_parser!(u8).range(10..=80))
                    .required(true),
            )
            .about("Set the sink temperature"),
        |args, context| Box::pin(set_sink_temperature(args, context)),
    )
    .with_command_async(
        Command::new("list_doors").about("List the available doors"),
        |args, context| Box::pin(list_doors(args, context)),
    )
    .with_command_async(
        Command::new("lock_door")
            .arg(Arg::new("id").required(true))
            .about("Lock the door"),
        |args, context| Box::pin(lock_door(args, context)),
    )
    .with_command_async(
        Command::new("unlock_door")
            .arg(Arg::new("id").required(true))
            .about("Unlock the door"),
        |args, context| Box::pin(unlock_door(args, context)),
    )
    .with_command_async(
        Command::new("list_fridges").about("List the available fridges"),
        |args, context| Box::pin(list_fridges(args, context)),
    )
    .with_command_async(
        Command::new("set_fridge_target_temperature")
            .arg(Arg::new("id").required(true))
            .arg(
                Arg::new("temperature")
                    .value_parser(value_parser!(i8).range(-20..=20))
                    .required(true),
            )
            .about("Set the fridge target temperature"),
        |args, context| Box::pin(set_fridge_target_temperature(args, context)),
    )
    .with_command(
        Command::new("quit").about("Quit the repl"),
        |_, _context| Err(CliError::Quit),
    )
    .with_stop_on_ctrl_c(true)
    .with_on_after_command_async(|context| Box::pin(update_prompt(context)))
    .with_error_handler(|e, _context| {
        if matches!(e, CliError::Quit) {
            Err(reedline_repl_rs::Error::UnknownCommand("quit".to_string()))
        } else {
            eprintln!("{}", e);
            Ok(())
        }
    });

    use reedline_repl_rs::Error;
    match repl.run_async().await {
        Ok(_) => Ok(()),
        Err(Error::UnknownCommand(e)) => {
            if e == "quit" {
                Ok(())
            } else {
                Err(Error::UnknownCommand(e))
            }
        }
        Err(e) => Err(e),
    }?;

    Ok(())
}
