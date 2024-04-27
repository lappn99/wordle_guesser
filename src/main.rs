use std::collections::HashMap;
use std::io::BufRead;
use std::{env, io, process};
use std::fs::File;
use rand::seq::SliceRandom;

#[derive(Copy, Clone, PartialEq, Debug)]
enum CharacterStatus {
    CorrectPosition(usize),
    InWord,
    NotInWord
} 

fn main(){

    let mut rng = rand::thread_rng();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Please supply word");
        process::exit(1);
    }

    let word = &args[1];
    
    
    let legal_words: Vec<String> = read_lines("./data/legal_wotd.txt").unwrap_or_else(|err| {
        println!("{}", err.to_string());
        process::exit(1);
    }).map(|l| l.unwrap()).collect();

    let word_is_legal = legal_words.iter().any(|s| *s == *word);
    if !word_is_legal {
         println!("Word {} not legal", word);
         process::exit(1);
    }

    //Words found in legal_wotd.txt are not in guessable.txt, so have to combine them
    let word_bank: Vec<String> = read_lines("./data/guessable.txt").unwrap_or_else(|err| {
         println!("{}", err.to_string());
         process::exit(1);
    }).map(|l| l.unwrap_or_default()).chain(legal_words).collect(); 

    //Status of each character in alphabet and how in context of the chosen word
    //So if word is 'alone' and a word with 'a' in the first position was guessed it would look like:
    //'a' : CharacterStatus::CorrectPosition(0)
    let mut alphabet_status = HashMap::new();
    let mut guesed_words : Vec<String> = vec![];
    let mut num_guesses = 0;

    loop {
        //Get available words (legal words that haven't been guessed)
        let available_words: Vec<String> = word_bank.clone().into_iter().filter(|s| !guesed_words.contains(s)).collect();

        // Whole alphabet is available
        let guess: String = if alphabet_status.len() == 0 {
                //Guess random word
                available_words.choose(&mut rng).unwrap().to_string()
        } else {
            let mut word_rankings = HashMap::new();

            for word in available_words {
                let mut rank = 0;
                //Rank word by the status of each character 
                //Can tweak these rankings
                for (i, c) in word.char_indices() {
                    if alphabet_status.contains_key(&c) {
                        
                        match alphabet_status.get(&c).unwrap() {
                            CharacterStatus::CorrectPosition(index) => rank += {
                                if *index == i {
                                    5
                                } else {
                                    2
                                }
                
                            },
                            CharacterStatus::InWord => rank += 3,
                            CharacterStatus::NotInWord => rank += -10
                        }
                    } else {
                        rank += 1
                    }
                }
                word_rankings.insert(word.clone(),rank);
                
            }
            //Pick word with highest ranking
            word_rankings.iter()
                .max_by(|a, b| a.1.cmp(&b.1)).unwrap().0.to_string()
            
        };

        println!("guess: {}", guess);
        
        //Goes through each letter and updates its "Alphabet status"
        for (i, c) in guess.char_indices() {
            //If character in answer word
            if word.contains(c) {
                //Create entry for it
                let entry = alphabet_status.entry(c).or_insert({
                    if let Some(index) = word.find(c) {
                        //Same position
                        if index == i {
                            CharacterStatus::CorrectPosition(i)
                        } else { //Not in same position
                            CharacterStatus::InWord
                        }

                    } else {
                        CharacterStatus::NotInWord
                    }
                    
                });

                //If the letter is known to be in the word and its correct position is now found, update the character status
                //Obviously the reverse would not be true
                if *entry == CharacterStatus::InWord && word.find(c).unwrap() == i {
                    *entry = CharacterStatus::CorrectPosition(i);
                }


            } else {
                alphabet_status.entry(c).or_insert(CharacterStatus::NotInWord);
            }
        }

        
        num_guesses += 1;
        if guess == *word {
            println!("{} is word!\nTook {} guesses", guess, num_guesses);
            break;
        }
        guesed_words.push(guess);
        
        


    }
}

fn read_lines(filename: &str) -> io::Result<io::Lines<io::BufReader<File>>> {
    if let Ok(file) = File::open(filename) {
        return Ok(io::BufReader::new(file).lines());
    } else {
        return Err(io::Error::last_os_error());
    }
    
}




