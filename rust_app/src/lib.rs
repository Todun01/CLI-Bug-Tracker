use std::{error::Error, f32::consts::E, io::{self, Write}, process::{self, exit}};
use bcrypt::{hash, verify, DEFAULT_COST};
use dotenvy::dotenv;
use sqlx::{pool, postgres::PgPoolOptions, Pool, Postgres};
use std::env;
use rpassword::read_password;
use std::pin::Pin;
use std::future::Future;

pub struct Args{
    init_command: String,
    query: String
}
pub struct AuthUser <'a>{
    id: & 'a i32,
    username: & 'a String,
    pool: Pool<Postgres>
}
extern crate sqlx;


impl Args  {
    pub fn parse_args(_args: &[String]) -> Result <Args, String>{
        // if _args[0].trim().to_lowercase() != "target/debug/rust_app"{
        // }
        if _args.len() < 3{
            return Err(format!("You need to enter more arguments"))
        }
        if _args.len() > 3{
            return Err(format!("You entered too many arguments"))
        }
        // println!("{}", _args[0]);
        let init_command = _args[1].clone();
        let query = _args[2].clone();
        if init_command != "tracer"{
            return Err(format!("{} is not a recognized command", init_command))
        }
        let allowed_query = "start".to_string();
        if !allowed_query.contains(&query){
            return Err(format!("{} is not a recognized command", query))
        }
        Ok(Args{init_command, query})
    }
    pub fn parse_session_args(_args: &[String]) -> Result<Args, String>{
        if _args.len() < 2{
            return Err(format!("You need to enter more arguments"))
        }
        if _args.len() > 2{
            return Err(format!("You entered too many arguments"))
        }
        let init_command = _args[0].clone().to_lowercase();
        let query = _args[1].clone().to_lowercase();
        if init_command != "tracer"{
            return Err(format!("{} is not a recognized command", init_command))
        }
        let allowed_queries = vec!["log".to_string(), 
        "view".to_string(), "update".to_string()];
        if !allowed_queries.contains(&query){
            println!("Allowed commands: {}", allowed_queries.join(","));
            return Err(format!("{} is not a recognized command", query))
        }
        Ok(Args{init_command, query})
    }
}

pub fn get_response()-> Result<String, Box<dyn Error>>{
    let mut response = String::new();
    io::stdin().read_line(&mut response).expect("Failed to read response");

    Ok(response)
}
pub async fn start() -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>>>>{
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    Box::pin(async move{
        let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
        // delete closed bugs after 1 day
        sqlx::query!(
            "DELETE FROM bugs WHERE status = $1 AND updated_at < NOW() - INTERVAL '1 day'",
            "closed"
        )
        .execute(&pool)
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
    })
    
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
            println!("User found.✅");
            print!("Enter your password: ");
            io::stdout().flush().unwrap();
            let mut isPassword = false;
            while !isPassword {
                let password = read_password().unwrap();
                match verify(&password, &user.password) {
                    Ok(true) => {   
                        isPassword = true;
                        println!("User authenticated! Welcome {}", &user.username);
                        println!("What would you like to do today?");
                        io::stdout().flush().unwrap();
                        let mut command = String::new();
                        io::stdin().read_line(&mut command).expect("Error reading command");
                        let mut items: Vec<String> = command
                            .trim()
                            .split(' ')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        // items.insert(0, "tracerapp".to_string());
                        let auth_user:AuthUser = AuthUser { id: &user.id, 
                            username: &user.username, 
                            pool: _pool.clone()};
                        if let Err(e) = run_in_session(&items, auth_user, _pool.clone()).await{
                            eprintln!("Application error: {}", e);
                            process::exit(1);
                        }
                        
                    },
                    Ok(false) => {
                        println!("Incorrect password.Please try again:");
                    },
                    Err(_) => println!("Failed to verify password.❌"),
                }
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

    let hashed_password = hash(_password.trim(), DEFAULT_COST).expect("Failed to hash password");

    let result = sqlx::query!(
        "INSERT INTO users (username, password) VALUES ($1, $2)", 
        _username.trim(),
        hashed_password).execute(&pooL).await;
    
    match result{
        Ok(_) => {
            println!("User created successfully.");
            println!("Do you want to login?(Y/n)");
            io::stdout().flush().unwrap();
            let mut response = String::new();
            io::stdin().read_line(&mut response).expect("Error reading response");

            if response.trim() == "Y"{
                println!("Please enter your username:");
                io::stdout().flush().unwrap();
                let mut username = String::new();
                io::stdin().read_line(&mut username).expect("Error reading username");
                if let Err(e) = login(&username.trim(), pooL).await{
                    eprintln!("Application error: {}", e);
                    process::exit(1);
                }
            } else if  response.trim() == "n"{
                println!("Alright, try again.");
                io::stdout().flush().unwrap();
                process::exit(1)
            } else {
                println!("Unrecognized response.");
                process::exit(1);
            }
        },
        Err(e) => println!("Error creating user: {}", e)
    }
    Ok(())
}
pub async fn log(user_id: i32, _pool: Pool<Postgres>) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>>>>{
    
    println!("To create a new log, please describe your bug/issue.");
    println!("Bug name:");
    io::stdout().flush().unwrap();

    let mut bug_name = String::new();
    io::stdin().read_line(&mut bug_name).expect("Failed to read bug name");
    Box::pin(async move{
        let bug = sqlx::query!(
            "SELECT name, description, status FROM bugs WHERE user_id = $1 AND name = $2",
            user_id,
            bug_name.trim()
        ).fetch_optional(&_pool)
        .await?;
        match bug{
            Some(bug) => {
                eprintln!("Sorry, that bug is already logged. Run tracer update to update it.");
                log(user_id, _pool).await.await?;
                Ok(())
            },
            None => {
                println!("Bug description (type END to finish):");
                io::stdout().flush().unwrap();
                let mut bug_desc = String::new();
                loop {
                    let mut line = String::new();
                    io::stdin().read_line(&mut line)?;
                    if line.trim() == "END" {
                        break;
                    }
                    bug_desc.push_str(&line);
                }
                sqlx::query!(
                    "INSERT INTO bugs (user_id, name, description) VALUES ($1, $2, $3)",
                    user_id,
                    bug_name.trim(),
                    bug_desc.trim(),
                )
                .execute(&_pool)
                .await?;
                println!("Bug logged successfully.✅");
                let all_bugs = sqlx::query!(
                    "SELECT name, description, status FROM bugs WHERE user_id = $1",
                    user_id
                ).fetch_all(&_pool)
                .await?;
                println!("Here are your logged bugs:");
                println!("  Bug Name  |  Bug Description  |  Status  ");
                for (i, bug) in all_bugs.iter().enumerate(){
                    println!("{}.  {}  |  {}  |  {}  ", 
                    i+1,
                    bug.name, 
                    bug.description.clone().unwrap_or("no description".to_string()), 
                    bug.status)
                }
                Ok(())
            }
            
        }
    })
    
    
}
pub async fn view(user_id: i32, POOL: Pool<Postgres>)->Result<(), Box<dyn Error>>{
//     let rows = sqlx::query_scalar!("SELECT COUNT(*) FROM bugs WHERE user_id = $1",
//     user_id
// )
//         .fetch_one(&POOL)
//         .await?;
    let all_bugs = sqlx::query!(
        "SELECT name, description, status FROM bugs WHERE user_id = $1",
        user_id
    ).fetch_all(&POOL)
    .await?;
    println!("Here are your logged bugs:");
    println!("  Bug Name  |  Bug Description  |  Status  ");
    for (i, bug) in all_bugs.iter().enumerate(){
        println!("{}.  {}  |  {}  |  {}  ", 
        i+1,
        bug.name, 
        bug.description.clone().unwrap_or("no description".to_string()), 
        bug.status)
    }
    Ok(())
}
pub async fn update(user_id: i32, pooL:Pool<Postgres>)-> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>>>>{
    println!("Update the status of a bug.");
    io::stdout().flush().unwrap();
    Box::pin(async move{
    view(user_id, pooL.clone()).await?;

    println!("Enter the name of the bug you want to update:");
    io::stdout().flush().unwrap();

    let bug_name = get_response().unwrap().trim().to_string();

    let id = sqlx::query!(
        "SELECT id FROM bugs WHERE name = $1 AND user_id = $2",
        bug_name,
        user_id
    )
    .fetch_optional(&pooL)
    .await?;
    match id{
        Some(record) => {
            let bug_id = record.id;
            println!("Bug found!");
            println!("What would you like to update?(Status/Description)");
            io::stdout().flush().unwrap();

            let response = get_response().unwrap().trim().to_string();

            if response.to_lowercase() == "status"{
                println!("NOTE: Closed bugs are automatically deleted after 24 hours.");
                println!("PLease update the status of '{}'(Open/Closed):", bug_name);
                io::stdout().flush().unwrap();

                let status = get_response().unwrap().trim().to_string();

                let bug_status = sqlx::query!(
                    "SELECT status FROM bugs WHERE id = $1",
                    bug_id
                ).fetch_one(&pooL)
                .await?;
                let allowed_commands: Vec<String> = vec!["open".to_string(), "closed".to_string()];
                if allowed_commands.contains(&status){
                    if 
                        bug_status.status.to_lowercase() == 
                        status.trim().to_lowercase(){
                            println!("Bug is already {}", status);
                            println!("Do you still wish to update?(Y/n");
                            io::stdout().flush().unwrap();

                            let response = get_response().unwrap().trim().to_string();
                            if response.eq_ignore_ascii_case("Y"){
                                update(user_id, pooL.clone()).await.await?;
                                
                            } else if response.eq_ignore_ascii_case("n"){
                                process::exit(1);
                            } else{
                                eprintln!("{} is not a recognized response", response);
                                process::exit(1);
                            }
                        } else{
                            let update = sqlx::query!(
                                "UPDATE bugs SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 AND user_id = $3",
                                status.trim(),
                                bug_id,
                                user_id
                            )
                            .execute(&pooL)
                            .await?;
                            if update.rows_affected() == 0{
                                eprintln!("Failed to update status.");
                                process::exit(1);
                            } else{
                                println!("Status updated successfully!");
                                view(user_id, pooL.clone()).await?;
                                process::exit(1);
                            }
                        }
                } else{
                    println!("{} is not a recognized command.", status);
                    process::exit(1);
                }
                
            } else if response.to_lowercase() == "description"{
                println!("Please update the description of '{}':", bug_name);
                io::stdout().flush().unwrap();

                let mut description = String::new();
                loop {
                    let mut line = String::new();
                    io::stdin().read_line(&mut line)?;
                    if line.trim() == "END" {
                        break;
                    }
                    description.push_str(&line);
                }
                let update = sqlx::query!(
                    "UPDATE bugs SET description = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 AND user_id = $3",
                    description.trim(),
                    bug_id,
                    user_id
                )
                .execute(&pooL)
                .await?;
                if update.rows_affected() == 0{
                    eprintln!("Failed to update description.");
                    process::exit(1);
                } else{
                    println!("Description updated successfully!");
                    view(user_id, pooL.clone()).await?;
                    process::exit(1);
                }
            } else{
                eprintln!("{} is not a recognized command", response);
                process::exit(1);
            }
            
        },
        None => {
            eprintln!("'{}' is not logged under this user. Do you wish to log it?(Y/n)", bug_name);
            io::stdout().flush().unwrap();

            let response = get_response().unwrap().trim().to_string();
            if response.eq_ignore_ascii_case("Y"){
                log(user_id, pooL).await.await?
            } else if response.eq_ignore_ascii_case("n") {
                process::exit(1);
            } else{
                eprintln!("{} is not a recognized response", response);
                process::exit(1);
            }
        }
    }
    Ok(())
    })
}
pub async fn run(items:&[String]) -> Result<(), Box<dyn Error>>{
    let _args = Args::parse_args(items).unwrap_or_else(|err| {
        eprintln!("Error parsing arguments: {}", err);
        process::exit(1);
    });
    if _args.query == "start"{
        //welcome prompt
        println!("🔧 Welcome to TRACER: A CLI BUg Tracker🔧");
        start().await.await.expect("Failed to start");
    }
    
    Ok(())
}
pub async fn run_in_session<'a>(items:&[String], user:AuthUser<'a>, pool:Pool<Postgres>) -> Result<(), Box<dyn Error>>{
    let _args = Args::parse_session_args(items).unwrap_or_else(|err| {
        eprintln!("Error parsing arguments: {}", err);
        process::exit(1);
    });
    if _args.query == "log"{
        log(*user.id, pool.clone()).await.await.expect("Failed to log");
    }
    else if _args.query == "view"{
        if let Err(e) = view(*user.id, pool.clone()).await{
            eprintln!("Application error: {}", e);
            process::exit(1)
        } 
    } else if _args.query == "update"{
        update(*user.id, pool.clone()).await.await.expect("Failed to update.")
    }
    Ok(())
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

#[test]

fn response_test(){
    let response = get_response().unwrap();
    println!("{}", response)
}