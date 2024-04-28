use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::ExitCode;

#[derive(Copy, Clone, PartialEq, Debug)]
enum CharacterStatus {
    CorrectPosition(usize),
    InWord,
    NotInWord,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprint!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut rng = rand::thread_rng();

    let wotd = wotd_from_args()?;

    let legal_wotds: Vec<String> = BufReader::new(File::open("./data/legal_wotd.txt")?)
        .lines()
        .map(|l| l.unwrap())
        .collect();

    let wotd_is_legal = legal_wotds.iter().any(|s| s == &wotd);
    if !wotd_is_legal {
        return Err(format!("Word of the day '{}' is not legal", wotd))?;
    }

    //Words found in legal_wotd.txt are not in guessable.txt, so have to combine them
    let word_bank: Vec<String> = BufReader::new(File::open("./data/guessable.txt")?)
        .lines()
        .map(|l| l.unwrap_or_default())
        .chain(legal_wotds)
        .collect();

    //Status of each character in alphabet and how it fits in context of the chosen word
    //So if word is 'alone' and a word with 'a' in the first position was guessed it would look like:
    //'a' : CharacterStatus::CorrectPosition(0)
    let mut alphabet_status = HashMap::new();
    let mut guesed_words: Vec<&str> = vec![];
    let mut num_guesses = 0;

    loop {
        //Get all available words (words which are legal and have not been)
        let available_words = word_bank
            .iter()
            .map(|s| s.as_str())
            .filter(|s| !guesed_words.contains(s));

        // Whole alphabet is available
        let guess: &str = if alphabet_status.len() == 0 {
            //Guess random word
            let available_words: Vec<_> = available_words.collect();
            available_words.choose(&mut rng).unwrap()
        } else {
            let mut word_rankings = HashMap::new();

            for word in available_words {
                let mut rank = 0;
                //Rank word by the status of each character
                //Can tweak these rankings
                for (i, c) in word.char_indices() {
                    if alphabet_status.contains_key(&c) {
                        match alphabet_status.get(&c).unwrap() {
                            CharacterStatus::CorrectPosition(index) => {
                                rank += {
                                    if *index == i {
                                        5
                                    } else {
                                        2
                                    }
                                }
                            }
                            CharacterStatus::InWord => rank += 2,
                            CharacterStatus::NotInWord => rank += -10,
                        }
                    } else {
                        rank += 1
                    }
                }
                word_rankings.insert(word, rank);
            }
            //Pick word with highest ranking
            word_rankings.iter().max_by(|a, b| a.1.cmp(&b.1)).unwrap().0
        };

        println!("guess: {}", guess);

        //Goes through each letter and updates its "Alphabet status"
        for (i, c) in guess.char_indices() {
            //If character in answer word
            if wotd.contains(c) {
                //Create entry for it
                let entry = alphabet_status.entry(c).or_insert({
                    if let Some(index) = wotd.find(c) {
                        //Same position
                        if index == i {
                            CharacterStatus::CorrectPosition(i)
                        } else {
                            //Not in same position
                            CharacterStatus::InWord
                        }
                    } else {
                        CharacterStatus::NotInWord
                    }
                });

                //If the letter is known to be in the word and its correct position is now found, update the character status
                //Obviously the reverse would not be true
                if *entry == CharacterStatus::InWord && wotd.find(c).unwrap() == i {
                    *entry = CharacterStatus::CorrectPosition(i);
                }
            } else {
                alphabet_status
                    .entry(c)
                    .or_insert(CharacterStatus::NotInWord);
            }
        }

        num_guesses += 1;
        if guess == &wotd {
            println!("{} is word!\nTook {} guesses", guess, num_guesses);
            break;
        }
        guesed_words.push(guess);
    }

    Ok(())
}

fn wotd_from_args() -> Result<String, &'static str> {
    env::args()
        .skip(1)
        .next()
        .ok_or("Please supply word of the day")
}
