mod api;
mod credentials;
mod input_file;
mod routes;

use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Arg, ArgAction, ArgMatches, Command, builder::NonEmptyStringValueParser};

use self::{
    api::Api,
    routes::{Auth, GROUPS, Input, Route},
};

fn command_line() -> Command {
    let mut cli = Command::new("p2me")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Work with the Prove2me API from the command line")
        .after_help(
            "Examples:\n  p2me theorems search poincare\n  p2me verify status <submission-id>",
        )
        .subcommand_required(true)
        .arg_required_else_help(true);
    for group in GROUPS {
        let mut command = Command::new(group.name)
            .about(group.about)
            .subcommand_required(true)
            .arg_required_else_help(true);
        for route in group.routes {
            command = command.subcommand(operation(route));
        }
        cli = cli.subcommand(command);
    }
    cli
}

fn operation(route: &Route) -> Command {
    let mut command = Command::new(route.name).about(route.about);
    for id in route.path.split('/').filter_map(|s| s.strip_prefix(':')) {
        command = command.arg(positional(id));
    }
    for name in route.query.iter().copied() {
        let mut arg = Arg::new(name).long(name).value_name(name);
        if matches!(route.input, Input::Search) && name == "q" {
            arg = arg
                .action(ArgAction::Append)
                .help("Keyword search; repeat for a batch");
        }
        command = command.arg(arg);
    }
    command = match route.input {
        Input::None => command,
        Input::Json => command.arg(json_file(
            "json-file",
            "Read the JSON request object (- for stdin)",
        )),
        Input::Search => command.arg(
            positional("QUERY")
                .required(false)
                .help("First keyword search; omit to browse"),
        ),
        Input::Submit => command
            .arg(positional("THEOREM_ID"))
            .arg(file_arg())
            .arg(
                Arg::new("proof-type")
                    .long("proof-type")
                    .value_parser(["prove", "disprove"])
                    .default_value("prove"),
            )
            .arg(
                Arg::new("explanation")
                    .long("explanation")
                    .help("Written explanation to submit with the file"),
            ),
        Input::Status => command.arg(positional("SUBMISSION_ID")),
        Input::Avatar => command.arg(file_arg()),
    };
    if matches!(route.auth, Auth::Session) {
        command = command.arg(json_file(
            "session-file",
            "Read a login or refresh-session JSON result (- for stdin)",
        ));
    }
    command
}

fn positional(name: &'static str) -> Arg {
    Arg::new(name)
        .required(true)
        .value_parser(NonEmptyStringValueParser::new())
}

fn file_arg() -> Arg {
    Arg::new("FILE")
        .required(true)
        .value_parser(clap::value_parser!(PathBuf))
}

fn json_file(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name)
        .long(name)
        .value_name("FILE")
        .required(true)
        .value_parser(clap::value_parser!(PathBuf))
        .help(help)
}

pub fn run(base_url: &str) -> ExitCode {
    let matches = command_line().get_matches();
    let (group, matches) = matches.subcommand().expect("command group is required");
    let (operation, args) = matches.subcommand().expect("operation is required");
    let route = GROUPS
        .iter()
        .find(|g| g.name == group)
        .and_then(|g| g.routes.iter().find(|route| route.name == operation))
        .expect("clap accepted a registered route");
    match execute(base_url, route, args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn execute(base_url: &str, route: &Route, args: &ArgMatches) -> Result<(), String> {
    let api = Api::new(base_url)?;
    let segments = route
        .path
        .split('/')
        .map(|segment| path_segment(segment, args))
        .collect::<Result<Vec<_>, _>>()?;
    let mut request = api.request(route.method.clone(), &segments);
    let mut query = Vec::new();
    match route.input {
        Input::Search => {
            if let Some(value) = args.get_one::<String>("QUERY") {
                query.push(("q".to_owned(), value.as_str()));
            }
        }
        Input::Status => query.push(("submission_id".to_owned(), text_arg(args, "SUBMISSION_ID"))),
        _ => {}
    }
    for name in route.query.iter().copied() {
        if let Some(values) = args.get_many::<String>(name) {
            query.extend(values.map(|value| (name.replace('-', "_"), value.as_str())));
        }
    }
    request = request.query(&query);
    request = match route.input {
        Input::Json => request.json(&input_file::read_json(file_path(args, "json-file"))?),
        Input::Submit => request.multipart(api::proof_form(
            text_arg(args, "THEOREM_ID"),
            file_path(args, "FILE"),
            text_arg(args, "proof-type"),
            args.get_one::<String>("explanation").map(String::as_str),
        )?),
        Input::Avatar => request.multipart(api::avatar_form(file_path(args, "FILE"))?),
        Input::None | Input::Search | Input::Status => request,
    };
    let result = match route.auth {
        Auth::Public => api::send_json(request, "request")?,
        Auth::ApiKey => api::exchange_key(request, &credentials::read_key()?)?,
        Auth::Bearer => api.send_authenticated(request, &credentials::read_key()?)?,
        Auth::Session => {
            let session = input_file::read_json(file_path(args, "session-file"))?;
            let token = api::access_token(&session)
                .ok_or("session JSON does not contain an access token")?;
            api::send_json(request.bearer_auth(token), "request")?
        }
    };
    if let Some(result) = result {
        let mut stdout = io::stdout().lock();
        serde_json::to_writer_pretty(&mut stdout, &result)
            .map_err(|_| "could not write the JSON result")?;
        writeln!(stdout).map_err(|_| "could not write the JSON result")?;
        stdout
            .flush()
            .map_err(|_| "could not write the JSON result")?;
    }
    Ok(())
}

fn path_segment<'a>(segment: &'a str, args: &'a ArgMatches) -> Result<&'a str, &'static str> {
    let Some(id) = segment.strip_prefix(':') else {
        return Ok(segment);
    };
    let value = text_arg(args, id);
    if value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
    {
        Ok(value)
    } else if id == "THEOREM_ID" {
        Err("invalid theorem ID")
    } else {
        Err("invalid path parameter: use only ASCII letters, digits, hyphens, and underscores")
    }
}

fn text_arg<'a>(args: &'a ArgMatches, id: &str) -> &'a str {
    args.get_one::<String>(id)
        .expect("text argument is required")
}

fn file_path<'a>(args: &'a ArgMatches, id: &str) -> &'a Path {
    args.get_one::<PathBuf>(id)
        .expect("file argument is required")
}
