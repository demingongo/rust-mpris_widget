use std::{ env, process };

use mpris_widget::Config;

// There’s a tendency among many Rustaceans to avoid using clone to fix ownership problems because of its runtime cost. 
// In Chapter 13, you’ll learn how to use more efficient methods in this type of situation.

//const LIST_PLAYERS_CMD: &str = "~/Documents/Github/awesomewm-mpris-widget/bin/list_players_metadata";
//
//#[derive(Debug)]
//enum State {
//    Playing,
//    Paused,
//    Stopped
//}
//
//struct Config {
//    action: String,
//    player: String,
//}
//
//impl Config {
//    fn build(args: &[String]) -> Result<Config, &'static str> {
//        // play_pause, previous, next, select
//        let action = match args.get(1) {
//            Some(v) => v.clone(),
//            None => String::from("")
//        };
//        // e.g.: spotify, musikcube, ...
//        let player = match args.get(2) {
//            Some(v) => v.clone(),
//            None => String::from("")
//        };
//
//        Ok(Config { action, player })
//    }
//}
//
//#[derive(Debug)]
//struct PlayerMetadata {
//    state: State,
//    artist: String,
//    title: String,
//    //art_url: String,
//    //album: String,
//    player: String,
//
//    separator: String,
//    state_paused: String,
//    state_playing: String,
//    state_stopped: String
//}
//
//impl PlayerMetadata {
//    fn create(player: &str, state: &str, artist: &str, title: &str
//        //, album: &str, art_url: &str
//    ) -> Self {
//        let player_state: State;
//        if state == "Playing" {
//            player_state = State::Playing;
//        } else if state == "Paused" {
//            player_state = State::Paused;
//        } else {
//            player_state = State::Stopped;
//        }
//        Self {
//            state:      player_state,
//            artist:     String::from(artist),
//            title:      String::from(title),
//            //art_url:    String::from(art_url),
//            //album:      String::from(album),
//            player:     String::from(player),
//
//            separator:     String::from(" - "),
//            state_paused:  String::from(" "),
//            state_playing: String::from("󰝚 "),
//            state_stopped: String::from(" ")
//        }
//    }
//
//    fn get_display(&self) -> String {
//        let mut result = self.title.clone();
//
//        let state_display = match self.state {
//            State::Paused => self.state_paused.clone(),
//            State::Playing => self.state_playing.clone(),
//            State::Stopped => self.state_stopped.clone()
//        };
//
//        if !self.artist.is_empty() {
//            result = self.artist.clone() + self.separator.as_str() + self.title.clone().as_str();
//        }
//
//        result = state_display + result.as_str();
//
//        result
//    }
//}
//
//fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
//    let (sender, receiver) = bounded(100);
//    ctrlc::set_handler(move || {
//        let _ = sender.send(());
//    })?;
//
//    Ok(receiver)
//}
//
//async fn fetch_data(resp: &mut String) -> Option<i32> {
//    //println!("");
//    //println!("fetching data ...");
//    let output_result = Command::new("sh")
//    .arg("-c")
//    .arg(LIST_PLAYERS_CMD)
//    .output();
//
//    let output = match output_result {
//        Ok(result) => result,
//        Err(error) => match error.kind() {
//            other_error => {
//                panic!("Failed to execute script: {:?}", other_error);
//            }
//        },
//    };
//
//    let output_string = String::from_utf8(output.stdout).unwrap();
//
//    //println!("output is {}", output_string);
//
//    let players_list: Vec<&str> = output_string.split("\n").collect();
//
//    let mut first_display = String::new();
//    let mut first_player: Option<PlayerMetadata> = None;
//
//    for data in players_list {
//        if data.len() < 1 {
//            break;
//        }
//
//        let metadata: Vec<&str> = data.split(";").collect();
//
//        let formatted_data = PlayerMetadata::create(
//            metadata.get(6)?.trim(), 
//            metadata.get(0)?.trim(),
//            metadata.get(1)?.trim(),
//            metadata.get(2)?.trim(),
//            //metadata.get(4)?.trim(),
//            //metadata.get(3)?.trim()
//        );
//
//        first_display = formatted_data.get_display();
//        first_player = Some(formatted_data);
//        break;
//        //println!("{}", formatted_data.get_display());
//        //println!();
//
//        //dbg!(formatted_data);
//    }
//
//    if resp.as_str() != first_display.as_str() {
//        // change resp by value of first_display
//        *resp = first_display.clone();
//        if let Some(value) = first_player {
//            println!("{{\"text\": \"{}\", \"class\": \"custom-{}\", \"alt\": \"{}\"}}", resp, value.player, value.player);  
//        } else {
//            println!("{}", resp);
//        }
//    }
//
//    output.status.code()
//}
//
//async fn run(config: Config) -> Result<(), Box<dyn Error>> {
//    if !config.action.is_empty() {
//        // do action
//    } else {
//        let ctrl_c_events = ctrl_channel()?;
//        //let mut interval = time::interval(Duration::from_secs(5));
//        let ticks = tick(Duration::from_secs(1));
//        let mut current_display = String::from("");
//    
//        loop {
//            select! {
//                recv(ticks) -> _ => {
//                    //interval.tick().await;
//                    let code: Option<i32> = fetch_data(&mut current_display).await;
//                    if code != Some(0) {
//                        break;
//                    }
//                }
//                recv(ctrl_c_events) -> _ => {
//                    println!();
//                    //println!("Goodbye!");
//                    break;
//                }
//            }
//        };
//    }
//
//    Ok(())
//}

#[tokio::main] // to allow 'main' function to be async
async fn main() {
    

    // parse arguments
    //let args: Vec<String> = env::args().collect();
    //let config = Config::build(&args).unwrap_or_else(|err| {
    //    eprintln!("Problem parsing arguments: {err}");
    //    process::exit(1);
    //});
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    // run application
    if let Err(e) = mpris_widget::run(config).await {
        eprintln!("Application error: {e}");
        process::exit(1);
    }

    //let ctrl_c_events = ctrl_channel()?;
    //    //let mut interval = time::interval(Duration::from_secs(5));
    //    let ticks = tick(Duration::from_secs(1));
    //    let mut current_display = String::from("");
    //
    //    loop {
    //        select! {
    //            recv(ticks) -> _ => {
    //                //interval.tick().await;
    //                let code: Option<i32> = fetch_data(&mut current_display).await;
    //                if code != Some(0) {
    //                    break;
    //                }
    //            }
    //            recv(ctrl_c_events) -> _ => {
    //                println!();
    //                println!("Goodbye!");
    //                break;
    //            }
    //        }
    //    };
}
