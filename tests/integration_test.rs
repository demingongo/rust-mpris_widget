
#[cfg(test)]
mod tests {
    use mpris_widget::do_action;

    #[test]
    fn play_pause() {
        let action: String = String::from("play-pause");
        let player = String::new();

        let result = do_action(&action, &player);

        if let Err(error) = result {
            assert!(false, "'do_action' error: {}", error);
        }
    }

    #[test]
    #[should_panic]
    fn fail_command() {
        let action: String = String::from("unknown_command");
        let player = String::new();

        let result = do_action(&action, &player);

        if let Err(error) = result {
            assert!(false, "'do_action' error: {}", error);
        }
    }

    #[test]
    fn select_player_command() {
        let action: String = String::from("select");
        let player = String::from("musikcube");

        let result = do_action(&action, &player);

        if let Err(error) = result {
            assert!(false, "'do_action' error: {}", error);
        }
    }
}