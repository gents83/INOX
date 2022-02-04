use std::{env, str::FromStr};

#[derive(Default)]
struct Command {
    name: String,
    values: Vec<String>,
}

#[derive(Default)]
pub struct CommandParser {
    commands: Vec<Command>,
}

impl CommandParser {
    pub fn from_command_line() -> Self {
        Self::from_strings(env::args().collect())
    }
    pub fn from_string(s: &str) -> Self {
        let mut strings = Vec::new();
        s.lines().for_each(|s| {
            s.split_whitespace()
                .for_each(|s| strings.push(s.to_string()))
        });
        Self::from_strings(strings)
    }

    pub fn has(&self, command_name: &str) -> bool {
        self.commands
            .iter()
            .any(|c| c.name.as_str() == command_name)
    }

    pub fn get_values_of<T>(&self, command_name: &str) -> Vec<T>
    where
        T: FromStr + Default,
    {
        let mut values = Vec::new();
        self.commands
            .iter()
            .filter(|c| c.name.as_str() == command_name)
            .for_each(|c| {
                c.values.iter().for_each(|s| {
                    values.push(s.parse::<T>().unwrap_or_default());
                });
            });
        values
    }

    fn from_strings(args: Vec<String>) -> Self {
        let mut commands = Vec::new();
        for a in args {
            if a.starts_with('-') {
                let mut name = a.clone();
                name.remove(0);
                let mut values: Vec<String> =
                    name.split_whitespace().map(|s| s.to_string()).collect();
                let name = values.remove(0);
                values.retain(|s| !s.is_empty());
                commands.push(Command { name, values });
            } else if let Some(command) = commands.last_mut() {
                command.values.push(a);
            } else {
                eprintln!("Waiting for a command '-name' instead got {}", a);
            }
        }
        Self { commands }
    }
}
