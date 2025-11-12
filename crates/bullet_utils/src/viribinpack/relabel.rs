use std::io::{BufWriter, Write};
use std::{fs::File, io::BufReader, path::PathBuf};
use structopt::StructOpt;
use viriformat::chess::board::{DrawType, GameOutcome, WinType};
use viriformat::dataformat::{Game, WDL};

#[derive(StructOpt)]
pub struct RelabelOptions {
    #[structopt(required = true)]
    pub input: PathBuf,
}

impl RelabelOptions {
    pub fn run(&self) -> anyhow::Result<()> {
        println!("Reading from [{:#?}]", self.input);

        let file = File::open(&self.input)?;

        let mut reader = BufReader::new(file);
        let mut games = 0usize;

        let mut wins = 0;
        let mut losses = 0;
        let mut draws = 0;

        let mut relabelled_draw = 0;
        let mut relabelled_win = 0;
        let mut empty_games = 0;

        let mut buffer = Vec::new();
        let mut writer = BufWriter::new(File::create(&"relabelled.viri")?);

        while let Ok(mut game) = Game::deserialise_from(&mut reader, buffer) {
            games += 1;

            if games % 16384 == 0 {
                print!("Saw {games} games\r");
            }

            if game.moves.len() == 0 {
                empty_games += 1;
                buffer = game.moves;
                buffer.clear();
                continue;
            }

            let (_, score) = game.moves.last().unwrap();
            let score = score.get();

            if game.outcome() == WDL::Loss {
                if score.abs() <= 10 {
                    relabelled_draw += 1;
                    game.set_outcome(GameOutcome::Draw(DrawType::Adjudication));
                } else if score.is_positive() {
                    relabelled_win += 1;
                    game.set_outcome(GameOutcome::WhiteWin(WinType::Adjudication))
                }
            }

            game.serialise_into(&mut writer)?;

            match game.outcome() {
                viriformat::dataformat::WDL::Win => wins += 1,
                viriformat::dataformat::WDL::Draw => draws += 1,
                viriformat::dataformat::WDL::Loss => losses += 1,
            }

            buffer = game.moves;
            buffer.clear();
        }

        writer.flush()?;

        println!();
        println!("Summary:");
        println!("Games = {games}");
        println!("Relabelled to draw: {relabelled_draw}");
        println!("Relabelled to win: {relabelled_win}");
        println!("Empty games: {empty_games}");

        Ok(())
    }
}
