use clap::{crate_authors, crate_version, value_t, App, AppSettings, Arg};
use dialoguer::{console::Term, theme::ColorfulTheme, Input, MultiSelect, Password, Select};
use dotenv::dotenv;
use std::env;
use thoth::api::account::model::{AccountData, LinkedPublisher};
use thoth::api::account::service::{all_emails, all_publishers, register, update_password};
use thoth::api::db::{establish_connection, run_migrations};
use thoth::api_server;
use thoth::app_server;
use thoth::export_server;
use thoth_errors::ThothResult;

fn host_argument(env_value: &'static str) -> Arg<'static> {
    Arg::with_name("host")
        .short('H')
        .long("host")
        .value_name("HOST")
        .env(env_value)
        .default_value("0.0.0.0")
        .help("host to bind")
        .takes_value(true)
}

fn port_argument(default_value: &'static str, env_value: &'static str) -> Arg<'static> {
    Arg::with_name("port")
        .short('p')
        .long("port")
        .value_name("PORT")
        .env(env_value)
        .default_value(default_value)
        .help("Port to bind")
        .takes_value(true)
}

fn domain_argument() -> Arg<'static> {
    Arg::with_name("domain")
        .short('d')
        .long("domain")
        .value_name("THOTH_DOMAIN")
        .env("THOTH_DOMAIN")
        .default_value("localhost")
        .help("Authentication cookie domain")
        .takes_value(true)
}

fn key_argument() -> Arg<'static> {
    Arg::with_name("key")
        .short('k')
        .long("secret-key")
        .value_name("SECRET")
        .env("SECRET_KEY")
        .help("Authentication cookie secret key")
        .takes_value(true)
}

fn session_argument() -> Arg<'static> {
    Arg::with_name("duration")
        .short('s')
        .long("session-length")
        .value_name("DURATION")
        .env("SESSION_DURATION_SECONDS")
        .default_value("3600")
        .help("Authentication cookie session duration (seconds)")
        .takes_value(true)
}

fn gql_url_argument() -> Arg<'static> {
    Arg::with_name("gql-url")
        .short('u')
        .long("gql-url")
        .value_name("THOTH_GRAPHQL_API")
        .env("THOTH_GRAPHQL_API")
        .default_value("http://localhost:8000")
        .help("Thoth GraphQL's, public facing, root URL.")
        .takes_value(true)
}

fn gql_endpoint_argument() -> Arg<'static> {
    Arg::with_name("gql-endpoint")
        .short('g')
        .long("gql-endpoint")
        .value_name("THOTH_GRAPHQL_ENDPOINT")
        .env("THOTH_GRAPHQL_ENDPOINT")
        .default_value("http://localhost:8000/graphql")
        .help("Thoth GraphQL's endpoint")
        .takes_value(true)
}

fn export_url_argument() -> Arg<'static> {
    Arg::with_name("export-url")
        .short('u')
        .long("export-url")
        .value_name("THOTH_EXPORT_API")
        .env("THOTH_EXPORT_API")
        .default_value("http://localhost:8181")
        .help("Thoth Export API's, public facing, root URL.")
        .takes_value(true)
}

fn threads_argument(env_value: &'static str) -> Arg<'static> {
    Arg::with_name("threads")
        .short('t')
        .long("threads")
        .value_name("THREADS")
        .env(env_value)
        .default_value("5")
        .help("Number of HTTP workers to start")
        .takes_value(true)
}

fn keep_alive_argument(env_value: &'static str) -> Arg<'static> {
    Arg::with_name("keep-alive")
        .short('K')
        .long("keep-alive")
        .value_name("THREADS")
        .env(env_value)
        .default_value("5")
        .help("Number of seconds to wait for subsequent requests")
        .takes_value(true)
}

fn thoth_commands() -> App<'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(App::new("migrate").about("Run the database migrations"))
        .subcommand(
            App::new("start")
                .about("Start an instance of Thoth API or GUI")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    App::new("graphql-api")
                        .about("Start the thoth GraphQL API server")
                        .arg(host_argument("GRAPHQL_API_HOST"))
                        .arg(port_argument("8000", "GRAPHQL_API_PORT"))
                        .arg(threads_argument("GRAPHQL_API_THREADS"))
                        .arg(keep_alive_argument("GRAPHQL_API_KEEP_ALIVE"))
                        .arg(gql_url_argument())
                        .arg(domain_argument())
                        .arg(key_argument())
                        .arg(session_argument()),
                )
                .subcommand(
                    App::new("app")
                        .about("Start the thoth client GUI")
                        .arg(host_argument("APP_HOST"))
                        .arg(port_argument("8080", "APP_PORT"))
                        .arg(threads_argument("APP_THREADS"))
                        .arg(keep_alive_argument("APP_KEEP_ALIVE")),
                )
                .subcommand(
                    App::new("export-api")
                        .about("Start the thoth metadata export API")
                        .arg(host_argument("EXPORT_API_HOST"))
                        .arg(port_argument("8181", "EXPORT_API_PORT"))
                        .arg(threads_argument("EXPORT_API_THREADS"))
                        .arg(keep_alive_argument("EXPORT_API_KEEP_ALIVE"))
                        .arg(export_url_argument())
                        .arg(gql_endpoint_argument()),
                ),
        )
        .subcommand(
            App::new("init")
                .about("Run the database migrations and start the thoth API server")
                .arg(host_argument("GRAPHQL_API_HOST"))
                .arg(port_argument("8000", "GRAPHQL_API_PORT"))
                .arg(threads_argument("GRAPHQL_API_THREADS"))
                .arg(keep_alive_argument("GRAPHQL_API_KEEP_ALIVE"))
                .arg(gql_url_argument())
                .arg(domain_argument())
                .arg(key_argument())
                .arg(session_argument()),
        )
        .subcommand(
            App::new("account")
                .about("Manage user accounts")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(App::new("register").about("Create a new user account"))
                .subcommand(App::new("password").about("Reset a password")),
        )
}

fn main() -> ThothResult<()> {
    // load environment variables from `.env`
    dotenv().ok();

    match thoth_commands().get_matches().subcommand() {
        Some(("start", start_matches)) => match start_matches.subcommand() {
            Some(("graphql-api", api_matches)) => {
                let host = api_matches.value_of("host").unwrap().to_owned();
                let port = api_matches.value_of("port").unwrap().to_owned();
                let threads = value_t!(api_matches.value_of("threads"), usize).unwrap();
                let keep_alive = value_t!(api_matches.value_of("keep-alive"), u64).unwrap();
                let url = api_matches.value_of("gql-url").unwrap().to_owned();
                let domain = api_matches.value_of("domain").unwrap().to_owned();
                let secret_str = api_matches.value_of("key").unwrap().to_owned();
                let session_duration = value_t!(api_matches.value_of("duration"), i64).unwrap();
                api_server(
                    host,
                    port,
                    threads,
                    keep_alive,
                    url,
                    domain,
                    secret_str,
                    session_duration,
                )
                .map_err(|e| e.into())
            }
            Some(("app", client_matches)) => {
                let host = client_matches.value_of("host").unwrap().to_owned();
                let port = client_matches.value_of("port").unwrap().to_owned();
                let threads = value_t!(client_matches.value_of("threads"), usize).unwrap();
                let keep_alive = value_t!(client_matches.value_of("keep-alive"), u64).unwrap();
                app_server(host, port, threads, keep_alive).map_err(|e| e.into())
            }
            Some(("export-api", client_matches)) => {
                let host = client_matches.value_of("host").unwrap().to_owned();
                let port = client_matches.value_of("port").unwrap().to_owned();
                let threads = value_t!(client_matches.value_of("threads"), usize).unwrap();
                let keep_alive = value_t!(client_matches.value_of("keep-alive"), u64).unwrap();
                let url = client_matches.value_of("export-url").unwrap().to_owned();
                let gql_endpoint = client_matches.value_of("gql-endpoint").unwrap().to_owned();
                export_server(host, port, threads, keep_alive, url, gql_endpoint)
                    .map_err(|e| e.into())
            }
            _ => unreachable!(),
        },
        Some(("migrate", _)) => run_migrations(),
        Some(("init", init_matches)) => {
            let host = init_matches.value_of("host").unwrap().to_owned();
            let port = init_matches.value_of("port").unwrap().to_owned();
            let threads = value_t!(init_matches.value_of("threads"), usize).unwrap();
            let keep_alive = value_t!(init_matches.value_of("keep-alive"), u64).unwrap();
            let url = init_matches.value_of("gql-url").unwrap().to_owned();
            let domain = init_matches.value_of("domain").unwrap().to_owned();
            let secret_str = init_matches.value_of("key").unwrap().to_owned();
            let session_duration = value_t!(init_matches.value_of("duration"), i64).unwrap();
            run_migrations()?;
            api_server(
                host,
                port,
                threads,
                keep_alive,
                url,
                domain,
                secret_str,
                session_duration,
            )
            .map_err(|e| e.into())
        }
        Some(("account", account_matches)) => match account_matches.subcommand() {
            Some(("register", _)) => {
                let pool = establish_connection();

                let name: String = Input::new()
                    .with_prompt("Enter given name")
                    .interact_on(&Term::stdout())?;
                let surname: String = Input::new()
                    .with_prompt("Enter family name")
                    .interact_on(&Term::stdout())?;
                let email: String = Input::new()
                    .with_prompt("Enter email address")
                    .interact_on(&Term::stdout())?;
                let password = Password::new()
                    .with_prompt("Enter password")
                    .with_confirmation("Confirm password", "Passwords do not match")
                    .interact_on(&Term::stdout())?;
                let is_superuser: bool = Input::new()
                    .with_prompt("Is this a superuser account")
                    .default(false)
                    .interact_on(&Term::stdout())?;
                let is_bot: bool = Input::new()
                    .with_prompt("Is this a bot account")
                    .default(false)
                    .interact_on(&Term::stdout())?;

                let mut linked_publishers = vec![];
                if let Ok(publishers) = all_publishers(&pool) {
                    let chosen: Vec<usize> = MultiSelect::new()
                        .items(&publishers)
                        .with_prompt("Select publishers to link this account to")
                        .interact_on(&Term::stdout())?;
                    for index in chosen {
                        let publisher = publishers.get(index).unwrap();
                        let is_admin: bool = Input::new()
                            .with_prompt(format!(
                                "Make user an admin of '{}'?",
                                publisher.publisher_name
                            ))
                            .default(false)
                            .interact_on(&Term::stdout())?;
                        let linked_publisher = LinkedPublisher {
                            publisher_id: publisher.publisher_id,
                            is_admin,
                        };
                        linked_publishers.push(linked_publisher);
                    }
                }
                let account_data = AccountData {
                    name,
                    surname,
                    email,
                    password,
                    is_superuser,
                    is_bot,
                };
                register(account_data, linked_publishers, &pool).map(|_| ())
            }
            Some(("password", _)) => {
                let pool = establish_connection();
                let all_emails = all_emails(&pool).expect("No user accounts present in database.");
                let email_selection = Select::with_theme(&ColorfulTheme::default())
                    .items(&all_emails)
                    .default(0)
                    .with_prompt("Select a user account")
                    .interact_on(&Term::stdout())?;
                let password = Password::new()
                    .with_prompt("Enter new password")
                    .with_confirmation("Confirm password", "Passwords do not match")
                    .interact_on(&Term::stdout())?;
                let email = all_emails.get(email_selection).unwrap();

                update_password(email, &password, &pool).map(|_| ())
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

#[test]
fn verify_app() {
    thoth_commands().debug_assert();
}
