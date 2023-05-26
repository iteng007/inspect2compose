use clap::{command, value_parser, arg};
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::io::Write;

fn main() {
    let matches = command!()
        .arg(
            arg!(
                -i --input <FILE> "Sets the input JSON file from 'docker inspect'"
            )
            .required(true)
            .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(
                -o --output <FILE> "Sets the output YAML file for Docker Compose"
            )
            .required(true)
            .value_parser(value_parser!(PathBuf)),
        )
        .get_matches();

    // Get and use arguments
    let input = matches.get_one::<PathBuf>("input").unwrap();
    let output = matches.get_one::<PathBuf>("output").unwrap();

    println!("Input JSON file: {}", input.display());
    println!("Output YAML file: {}", output.display());

    // Open JSON file
    let file = File::open(&input).expect("Could not open JSON file");
    let reader = BufReader::new(file);

    // Parse JSON file
    let json_data: Value = serde_json::from_reader(reader).expect("Could not parse JSON file");
    // println!("Parsed JSON data: {:?}", json_data);
    let json_data = json_data.as_array().unwrap().first().unwrap();
    let image = json_data["Config"]["Image"].as_str().unwrap();
    let container_name = json_data["Name"].as_str().unwrap().trim_start_matches('/');
    let restart = json_data["HostConfig"]["RestartPolicy"]["Name"].as_str().unwrap();

    let networks = json_data["NetworkSettings"]["Networks"]
        .as_object().unwrap()
        .keys()
        .collect::<Vec<&String>>();
        
    let ports = json_data["NetworkSettings"]["Ports"]
        .as_object().unwrap()
        .keys()
        .collect::<Vec<&String>>();
        
    let volumes: Vec<String> = json_data["Mounts"].as_array().unwrap().iter()
        .map(|v| format!("{}:{}", v["Source"].as_str().unwrap(), v["Destination"].as_str().unwrap()))
        .collect();

    let environment: Vec<String> = json_data["Config"]["Env"].as_array().unwrap().iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    let mut compose_str = String::new();

    compose_str += &format!("version: \"3\"\n\nservices:\n");
    compose_str += &format!("  {}:\n", container_name);
    compose_str += &format!("    image: {}\n", image);
    compose_str += &format!("    restart: {}\n", restart);
        
    compose_str += "    networks:\n";
    for network in &networks {
        compose_str += &format!("      - {}\n", network);
    }
        
    compose_str += "    ports:\n";
    for port in &ports {
        compose_str += &format!("      - \"{}\"\n", port);
    }
        
    compose_str += "    volumes:\n";
    for volume in &volumes {
        compose_str += &format!("      - \"{}\"\n", volume);
    }
        
    compose_str += "    environment:\n";
    for env in &environment {
        compose_str += &format!("      - {}\n", env);
    }
        
    compose_str += "\nnetworks:\n";
    for network in &networks {
        compose_str += &format!("  {}:\n", network);
        compose_str += "    external: true\n";
    }
        
    println!("{}", compose_str);

    // Write to output file
    let mut output_file = File::create(&output).expect("Could not create output file");
    output_file.write_all(compose_str.as_bytes()).expect("Could not write to output file");
}
