pub mod search {
    use colored::*;
    use std::{process::Command, thread};

    pub fn play_selection(selection: &str, title: &str, audio_only: bool) {
        // Clear the terminal
        let _ = Command::new("clear").status();

        let selection = selection.to_owned();
        println!("{} {}", "Playing:".green().bold(), title.yellow());
        if audio_only {
            let mut child = Command::new("mpv")
                .arg("--no-video")
                // .arg("--force-window")
                .arg(&selection.trim())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .expect("Failed to execute command. Ensure mpv is installed.");

            let child_id = child.id();

            ctrlc::set_handler(move || {
                let _ = Command::new("kill")
                    .arg("-2")
                    .arg(child_id.to_string())
                    .spawn();
                // then exit the program
                std::process::exit(0);
            })
            .expect("Error setting Ctrl-C handler");

            // Wait for the child process to finish
            let _ = child.wait().expect("Failed to wait on child");
        } else {
            thread::spawn(move || {
                Command::new("mpv")
                    .arg(&selection.trim())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .expect("Failed to execute command. Ensure mpv is installed.")
            });
        }
    }
}
