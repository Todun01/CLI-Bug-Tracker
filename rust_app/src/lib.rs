use std::{error::Error, io::{self, Write}, process};
use bcrypt::{hash, verify, DEFAULT_COST};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
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
        if let Err(e) = login(user){
            eprintln!("Application error: {}", e);
            process::exit(1)
        }
    } 
    println!("Username not found");
    println!("Do you want to register?(Y/n)");
    io::stdout().flush().unwrap();

    let mut response = String::new();
    io::stdin().read_line(&mut response).expect("Error reading response.");
    if response.trim() == "Y"{
        if let Err(e) = register(){
            eprintln!("Application error: {}", e);
            process::exit(1)
        }
    }else{
        start();
    }
    Ok(())
}
pub fn login(user) -> Result<(), Box<dyn Error>>{
    Ok(())
}
pub fn register() -> Result<(), Box<dyn Error>>{
    Ok(())
}

pub fn run(items:&[String]){
    let _args = Args::parse_args(items).unwrap_or_else(|err| {
        eprintln!("Error parsing arguments: {}", err);
        process::exit(1);
    });
    if _args.query == "start"{
        //welcome prompt
        println!("🔧 Welcome to TRACER: A CLI BUg Tracker🔧");
    }
}

#[cfg(test)]

mod tests{
    use super::*;
}

#[test]
fn login_test(){

}