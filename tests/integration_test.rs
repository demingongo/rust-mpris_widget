
#[cfg(test)]
mod tests {
    use mpris_widget::{send_action, exec_action};


    #[test]
    fn play_pause() {
        let action: String = String::from("play-pause");
        let player = String::new();

        let result = send_action(&action, &player);
        // let result = mpris_widget::exec_action(&action, &player);

        if let Err(error) = result {
            assert!(false, "'send_action' error: {}", error);
        }
    }

    #[test]
    #[should_panic]
    fn fail_command() {
        let action: String = String::from("unknown_command");
        let player = String::new();

        let result = exec_action(&action, &player);

        if let Err(error) = result {
            assert!(false, "'send_action' error: {}", error);
        }
    }

    #[test]
    fn select_player_command() {
        let action: String = String::from("select");
        let player = String::from("elisa");

        let result = send_action(&action, &player);

        if let Err(error) = result {
            assert!(false, "'send_action' error: {}", error);
        }
    }
}