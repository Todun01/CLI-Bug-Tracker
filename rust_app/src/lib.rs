use std::{error::Error, io::{self, Write}, process};
use bcrypt::{hash, verify, DEFAULT_COST};
use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;
use rpassword::read_password;
pub struct Args{
    init_command: String,
    query: String
}

extern crate sqlx;


impl Args  {
    pub fn parse_args(_args: &[String]) -> Result <Args, String>{
        if _args.len() < 3{
            return Err(format!("You need to enter more arguments"))
        }
        if _args.len() > 3{
            return Err(format!("You entered too many arguments"))
        }
        let init_command = _args[1].clone();
        let query = _args[2].clone();
        if init_command != "tracer"{
            return Err(format!("{} is not a recognized command", init_command))
        }
        let allowed_queries = vec!["start".to_string(), "log".to_string(), 
        "view".to_string(), "update".to_string()];
        if !allowed_queries.contains(&query){
            return Err(format!("{} is not a recognized command", query))
        }
        Ok(Args{init_command, query})
    }
}

pub async fn start() -> Result<(), Box<dyn Error>>{
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
    //login prompt
    println!("Please enter your username: ");
    io::stdout().flush().unwrap();

    let mut _username = String::new();
    io::stdin().read_line(&mut _username).expect("Error reading username");
    let username: String = _username.trim().to_string();

    // check if user exists
    let user = sqlx::query!(
        "SELECT id, username, password FROM users WHERE username = $1",
        username
    )
    .fetch_optional(&pool)
    .await?;
    if let Some(user) = user{
        if let Err(e) = login(&username, pool).await{
            eprintln!("Application error: {}", e);
            process::exit(1)
        }
    } else{
        println!("{}", username);
        println!("Username not found");
        println!("Do you want to register?(Y/n)");
        io::stdout().flush().unwrap();

        let mut response = String::new();
        io::stdin().read_line(&mut response).expect("Error reading response.");
        if response.trim() == "Y"{
            if let Err(e) = register(pool).await{
                eprintln!("Application error: {}", e);
                process::exit(1)
            }
        }
        else if response.trim() == "n"{
            println!("Alright, try again.");
            io::stdout().flush().unwrap();
            process::exit(1)
        }
        else{
            eprintln!("Unrecognized response.");
        }
        
    }
    
    Ok(())
}
pub async fn login(username:&str, _pool:Pool<Postgres>) -> Result<(), Box<dyn Error>>{
    let user = sqlx::query!(
        "SELECT id, username, password FROM users WHERE username = $1",
        username
    )
    .fetch_optional(&_pool)
    .await?;
    match user {
        Some(user) => {
            println!("✅ User found.");
            print!("Enter your password: ");
            io::stdout().flush().unwrap();
            let password = read_password().unwrap();
            match verify(&password, &user.password) {
                Ok(true) => println!("✅ Logged in successfully!"),
                Ok(false) => println!("❌ Incorrect password."),
                Err(_) => println!("❌ Failed to verify password."),
            }
        }
        None => {
            println!("❌ User not found.");
        }
    }

    Ok(())
}
pub async fn register(pooL:Pool<Postgres>) -> Result<(), Box<dyn Error>>{
    println!("Please enter a username:");
    io::stdout().flush().unwrap();
    let mut _username = String::new();
    io::stdin().read_line(& mut _username).expect("Failed to read username");

    println!("Please enter a password:");
    io::stdout().flush().unwrap();
    let mut _password = String::new();
    io::stdin().read_line(& mut _password).expect("Failed to read password");

    let hashed_password = hash(_password, DEFAULT_COST).expect("Failed to hash password");

    let result = sqlx::query!(
        "INSERT INTO users (username, password) VALUES ($1, $2)", 
        _username,
        hashed_password).execute(&pooL).await;
    match result{
        Ok(_) => println!("User created successfully."),
        Err(e) => println!("Error creating user: {}", e)
    }
    Ok(())
}

pub async fn run(items:&[String]){
    let _args = Args::parse_args(items).unwrap_or_else(|err| {
        eprintln!("Error parsing arguments: {}", err);
        process::exit(1);
    });
    if _args.query == "start"{
        //welcome prompt
        println!("🔧 Welcome to TRACER: A CLI BUg Tracker🔧");
        start().await.expect("Failed to start");
    }
}

#[cfg(test)]

mod tests{
    use super::*;
}

#[test]
fn login_test(){
    //login prompt
}

#[test]

fn hasing(){
    let hashed = hash("1234", DEFAULT_COST).unwrap();
    println!("Hashed: {}", hashed);
}