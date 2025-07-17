use std::io::{self, Write};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

const BASE_URL: &str = "http://localhost:3000";

#[derive(Serialize)]
struct AuthPayload {
    email: String,
    password: String,
}

#[derive(Deserialize, Debug)]
struct LoginResponse {
    token: String,
    user: User,
}

#[derive(Deserialize, Debug)]
struct RegisterResponse {
    id: String,
    email: String,
    role: String,
}

#[derive(Deserialize, Debug)]
struct User {
    id: String,
    email: String,
    role: String,
}

#[derive(Serialize)]
struct TicketCreate {
    title: String,
    description: String,
}

#[derive(Deserialize, Debug)]
struct Ticket {
    id: Uuid,
    title: String,
    description: String,
    status: String,
}

fn main() {
    println!("🔐 Welcome to the Ticket CLI");
    println!("Type `login` to log in or `register` to create an account:");

    let mode = input("> ");
    let email = input("Email: ");
    let password = input("Password: ");
    let payload = AuthPayload { email, password };

    let client = Client::new();
    let endpoint = match mode.as_str() {
        "login" => format!("{}/auth/login", BASE_URL),
        "register" => format!("{}/auth/register", BASE_URL),
        _ => {
            println!("Invalid command.");
            return;
        }
    };

    let response = client.post(&endpoint).json(&payload).send();

    if mode == "login" {
        let login: LoginResponse = match response {
        Ok(res) if res.status().is_success() => res.json().unwrap(),
        Ok(res) => {
            println!("Failed: {}", res.text().unwrap_or_default());
            return;
        }
        Err(e) => {
            println!("Network error: {}", e);
            return;
        }
        };
        println!("Signed in as {}\n", login.user.email);
        show_ticket_menu(&client, &login.token);
    }else if mode == "register" {
        let login: RegisterResponse = match response {
        Ok(res) if res.status().is_success() => res.json().unwrap(),
        Ok(res) => {
            println!("Failed: {}", res.text().unwrap_or_default());
            return;
        }
        Err(e) => {
            println!("Network error: {}", e);
            return;
        }
        };
        println!("Registered in as {}\n", login.email);
        
    };

}

fn show_ticket_menu(client: &Client, token: &str) {
    loop {
        println!("\n Ticket Menu:");
        println!("1. Create Ticket");
        println!("2. List Tickets");
        println!("3. Update Ticket");
        println!("4. Delete Ticket");
        println!("5. Exit");

        let choice = input("> ");

        match choice.as_str() {
            "1" => create_ticket(client, token),
            "2" => list_tickets(client, token),
            "3" => update_ticket(client, token),
            "4" => delete_ticket(client, token),
            "5" => break,
            _ => println!("Invalid option."),
        }
    }
}

fn create_ticket(client: &Client, token: &str) {
    let title = input("Title: ");
    let description = input("Description: ");

    let ticket = TicketCreate { title, description };

    let res = client
        .post(&format!("{}/tickets", BASE_URL))
        .bearer_auth(token)
        .json(&ticket)
        .send();

    match res {
        Ok(r) if r.status().is_success() => {
            let t: Ticket = r.json().unwrap();
            println!("Ticket created: {} ({})", t.title, t.id);
        }
        Ok(r) => println!("Error: {}", r.text().unwrap_or_default()),
        Err(e) => println!("Network error: {}", e),
    }
}

fn list_tickets(client: &Client, token: &str) {
    let res = client
        .get(&format!("{}/tickets", BASE_URL))
        .bearer_auth(token)
        .send();

    match res {
        Ok(r) if r.status().is_success() => {
            let tickets: Vec<Ticket> = r.json().unwrap();
            for t in tickets {
                println!(
                    "- [{}] {} ({})\n  {}",
                    t.status, t.title, t.id, t.description
                );
            }
        }
        Ok(r) => println!("❌ Error: {}", r.text().unwrap_or_default()),
        Err(e) => println!("❌ Network error: {}", e),
    }
}

fn update_ticket(client: &Client, token: &str) {
    let id = input("🆔 Ticket ID to update: ");
    let title = input("📝 New Title (leave blank to skip): ");
    let description = input("📄 New Description (leave blank to skip): ");
    let status = input("📌 New Status (Open/In Progress/Closed): ");

    let update = json!({
        "title": if title.is_empty() { None } else { Some(title) },
        "description": if description.is_empty() { None } else { Some(description) },
        "status": if status.is_empty() { None } else { Some(status) }
    });

    let res = client
        .put(&format!("{}/tickets/{}", BASE_URL, id))
        .bearer_auth(token)
        .json(&update)
        .send();

    match res {
        Ok(r) if r.status().is_success() => println!("✅ Ticket updated."),
        Ok(r) => println!("Error: {}", r.text().unwrap_or_default()),
        Err(e) => println!("Network error: {}", e),
    }
}

fn delete_ticket(client: &Client, token: &str) {
    let id = input("🗑️ Ticket ID to delete: ");
    let res = client
        .delete(&format!("{}/tickets/{}", BASE_URL, id))
        .bearer_auth(token)
        .send();

    match res {
        Ok(r) if r.status().is_success() => println!("✅ Ticket deleted."),
        Ok(r) => println!("❌ Error: {}", r.text().unwrap_or_default()),
        Err(e) => println!("❌ Network error: {}", e),
    }
}

fn input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut out = String::new();
    io::stdin().read_line(&mut out).unwrap();
    out.trim().to_string()
}