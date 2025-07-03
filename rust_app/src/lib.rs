use std::{error::Error, process, io};
pub struct Args{
    init_command: String,
    query: String
}

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

pub fn start() -> Result<(), Box<dyn Error>>{

    Ok(())
}
pub fn login() -> Result<(), Box<dyn Error>>{
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
        let mut username = String::new();
        io::stdin().read_line(&mut username).expect("Error reading username");
    }
}