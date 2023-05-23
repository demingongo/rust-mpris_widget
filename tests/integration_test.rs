
#[cfg(test)]
mod tests {
    use mpris_widget::{send_action, exec_action, read_first_line};


    #[test]
    fn play_pause() {
        let action: String = String::from("play-pause");
        let player = String::new();

        let result = tokio_test::block_on(send_action(&action, &player, false, false));
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

        let result = exec_action(&action, &player, false);

        if let Err(error) = result {
            assert!(false, "'send_action' error: {}", error);
        }
    }

    #[test]
    fn select_player_command() {
        let action: String = String::from("select");
        let player = String::from("mpv");

        let result = tokio_test::block_on(send_action(&action, &player, false, false));

        if let Err(error) = result {
            assert!(false, "'send_action' error: {}", error);
        }
    }

    #[test]
    fn list_command() {
        let action: String = String::from("list");
        let player = String::new();

        let result = tokio_test::block_on(send_action(&action, &player, false, false));

        if let Err(error) = result {
            assert!(false, "'send_action' error: {}", error);
        }
    }

    #[test]
    fn read_first_line_of_file() {
        let file_path = String::from("./tests/output.txt");

        let expected = "player_name";

        let result = read_first_line(&file_path);

        if let Err(error) = result {
            assert!(false, "'read_first_line' error: {}", error);
        } else if let Ok(v) = result {
            assert!(v == expected, "first line was '{}', expected '{}'", v, expected);
        }
    }
}