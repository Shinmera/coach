extern crate rand;
use std::ascii::AsciiExt;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{self, Write, BufRead, BufReader, BufWriter};
use rand::{thread_rng, Rng};

struct Card{
    query: String,
    answers: Vec<String>,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}", self.query, self.answers.join(", "))
    }
}

struct Training{
    remaining: Vec<Card>,
    failed: Vec<Card>,
    completed: Vec<Card>,
}

fn is_answer_for_card(answer: &str, card: &Card) -> bool{
    card.answers.iter().find(|&x| x.eq_ignore_ascii_case(answer)).is_some()
}

fn answer_card(answer: &str, training: &mut Training) -> bool{
    let card = training.remaining.pop().unwrap();
    if is_answer_for_card(answer, &card){
        training.completed.push(card);
        true
    }else{
        training.failed.push(card);
        false
    }
}

fn make_training(mut cards: Vec<Card>) -> Training{
    thread_rng().shuffle(&mut cards);
    Training{
        remaining: cards,
        failed: vec![],
        completed: vec![],
    }
}

fn make_card(query: String, answers: Vec<String>) -> Card{
    Card{
        query: query,
        answers: answers
    }
}

fn parse_new_card(input: &str) -> Option<(String, Vec<String>)>{
    let mut terms = input.split(",");
    if let Some(query) = terms.next(){
        let answers : Vec<String> = terms.map(|x| String::from(x.trim())).collect();
        if 0 < answers.len(){
            return Some((String::from(query.trim()), answers));
        }
    }
    None
}

fn read_dictionary(file: &str) -> Result<Vec<Card>, io::Error>{
    File::open(file).and_then(|f|{
            let reader = BufReader::new(&f);
            let mut cards = vec![];
            let mut i = 0;
            
            for line in reader.lines() {
                i += 1;
                if let Some((query, answers)) = parse_new_card(&line.unwrap()){
                    cards.push(make_card(query, answers));
                }else{
                    println!("Dictionary malformed on line {}", i);
                }
            }
            Ok(cards)
    })
}

fn train(cards: Vec<Card>){
    let mut training = make_training(cards);
    let mut input = String::new();

    loop{
        while !training.remaining.is_empty(){
            input.clear();
            print!("{}\n> ", training.remaining.last().unwrap().query);
            io::stdout().flush().ok().expect("Could not flush stdout");
            io::stdin().read_line(&mut input).expect("IO failure");
            let line = input.trim();
            if answer_card(&line, &mut training){
                println!("Correct!");
            }else{
                println!("Wrong! Possible answers: {}", training.failed.last().unwrap().answers.join(", "));
            }
        }
        
        if !training.failed.is_empty(){
            println!("You failed {} cards. Do you want to repeat them? [yes]", training.failed.len());
            io::stdin().read_line(&mut input).expect("IO failure");
            if input.trim() == "yes" || input.trim() == ""{
                training.remaining = training.failed;
                training.failed = vec![];
                continue;
            }
        }

        return;
    }
}

fn write_dictionary(file: &str, cards: Vec<Card>) -> Result<(), io::Error>{
    File::create(file).and_then(|f|{
        let mut writer = BufWriter::new(&f);

        for card in cards.iter(){
            if let Err(e) = writeln!(writer, "{}", card){
                return Err(e);
            }
        }
        Ok(())
    })
}

fn create() -> Vec<Card>{
    let mut cards: Vec<Card> = vec![];
    let mut input = String::from("__");
    while 1 < input.len(){
        input.clear();
        print!("> ");
        io::stdout().flush().ok().expect("Could not flush stdout");
        io::stdin().read_line(&mut input).expect("IO failure");
        let line = input.trim();
        if let Some((query, answers)) = parse_new_card(line){
            cards.push(make_card(query, answers));
        }else if 1 < input.len(){
            println!("Please enter a query and at least one answer separated by commas.");
        }
    }
    cards
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3{
        println!("Usage: ./coach mode dictionary");
        return;
    }
    
    let mode = &args[1];
    let file = &args[2];
    if mode == "train"{
        train(read_dictionary(&file).expect("Failed to read dictionary!"));
    }else if mode == "create"{
        write_dictionary(file, create()).expect("Failed to create dictionary!");
    }
}
