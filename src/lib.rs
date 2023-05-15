use crossbeam_channel::{bounded, select, tick, Receiver};
use std::{env, error::Error, process::Command};
use tokio::time::Duration;

const LIST_PLAYERS_CMD: &str =
    "~/Documents/Github/awesomewm-mpris-widget/bin/list_players_metadata";

#[derive(Debug)]
enum State {
    Playing,
    Paused,
    Stopped,
}

pub struct Config {
    action: String,
    player: String,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        // unnecessary first arg
        args.next();

        // play_pause, previous, next, select
        let action = match args.next() {
            Some(v) => v,
            None => String::from(""), // arguments are optional so do not return Err
        };
        // e.g.: spotify, musikcube, ...
        let player = match args.next() {
            Some(v) => v,
            None => String::from(""), // arguments are optional so do not return Err
        };

        Ok(Config { action, player })
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
            state_playing: String::from("󰝚 "),
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

async fn fetch_data(resp: &mut String) -> Result<Option<i32>, Box<dyn Error>> {
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

        first_display = formatted_data.get_display();
        first_player = Some(formatted_data);
        break;
    }

    if resp.as_str() != first_display.as_str() {
        // change resp by value of first_display
        *resp = first_display;
        if let Some(value) = first_player {
            println!(
                "{{\"text\": \"{}\", \"class\": \"custom-{}\", \"alt\": \"{}\"}}",
                resp, value.player, value.player
            );
        } else {
            println!("{}", resp);
        }
    }

    Ok(output.status.code())
}

pub fn do_action(action_name: &String, player: &String) -> Result<(), Box<dyn Error>> {
    if action_name.eq("select") {
        if player.is_empty() {
            return Err("'select' command needs another argument (name of the player)".into());
        }
        // TODO: write the selected player into a file (somewhere where the loop process can access the info)
    } else {
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
    }

    Ok(())
}

//fn handle_client(mut stream: UnixStream) {
//    let mut buf = [0; 1024];
//    println!("thread handle_client read");
//    let count = stream.read(&mut buf).unwrap();
//    let response = String::from_utf8(buf[..count].to_vec()).unwrap();
//    println!("Response: {response}");
//}

pub async fn run(config: Config) -> Result<(), Box<dyn Error>> {
    if !config.action.is_empty() {
        // do action
        do_action(&config.action, &config.player)?;
    } else {
        let ctrl_c_events = ctrl_channel()?;
        let ticks = tick(Duration::from_secs(1));
        let mut current_display = String::from("");

        //thread::spawn(|| {
        //    println!("thread");
        //    // listen to Unix socket (https://doc.rust-lang.org/std/os/unix/net/struct.UnixListener.html)
        //    let listener = match UnixListener::bind("/tmp/mprisw_sock") {
        //        Ok(sock) => {
        //            if let Ok(Some(err)) = sock.take_error() {
        //                println!("Got listener error: {err:?}");
        //            }
        //            println!("Got sock 1");
        //            Some(sock)
        //        }
        //        err => {
        //            println!("Got listener error: {err:?}");
        //            None
        //        },
        //    };
        //
        //    if let Some(sock) = listener {
        //        println!("Got sock 2");
        //        for stream in sock.incoming() {
        //            match stream {
        //                Ok(stream) => {
        //                    println!("thread received stream");
        //                    thread::spawn(|| handle_client(stream));
        //                }
        //                Err(err) => {
        //                    println!("Got socket error: {err:?}");
        //                    break;
        //                }
        //            }
        //        }
        //    }
        //    println!("thread done");
        //});

        loop {
            select! {
                recv(ticks) -> _ => {
                    let code: Option<i32> = fetch_data(&mut current_display).await?;
                    if code != Some(0) {
                        break;
                    }
                }
                recv(ctrl_c_events) -> _ => {
                    // quit
                    println!();
                    break;
                }
            }
        }
    }

    Ok(())
}
