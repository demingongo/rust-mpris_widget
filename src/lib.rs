use crossbeam_channel::{bounded, select, tick, Receiver};
use std::{env, error::Error, process::Command, os::unix::net::{UnixStream, UnixListener}, thread::{self, JoinHandle}, io::{Write, Read}, fs};
use tokio::time::Duration;

const LIST_PLAYERS_CMD: &str =
    "~/Documents/Github/awesomewm-mpris-widget/bin/list_players_metadata";

const SOCK_PATH: &str = "/tmp/mpris_widget.sock";

#[derive(Debug)]
enum State {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug)]
pub struct StreamMessage {
    action: String,
    player: String,
}

impl StreamMessage {
    pub fn build(message: String) -> Result<StreamMessage, &'static str> {

        let split_message = message.split(" ");

        let it: Vec<_> = split_message.collect();

        let action = match it.get(0) {
            Some(v) => String::from(*v),
            None => String::new()
        };
        let player = match it.get(1) {
            Some(v) => String::from(*v),
            None => String::new()
        };

        Ok(StreamMessage { action, player })
    }

    fn is_empty(&self) -> bool {
        self.action.is_empty()
    }
}

pub struct Config {
    action: String,
    player: String,
    no_server: bool,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        // unnecessary first arg
        args.next();

        let mut extracted_args: Vec<String> = vec![];
        let mut options: Vec<String> = vec![];

        for arg in args {
            if arg.starts_with("--") {
                // it's an optional argument
                options.push(arg);
            } else {
                // it's an argument
                extracted_args.push(arg);
            }
        }

        let mut extracted_args_iter = extracted_args.into_iter();
        let options_iter = options.into_iter();

        // play_pause, previous, next, select
        let action = match extracted_args_iter.next() {
            Some(v) => v,
            None => String::new(), // arguments are optional so do not return Err
        };
        // e.g.: spotify, musikcube, ...
        let player = match extracted_args_iter.next() {
            Some(v) => v,
            None => String::new(), // arguments are optional so do not return Err
        };

        let mut no_server = false;

        for arg in options_iter {
            if arg.starts_with("--no-server") {
                no_server = true;
            }
        }

        Ok(Config { action, player, no_server })
    }
}

#[derive(Debug)]
pub struct PlayerMetadata {
    state: State,
    artist: String,
    title: String,
    //art_url: String,
    //album: String,
    player: String,

    separator: String,
    state_paused: String,
    state_playing: String,
    state_stopped: String,
}

impl PlayerMetadata {
    fn create(player: &str, state: &str, artist: &str, title: &str) -> Self {
        let player_state: State;
        if state == "Playing" {
            player_state = State::Playing;
        } else if state == "Paused" {
            player_state = State::Paused;
        } else {
            player_state = State::Stopped;
        }
        Self {
            state: player_state,
            artist: String::from(artist),
            title: String::from(title),
            //art_url:    String::from(art_url),
            //album:      String::from(album),
            player: String::from(player),

            separator: String::from(" - "),
            state_paused: String::from(" "),
            state_playing: String::from(""),
            state_stopped: String::from(" "),
        }
    }

    fn get_display(&self) -> String {
        let mut result = String::from("");

        let state_display = match self.state {
            State::Paused => &self.state_paused,
            State::Playing => &self.state_playing,
            State::Stopped => &self.state_stopped,
        };

        if !self.artist.is_empty() {
            result.push_str(state_display);
            result.push_str(&self.artist);
            result.push_str(&self.separator);
            result.push_str(&self.title);
        } else {
            result.push_str(state_display);
            result.push_str(&self.title);
        }

        result
    }
}

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

async fn fetch_list() -> Result<Vec<PlayerMetadata>, Box<dyn Error>> {
    let cmd_path =
        env::var("PLAYERS_METADATA_PATH").unwrap_or_else(|_| String::from(LIST_PLAYERS_CMD));
    let output = Command::new("sh").arg("-c").arg(cmd_path).output()?;

    let output_string = String::from_utf8(output.stdout).unwrap();

    //println!("output is {}", output_string);

    let players_list: Vec<&str> = output_string.split("\n").collect();

    let mut players: Vec<PlayerMetadata> = vec![];

    for data in players_list {
        if data.len() < 1 {
            break;
        }

        let metadata: Vec<&str> = data.split(";").collect();

        let formatted_data = PlayerMetadata::create(
            match metadata.get(6) {
                Some(value) => value.trim(),
                _ => return Err("Could not extract player's name".into()),
            },
            match metadata.get(0) {
                Some(value) => value.trim(),
                _ => return Err("Could not extract player's state".into()),
            },
            match metadata.get(1) {
                Some(value) => value.trim(),
                _ => return Err("Could not extract artist".into()),
            },
            match metadata.get(2) {
                Some(value) => value.trim(),
                _ => return Err("Could not extract title".into()),
            },
        );

        players.push(formatted_data);
    }

    Ok(players)
}

async fn fetch_data(selected_player: &String) -> Result<(Option<i32>, Option<PlayerMetadata>, String), Box<dyn Error>> {
    let cmd_path =
        env::var("PLAYERS_METADATA_PATH").unwrap_or_else(|_| String::from(LIST_PLAYERS_CMD));
    let output = Command::new("sh").arg("-c").arg(cmd_path).output()?;

    let output_string = String::from_utf8(output.stdout).unwrap();

    //println!("output is {}", output_string);

    let players_list: Vec<&str> = output_string.split("\n").collect();

    let mut first_display = String::new();
    let mut first_player: Option<PlayerMetadata> = None;

    for data in players_list {
        if data.len() < 1 {
            break;
        }

        let metadata: Vec<&str> = data.split(";").collect();

        let formatted_data = PlayerMetadata::create(
            match metadata.get(6) {
                Some(value) => value.trim(),
                _ => return Err("Could not extract player's name".into()),
            },
            match metadata.get(0) {
                Some(value) => value.trim(),
                _ => return Err("Could not extract player's state".into()),
            },
            match metadata.get(1) {
                Some(value) => value.trim(),
                _ => return Err("Could not extract artist".into()),
            },
            match metadata.get(2) {
                Some(value) => value.trim(),
                _ => return Err("Could not extract title".into()),
            },
        );

        let is_selected_player = formatted_data.player.eq(selected_player);

        if is_selected_player || first_player.is_none() {
            first_display = formatted_data.get_display();
            first_player = Some(formatted_data);
            if is_selected_player {
                break;
            }
        }
    }

    Ok((output.status.code(), first_player, first_display))
}

fn send_message_to_server(message: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut stream = UnixStream::connect(SOCK_PATH)?;
    stream.write_all(message)?;
    Ok(())
}

/// Executes the action
pub fn exec_action(action_name: &String, player: &String) -> Result<(), Box<dyn Error>> {
    let cmd_path = env::var("PLAYERCTL_PATH").unwrap_or_else(|_| String::from("playerctl"));
    let mut binding = Command::new(cmd_path);
    let mut command = binding.arg(action_name);

    if !player.is_empty() {
        command = command.arg("--player").arg(player);
    }

    let output = command.output()?;

    // error if exit code is not 0
    if Some(0) != output.status.code() {
        // must return Err("")? (try! like) or Err("".into()) (less confusing)
        return Err(String::from_utf8(output.stderr).unwrap().into());
    }
    Ok(())
}

/// Sends a command to the server or executes the action as a fallback.
/// If action_name == "list", it returns a list of metadata.
pub async fn send_action(action_name: &String, player: &String, no_server: bool) -> Result<(), Box<dyn Error>> {
    if action_name.eq("select") {
        if player.is_empty() {
            return Err("'select' command needs another argument (name of the player)".into());
        }
        // send message to select a player on the server
        let message: Vec<u8> = [action_name.as_bytes(), b" ", player.as_bytes()].concat();
        send_message_to_server(message.as_slice())?;
    } else if action_name.eq("list") {
        let data_list = fetch_list().await?;
        let mut output = String::from("[");

        for data in data_list.iter() {

            let text = data.get_display();
            let player_name = &data.player;

            output.push_str("{");
            // text
            output.push_str("\"text\": ");
            output.push_str((String::new() + "\"" + text.as_str() + "\"").as_str());
            output.push_str(",");
            // class
            output.push_str(" \"class\": ");
            output.push_str((String::new() + "\"custom-" + player_name.as_str() + "\"").as_str());
            output.push_str(",");
            // alt
            output.push_str(" \"alt\": ");
            output.push_str((String::new() + "\"" + player_name.as_str() + "\"").as_str());
            output.push_str(",");
            // tooltip
            output.push_str(" \"tooltip\": ");
            output.push_str((String::new() + "\"" + text.as_str() + "\"").as_str());

            output.push_str("},");
        }

        // remove the last comma
        if data_list.len() > 0 {
            output.pop();
        }

        output.push_str("]");

        println!("{}", output);
    } else {
        // try to send message to server
        if no_server {
            exec_action(action_name, player)?;
        } else {
            let message: Vec<u8> = [action_name.as_bytes(), b" ", player.as_bytes()].concat();
            let result = send_message_to_server(message.as_slice());

            // fallback, execute the action
            if let Err(_) = result {
                exec_action(action_name, player)?;
            }
        }
        
    }

    Ok(())
}

fn parse_stream_message(mut stream: UnixStream) -> Option<StreamMessage> {
    let mut buf = [0; 1024];
    let count = std::io::Read::read(&mut stream, &mut buf).unwrap();
    let response = String::from_utf8(buf[..count].to_vec()).unwrap();

    match StreamMessage::build(response) {
        Ok(result) => Some(result),
        _ => None
    }
}

fn get_first_line<R>(mut rdr: R) -> Result<String, Box<dyn Error>>
    where R: std::io::BufRead,
{
    let mut first_line = String::new();

    rdr.read_line(&mut first_line)?;

    // Trim the leading hashes and any whitespace
    Ok(first_line)
}

pub fn read_first_line(file_path: &String) -> Result<String, Box<dyn Error>> {
    let file = fs::File::open(file_path)?;
    let buffer = std::io::BufReader::new(file)
        .take(256); // limit number of bytes to be read before returning EOF

    let first_line = get_first_line(buffer)?;

    Ok(
        if let Some(v) = first_line.split("\n").next() {
            String::from(v) // removed next line
        } else {
            String::new()
        }
    )
}

fn start_server(tx: std::sync::mpsc::Sender<StreamMessage>, no_server: bool) -> JoinHandle<()> {
    thread::spawn(move || {
        if no_server {
            // end thread here
            return;
        }

        // listen to Unix socket (https://doc.rust-lang.org/std/os/unix/net/struct.UnixListener.html)
        let listener = match UnixListener::bind(SOCK_PATH) {
            Ok(sock) => {
                if let Ok(Some(err)) = sock.take_error() {
                    eprintln!("Got listener error: {err:?}");
                }
                Some(sock)
            }
            err => {
                eprintln!("Got listener error: {err:?}");
                None
            },
        };
    
        if let Some(sock) = listener {
            // listen to incoming streams (clients)
            for stream in sock.incoming() {
                //println!("incoming stream");
                match stream {
                    Ok(stream) => {
                        let message = parse_stream_message(stream);
                        
                        if let Some(stream_message) = message {
                            // do something
                            if stream_message.is_empty() {
                                // message is empty so we are done here
                                break;
                            }
                            // send info to main thread
                            let _ = tx.send(stream_message);
                        } else {
                            // we are done here
                            break;
                        }
                    }
                    Err(err) => {
                        eprintln!("Got socket error: {err:?}");
                        break;
                    }
                }
            }
        }
        
        std::fs::remove_file(SOCK_PATH).unwrap();
    })
}

pub async fn run(config: Config) -> Result<(), Box<dyn Error>> {
    if !config.action.is_empty() {
        // do action
        send_action(&config.action, &config.player, config.no_server).await?;
    } else {
        let ctrl_c_events = ctrl_channel()?;
        let ticks = tick(Duration::from_secs(1));
        let mut current_display = String::new();
        let mut current_player: String = String::new();

        let (tx, rx): (std::sync::mpsc::Sender<StreamMessage>, std::sync::mpsc::Receiver<StreamMessage>) = std::sync::mpsc::channel();

        let handle = start_server(tx, config.no_server);

        loop {
            select! {
                recv(ticks) -> _ => {
                    match rx.try_recv() {
                        Ok(message) => {
                            if message.action.eq("select") {
                                // changing player
                                current_player = message.player;
                            } else {
                                let result = exec_action(&message.action, &current_player);
                                if let Err(err) = result {
                                    eprintln!("Error (exec_action): {err:?}");
                                }
                            }
                        }
                        Err(_) => {}
                    }

                    // fetch data
                    let (code, metadata, text) = fetch_data(& current_player).await?;

                    // something happened while trying to fetch data
                    if code != Some(0) {
                        break;
                    }

                    let mut it_should_print = false;

                    // text to display
                    if !current_display.eq(&text) {
                        current_display = text;
                        it_should_print = true;
                    }

                    // player to display/control
                    if let Some(value) = metadata {
                        if !current_player.eq(&value.player) {
                            current_player = value.player;
                            it_should_print = true;
                        }
                    }

                    // print
                    if it_should_print {
                        if current_display.is_empty() {
                            println!("{}", current_display);
                        } else {
                            println!(
                                "{{\"text\": \"{}\", \"class\": \"custom-{}\", \"alt\": \"{}\", \"tooltip\": \"({}) {}\"}}",
                                current_display, current_player, current_player, current_player, current_display
                            );
                        }
                    }
                }
                recv(ctrl_c_events) -> _ => {
                    // quit
                    println!();
                    break;
                }
            }
        }

        if !config.no_server {
            // send message to stop server
            send_message_to_server(b" ")?;
        }

        handle.join().unwrap();
    }

    Ok(())
}
