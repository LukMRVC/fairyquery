/*!
Command is the representation for a DB command that provides additional information about
tables, and other database objects. It is sort of a meta command in SQLite sense.

*/

enum Command {
    DescribeTable(String),
    Unknown(String),
    ShowTables,
    ShowDatabases,
    Exit,
}

impl std::convert::From<&str> for Command {
    fn from(input: &str) -> Self {
        let input = input.trim();
        match input {
            "\\q" => Command::Exit,
            "\\dt" => Command::ShowTables,
            "\\dd" => Command::ShowDatabases,
            _ => Command::Unknown(input.to_owned()),
        }
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("Not implemented")
    }
}
