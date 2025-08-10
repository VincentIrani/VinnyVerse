// WELCOME TO THE VINNYVERSE ///////////////////////////////////////////////////////////////////////////////////////////////////
// This is the Rust core of the VinnyVerse, a virtual world where players and AI agents try their best to eat each other. //////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// File Imports ////////////////////////////////////////////////////////////////////////////////////////////////////////////////
mod utils;
mod cell_def;

use cell_def::{Cell, CellKind};

// External Imports ////////////////////////////////////////////////////////////////////////////////////////////////////////////
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use std::io::{self, BufRead};

use serde::{Serialize, Deserialize};
use serde_json;

// Constant Definition /////////////////////////////////////////////////////////////////////////////////////////////////////////
const world_size: usize = 15;

// Variables/enums Definition //////////////////////////////////////////////////////////////////////////////////////////////////
//Server states
#[derive(Debug)]
enum ServerState {
    Idle,
    WorldRunning
}

// Types of Commands
#[derive(Debug)]
enum Command {
    LoadWorld,
    SaveWorld,
    GenerateWorld,
    StartWorldLoop,
    StopWorldLoop,
    StartListener,
    StopListener,
    Quit,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "payload")]
enum UserInput{
    GenerateSoul {soul_id: String},
    MountSoul {soul_id: String},
    NameSoul {soul_id: String, name: String },
    DismountSoul {soul_id: String}, 
    Activate {soul_id: String, X: u32, Y: u32, power: i16},
    Build {soul_id: String, block_type: String, X: u32, Y: u32, dir: String, power: i16},
    UpdateBrain {soul_id: String, code: String},
    ReadBrain {soul_id: String},
}

// State Machine Transition Handler //////////////////////////////////////////////////////////////////////////////////////////////////
impl ServerState {
    fn handle_command(self, command: Command) -> ServerState {
        use ServerState::*;
        use Command::*;

        match (self, command) {
            (Idle, LoadWorld) => {
                println!("Loading world...");
                Idle
            }
            (Idle, SaveWorld) => {
                println!("Saving world...");
                Idle
            }
            (Idle, GenerateWorld) => {
                println!("Generating world...");
                Idle
            }
            (Idle, StartWorldLoop) => {
                println!("Starting world loop...");
                WorldRunning
            }
            (WorldRunning, StopWorldLoop) => {
                println!("Stopping world loop...");
                Idle
            }
            (state, StartListener) => {
                println!("Starting listener...");
                state
            }
            (state, StopListener) => {
                println!("Stopping listener...");
                state
            }
            (state, cmd) => {
                println!("Command {:?} invalid in state {:?}", cmd, state);
                state
            }
        }
    }
}

// Main Loop ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[tokio::main]
async fn main() {

    // Create a channel for commands (could be from network or user input)
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<Command>();

    // Spawn a task to read user input commands from stdin
    let input_tx = cmd_tx.clone();
    tokio::spawn(async move {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            let cmd = match line.trim().to_lowercase().as_str() {
                "load_world" => Command::LoadWorld,
                "save_world" => Command::SaveWorld,
                "generate_world" => Command::GenerateWorld,
                "start_listener" => Command::StartListener,
                "stop_listener" => Command::StopListener,
                "start_world" => Command::StartWorldLoop,
                "stop_world" => Command::StopWorldLoop,
                "quit" => Command::Quit,
                _ => {
                    println!("Unknown command");
                    continue;
                }
            };
            if input_tx.send(cmd).is_err() {
                break; // receiver dropped
            }
        }
    });

    // Create input queue (channel)
    let (tx, mut rx) = mpsc::unbounded_channel::<UserInput>();

    let listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();
    println!("Server listening on 127.0.0.1:9001");

    //This task will handle incoming WebSocket connections
    tokio::spawn(async move {
        while let Ok((stream, addr)) = listener.accept().await {
            println!("New client: {}", addr);

            let tx = tx.clone();

            tokio::spawn(async move {
                let ws_stream = accept_async(stream).await.unwrap();
                let mut read = ws_stream;

                while let Some(msg_result) = read.next().await {
                    match msg_result {
                        Ok(msg) => {
                            // Send client message to the world loop
                            if msg.is_text()  {
                                let text = msg.to_text().unwrap();

                                match serde_json::from_str::<UserInput>(text) {
                                    Ok(user_input) => {
                                        if tx.send(user_input).is_err() {
                                            println!("Receiver dropped, closing client {}", addr);
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                    println!("Failed to parse JSON from client {}: {}", addr, e);
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            println!("Client {} disconnected", addr);
                            break;
                        }
                        _ => {}
                    }
                }
            });
        }
    });

    // Initial state
    let mut state = ServerState::Idle;

    let mut world = utils::generate_world(world_size);

    let mut critter_layer: Vec<Vec<Cell>> = vec![vec![Cell::empty(); world_size]; world_size];
    
    // This is the server loop
    loop {

        match state {
            ServerState::Idle => {
            sleep(Duration::from_millis(100)).await;
            }
            ServerState::WorldRunning => {
                




                // Drain all messages currently buffered in rx
                let mut batch = Vec::new();
                while let Ok(msg) = rx.try_recv() {
                    batch.push(msg);
                }

                let mut build_que: Vec<UserInput> = Vec::new();

                println!("World loop got {} messages:", batch.len());
                for msg in batch {
                    match msg{
                        UserInput::GenerateSoul { soul_id } => {
                            println!("Generating soul with ID: {}", soul_id);
                            // Here you would add logic to generate a soul
                        }
                        UserInput::MountSoul { soul_id } => {
                            println!("Mounting soul with ID: {}", soul_id);
                            // Logic to mount a soul
                        }
                        UserInput::NameSoul { soul_id, name } => {
                            println!("Naming soul: {}", name);
                            // Logic to name a soul
                        }
                        UserInput::DismountSoul {soul_id}=> {
                            println!("Dismounting current soul");
                            // Logic to dismount a soul
                        }
                        UserInput::Activate {soul_id, X, Y, power } => {
                            println!("Activating at ({}, {}), power: {}", X, Y, power);
                            // Logic to activate something in the world
                        }
                        UserInput::Build {ref soul_id, ref block_type, X, Y, ref dir, power } => {
                            println!("Building {} at ({}, {}), direction: {}, power: {}", block_type, X, Y, dir, power);
                            build_que.push(msg);
                        }
                        UserInput::UpdateBrain {soul_id, code } => {
                            println!("Updating brain with code: {}", code);
                            // Logic to update brain code
                        }
                        UserInput::ReadBrain { soul_id } => {
                            println!("Reading brain state");
                            // Logic to read brain state
                        }
                    }
                }

                
                utils::the_sun(&mut world);

                println!("{:?}", build_que);

                utils::build_critters(&mut critter_layer, &mut build_que);

                println!("World size: {}x{}", world.len(), world[0].len());
        
                //utils::visualize_world_console(&world);
                utils::visualize_critter_layer(&critter_layer);

                sleep(Duration::from_millis(10000)).await;

            }
        }

        // Handle commands from the command channel
        while let Ok(cmd) = cmd_rx.try_recv() {
            if let Command::Quit = cmd {
                println!("Quitting server.");
                return;
            }
            // Update state based on command
            state = state.handle_command(cmd);
        }

    }
}


