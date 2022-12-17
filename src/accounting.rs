use std::collections::{HashMap, HashSet};
use crate::price_printer::print_price;
use crate::receipt::Receipt;

fn resolve_person(participants: &HashSet<String>, prefix: &str) -> String {
    for person in participants {
        if person.starts_with(prefix) {
            return person.clone();
        }
    }
    if prefix == "a" {
        return String::from("all");
    }
    if prefix == "p" {
        return String::from("paulis");
    }
    return format!("Person {}", prefix);
}

pub fn summary(receipts: Vec<Receipt>) {
    let mut participants = HashSet::<String>::new();

    for receipt in &receipts {
        participants.insert(receipt.purchaser.clone());
    }

    let mut spending: HashMap<String, HashMap<String, u32>> = HashMap::new();
    for person in &participants {
        spending.insert(person.clone(), HashMap::new());
    }

    for receipt in &receipts {
        let buyer = &receipt.purchaser;
        for item in &receipt.items {
            let consumer = &item.consumer;
            let current_total: u32 = *spending[buyer].get(consumer).unwrap_or(&0);
            spending.get_mut(buyer).map(|recipients| recipients
                .insert(consumer.clone(), current_total + item.total_price()));
        }
    }

    for (buyer, recipients) in &spending {
        println!("{} spent a total of", buyer);
        for (recipient, amount) in recipients {
            println!("{} GBP on {}", print_price(*amount), resolve_person(&participants, recipient.as_str()));
        }
        println!();
    }

    // Magic constants appear here because this was the final code I needed to answer my problem.
    // Delete this if you are using this for your own purposes.
    let raitis_debt = spending["oskars"]["r"] + spending["oskars"]["a"] / 2;
    let oskars_debt = spending["raitis"]["o"] + spending["raitis"]["a"] / 2;
    println!("Raitis debt to Oscar: {} GBP", print_price(raitis_debt));
    println!("Oskars debt to Raitis: {} GBP", print_price(oskars_debt));
    if raitis_debt > oskars_debt {
        println!("Raitis owes Oscar {} GBP! :O", print_price(raitis_debt - oskars_debt));
    } else {
        println!("Oskars owes Raitis {} GBP! :O", print_price(oskars_debt - raitis_debt));
    }
}