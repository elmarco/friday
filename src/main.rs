use std::fmt;
use std::process;

extern crate rustyline;
use rustyline::Editor;

#[macro_use]
extern crate clap;
use clap::{App, AppSettings, Arg, SubCommand};

use friday::friday::{Event, Friday, State, Using};

struct FmtVec<'a, T: fmt::Display>(&'a Vec<T>, usize);
impl<'a, T: fmt::Display> fmt::Display for FmtVec<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let map = self
            .0
            .iter()
            .enumerate()
            .map(|(n, c)| format!("#{} {}", self.1 + n, c))
            .collect::<Vec<_>>();
        write!(f, "{}", map.join(", "))
    }
}

// FIXME: rewrite with closure?
fn next(game: &mut Friday, event: Event) {
    if let Err(hint) = game.next(event) {
        println!("{}", hint);
    }
}

fn main() {
    let app = clap_app!(friday =>
       (version: crate_version!())
       (author: crate_authors!())
       (about: "Friday game written in Rust")
    )
    .arg(
        Arg::with_name("level")
            .short("l")
            .long("level")
            .takes_value(true)
            .value_name("LEVEL")
            .default_value("1"),
    );
    let matches = app.get_matches();
    let level = value_t!(matches, "level", usize).unwrap_or(1);

    let mut rl = Editor::<()>::new();
    let mut game = Friday::new(level);

    let mut cli = App::new("friday")
        .setting(AppSettings::NoBinaryName)
        .subcommand(
            SubCommand::with_name("show").arg(
                Arg::with_name("what")
                    .takes_value(true)
                    .possible_value("cards")
                    .possible_value("fight")
                    .possible_value("discards"),
            ),
        )
        .subcommand(
            SubCommand::with_name("choose")
                .arg(Arg::with_name("card").takes_value(true))
                .arg(Arg::with_name("replace").short("r").long("replace")),
        )
        .subcommand(SubCommand::with_name("use").arg(Arg::with_name("card").takes_value(true)))
        .subcommand(SubCommand::with_name("continue"))
        .subcommand(SubCommand::with_name("break"))
        .subcommand(SubCommand::with_name("fight"))
        .subcommand(SubCommand::with_name("win"))
        .subcommand(
            SubCommand::with_name("sort")
                .arg(Arg::with_name("discard").short("d").long("discard"))
                .arg(Arg::with_name("order").takes_value(true).multiple(true)),
        )
        .subcommand(
            SubCommand::with_name("lose")
                .arg(Arg::with_name("discard").takes_value(true).multiple(true)),
        );

    loop {
        {
            let mut left_len = 0;
            if let Some(left) = game.get_left() {
                println!("left: {}", FmtVec(left, 0).to_string());
                left_len = left.len();
            }
            if let Some(right) = game.get_right() {
                println!("right: {}", FmtVec(right, left_len).to_string());
            }
        }
        println!(
            "Life: {}/{}, Step: {:?}",
            game.life_points,
            game.max_life_points(),
            game.step
        );
        println!(
            "# hazard:{} fighting:{} aging:{}",
            game.hazard_deck.len(),
            game.fighting_deck.len(),
            game.aging_deck.len()
        );

        match game.state {
            State::ChooseHazard(ref h) => {
                let descs: Vec<_> = h
                    .iter()
                    .map(|c| {
                        format!(
                            "{} ({})",
                            c.description.kind.to_hazard_card(),
                            c.description.kind.to_fighting_card()
                        )
                    })
                    .collect();
                println!("Choose hazard: {}", FmtVec(&descs, 0));
            }
            State::Fighting(ref c, f, _, _, ref using) => {
                println!("Fight diff: {}", game.fight_diff().unwrap());
                println!(
                    "Free draws left: {}",
                    game.free_cards(&c.description.clone().kind.to_hazard_card()) - f
                );
                match using {
                    Using::None => (),
                    _ => println!("Using {:?}...", using),
                }
            }
            State::ChoosePirate => {
                println!("Choose pirate: {:#?}", game.pirate_cards);
            }
            State::Ended(won) => {
                if won {
                    println!("Game ended! 'Yo ho ho, you won! Score: {}", game.score());
                } else {
                    println!("Game ended! Score: {}", game.score());
                }
                process::exit(0);
            }
            ref state => {
                println!("Unknown state: {:?}", state);
            }
        }
        println!();
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if let Ok(m) = cli.get_matches_from_safe_borrow(line.split(' ')) {
                    if let Some(matches) = m.subcommand_matches("choose") {
                        if let Ok(card) = value_t!(matches.value_of("card"), usize) {
                            match game.state {
                                State::ChoosePirate => {
                                    next(&mut game, Event::Choice(card));
                                }
                                State::ChooseHazard(_) => {
                                    next(&mut game, Event::HazardChoice(Some(card)));
                                }
                                State::Fighting(_, _, _, _, Using::UnderDeck(_)) => {
                                    let replace = matches.is_present("replace");
                                    next(&mut game, Event::ChoiceUnder(card, replace));
                                }
                                State::Fighting(_, _, _, _, Using::Destroy(_))
                                | State::Fighting(_, _, _, _, Using::Swap(_, _))
                                | State::Fighting(_, _, _, _, Using::Double)
                                | State::Fighting(_, _, _, _, Using::Copy) => {
                                    next(&mut game, Event::Choice(card));
                                }
                                _ => {}
                            }
                        } else {
                            next(&mut game, Event::HazardChoice(None));
                        }
                    } else if let Some(matches) = m.subcommand_matches("use") {
                        if let Ok(card) = value_t!(matches.value_of("card"), usize) {
                            next(&mut game, Event::Use(card));
                        }
                    } else if m.subcommand_matches("continue").is_some() {
                        next(&mut game, Event::Continue);
                    } else if m.subcommand_matches("break").is_some() {
                        next(&mut game, Event::Break);
                    } else if m.subcommand_matches("fight").is_some() {
                        next(&mut game, Event::Fight);
                    } else if m.subcommand_matches("win").is_some() {
                        next(&mut game, Event::Win);
                    } else if let Some(matches) = m.subcommand_matches("sort") {
                        let order =
                            values_t!(matches.values_of("order"), usize).unwrap_or_default();
                        next(&mut game, Event::Sort(order, matches.is_present("discard")));
                    } else if let Some(matches) = m.subcommand_matches("show") {
                        if let Some(what) = matches.value_of("what") {
                            match what {
                                "fight" => {
                                    if let State::Fighting(ref c, _, _, _, _) = game.state {
                                        println!("Fighting {:#?}", c);
                                    }
                                }
                                "cards" => {
                                    println!("pirates: {:?}", game.pirate_cards);
                                    println!("left: {:?}", game.get_left());
                                    println!("right: {:?}", game.get_right());
                                }
                                "discards" => {
                                    println!("destroyed: {:?}", game.destroyed);
                                    println!("hazard: {:?}", game.hazard_discard);
                                    println!("fighting: {:?}", game.fighting_discard);
                                }
                                _ => {}
                            }
                        }
                    } else if let Some(matches) = m.subcommand_matches("lose") {
                        let mut vals =
                            values_t!(matches.values_of("discard"), usize).unwrap_or_default();
                        next(&mut game, Event::Lose(vals.as_mut_slice()));
                    }
                }
                rl.add_history_entry(line.as_ref());
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
