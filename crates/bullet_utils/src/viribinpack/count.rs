use std::{fs::File, io::BufReader, path::PathBuf};

use structopt::StructOpt;
use viriformat::dataformat::Game;

#[derive(StructOpt)]
pub struct CountOptions {
    #[structopt(required = true)]
    pub input: PathBuf,
}

impl CountOptions {
    pub fn run(&self) -> anyhow::Result<()> {
        println!("Reading from [{:#?}]", self.input);

        let file = File::open(&self.input)?;
        let bytes = file.metadata()?.len();

        let mut reader = BufReader::new(file);
        let mut games = 0usize;
        let mut positions = 0usize;
        let mut kept = 0;
        let mut filtered = 0;

        let mut wins = 0;
        let mut losses = 0;
        let mut draws = 0;

        let mut buffer = Vec::new();
        let mut boardsbuffer = Vec::new();

        let filter = viriformat::dataformat::Filter::default();

        while let Ok(game) = Game::deserialise_from(&mut reader, buffer) {
            games += 1;

            game.splat_to_bulletformat(|b| {
                    boardsbuffer.push(b);
                    Ok(())
                },
                &filter,
            ).unwrap();

            positions += game.moves.len();

            let actual_positions_after_filtering = boardsbuffer.len();
            let filtered_in_this_game = game.moves.len() - actual_positions_after_filtering;
            kept += actual_positions_after_filtering;
            filtered += filtered_in_this_game;

            if games % 16384 == 0 {
                print!("Counted {games} games\r");
            }

            if game.moves.len() == 0 {
                buffer = game.moves;
                buffer.clear();
                boardsbuffer.clear();
                continue;
            }

            match game.outcome() {
                viriformat::dataformat::WDL::Win => wins += 1,
                viriformat::dataformat::WDL::Draw => draws += 1,
                viriformat::dataformat::WDL::Loss => losses += 1,
            }

            buffer = game.moves;
            buffer.clear();
            boardsbuffer.clear();
        }

        println!();
        println!("Summary:");
        println!("Games = {games}");
        println!("Positions = {positions} (kept={kept}, filtered={filtered})");
        println!("Wins = {wins}, Draws = {draws}, Losses = {losses}");
        println!("Bytes per position = {}", bytes as f64 / positions as f64);

        Ok(())
    }
}
