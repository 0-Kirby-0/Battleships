use std::io::{self, Write};

fn handle_input(&mut self) {
    loop {
        print!("Enter command: ");

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        io::stdout().flush().unwrap();
        let words = input.split_whitespace().collect::<Vec<&str>>();

        match words.first() {
            Some(&"fire") => {
                if words.len() != 3 {
                    println!("Incorrect number of arguments. Useage: fire <column> <row>");
                }
                if let (Ok(column), Ok(row)) =
                    (words[1].parse::<usize>(), words[2].parse::<usize>())
                {
                    self.fire(row - 1, column - 1); //subtracting for 1-indexing
                    break;
                } else {
                    println!("Incorrect argument format. Please provide positive integer coordinates. Useage: fire <column> <row>")
                }
            }

            Some(&"sunk") => {
                if words.len() != 2 {
                    println!("Incorrect number of arguments. Useage: sunk <ship length>");
                }
                if let Ok(shiplength) = words[1].parse::<usize>() {
                    if let Some(pos) = self.ships.iter().position(|&ship| ship == shiplength) {
                        self.ships.remove(pos);
                        self.populate_heatmap();
                        break;
                    } else {
                        println!(
                            "Ship was not found. Currently active ships are: {:?}",
                            self.ships
                        );
                    }
                } else {
                    println!("Incorrect argument format. Please provide positive integer ship length. Useage: sunk <ship length>");
                }
            }

            Some(&"reset") => *self = State::new(DEFAULT_WIDTH, DEFAULT_HEIGHT),

            Some(&_) | None => {
                println!("Please enter valid command. Available commands are: fire, sunk, reset")
            }
        }
    }
}
